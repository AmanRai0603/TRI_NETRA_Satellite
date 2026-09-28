//! adcs-link/1, engine side (fsw/targets/link/adcs_link.h, docs/VIRTUAL_OBC.md): the
//! flight software runs on a virtual OBC (a host process, QEMU Cortex-M4) or a real OBC
//! (TCP / serial bridge), and every tick the engine sends it the bus images and reads
//! back what the flight software wrote, in lockstep.
use crate::Bus;
use adcs_fsw::hal::CanFrame;
use std::io::{BufReader, BufWriter, Read, Write};
use std::net::TcpStream;
use std::process::{Child, Command, Stdio};

pub const CONFIG: u8 = 0x01;
pub const TICK: u8 = 0x02;
pub const CMD: u8 = 0x03;
pub const BYE: u8 = 0x04;
pub const ACK: u8 = 0x81;
pub const OUT: u8 = 0x82;

pub fn crc(p: &[u8], mut c: u16) -> u16 {
    for &b in p {
        c ^= (b as u16) << 8;
        for _ in 0..8 { c = if c & 0x8000 != 0 { (c << 1) ^ 0x1021 } else { c << 1 }; }
    }
    c
}

/// Where the OBC is.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Target {
    /// spawn a program and speak on its stdin/stdout (a POSIX virtual OBC, or QEMU with -serial stdio)
    Spawn(Vec<String>),
    /// connect to host:port (a virtual OBC started with --listen, or a serial-to-TCP bridge to a real OBC)
    Tcp(String),
}

pub struct Link {
    child: Option<Child>,
    rd: Box<dyn Read + Send>,
    wr: Box<dyn Write + Send>,
    pub build_id: String,
    pub debug: Vec<f64>,
    pub bytes_tx: u64,
    pub bytes_rx: u64,
    /// timing trailer of the last OUT: counter ticks inside adcs_fsw_step and the counter rate
    pub exec_ticks: u32,
    pub clock_hz: u32,
    /// QEMU with the insn_count plugin: the exact guest instructions of every step
    counts: Option<(std::fs::File, std::path::PathBuf)>,
    pub insn: Option<u64>,
}

impl Link {
    /// The last step on the OBC: (seconds by the OBC's own counter, exact guest instructions).
    pub fn exec(&self) -> Option<(f64, Option<f64>)> {
        if self.clock_hz == 0 && self.insn.is_none() { return None; }
        let s = if self.clock_hz > 0 { self.exec_ticks as f64/self.clock_hz as f64 } else { 0.0 };
        Some((s, self.insn.map(|n| n as f64)))
    }
}

impl Link {
    pub fn open(t: &Target) -> Result<Link, String> {
        static N: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
        let mut counts = None;
        let mut t = t.clone();
        if let Target::Spawn(cmd) = &mut t {
            if cmd.iter().any(|a| a.contains("{COUNTS}")) {
                let p = std::env::temp_dir().join(format!("adcs-insn-{}-{}.bin", std::process::id(), N.fetch_add(1, std::sync::atomic::Ordering::Relaxed)));
                std::fs::write(&p, []).map_err(|e| format!("{}: {e}", p.display()))?;
                for a in cmd.iter_mut() { *a = a.replace("{COUNTS}", &p.display().to_string()); }
                counts = Some((std::fs::File::open(&p).map_err(|e| e.to_string())?, p));
            }
        }
        let t = &t;
        let (child, rd, wr): (Option<Child>, Box<dyn Read + Send>, Box<dyn Write + Send>) = match t {
            Target::Spawn(cmd) => {
                let mut c = Command::new(&cmd[0]).args(&cmd[1..]).stdin(Stdio::piped()).stdout(Stdio::piped()).stderr(Stdio::inherit())
                    .spawn().map_err(|e| format!("cannot start the virtual OBC {:?}: {e}", cmd))?;
                let rd = BufReader::new(c.stdout.take().unwrap());
                let wr = BufWriter::new(c.stdin.take().unwrap());
                (Some(c), Box::new(rd), Box::new(wr))
            }
            Target::Tcp(addr) => {
                let s = TcpStream::connect(addr).map_err(|e| format!("cannot reach the OBC at {addr}: {e}"))?;
                s.set_nodelay(true).ok();
                let r = s.try_clone().map_err(|e| e.to_string())?;
                (None, Box::new(BufReader::new(r)), Box::new(BufWriter::new(s)))
            }
        };
        Ok(Link { child, rd, wr, build_id: String::new(), debug: vec![], bytes_tx: 0, bytes_rx: 0, exec_ticks: 0, clock_hz: 0, counts, insn: None })
    }

    fn send(&mut self, ty: u8, p: &[u8]) -> Result<(), String> {
        let mut f = Vec::with_capacity(p.len() + 7);
        f.extend_from_slice(&[0xA5, 0x5A, ty]);
        f.extend_from_slice(&(p.len() as u16).to_le_bytes());
        f.extend_from_slice(p);
        let c = crc(&f[2..], 0xFFFF);
        f.extend_from_slice(&c.to_le_bytes());
        self.wr.write_all(&f).and_then(|_| self.wr.flush()).map_err(|e| format!("link write: {e}"))?;
        self.bytes_tx += f.len() as u64;
        Ok(())
    }

    fn recv(&mut self) -> Result<(u8, Vec<u8>), String> {
        let mut b = [0u8; 1];
        let mut prev = 0u16;
        loop {
            self.rd.read_exact(&mut b).map_err(|e| format!("link read (OBC gone?): {e}"))?;
            if prev == 0xA5 && b[0] == 0x5A { break; }
            prev = b[0] as u16;
        }
        let mut h = [0u8; 3];
        self.rd.read_exact(&mut h).map_err(|e| e.to_string())?;
        let n = u16::from_le_bytes([h[1], h[2]]) as usize;
        let mut p = vec![0u8; n + 2];
        self.rd.read_exact(&mut p).map_err(|e| e.to_string())?;
        let c = crc(&p[..n], crc(&h, 0xFFFF));
        if c != u16::from_le_bytes([p[n], p[n + 1]]) { return Err("link: CRC error from the OBC".into()); }
        self.bytes_rx += (n + 7) as u64;
        p.truncate(n);
        Ok((h[0], p))
    }

    /// CONFIG: boot the flight software with its adcs-fswcfg/1 blob.
    pub fn config(&mut self, blob: &[u8], start_ns: u64) -> Result<i32, String> {
        let mut p = start_ns.to_le_bytes().to_vec();
        p.extend_from_slice(blob);
        self.send(CONFIG, &p)?;
        let (ty, r) = self.recv()?;
        if ty != ACK || r.len() < 5 { return Err("link: no ACK to CONFIG".into()); }
        let n = r[4] as usize;
        self.build_id = String::from_utf8_lossy(&r[5..5 + n.min(r.len() - 5)]).into_owned();
        Ok(i32::from_le_bytes([r[0], r[1], r[2], r[3]]))
    }

    /// TICK: bus images in, flight-software writes out (PWM, CAN) back onto the bus.
    pub fn tick(&mut self, bus: &mut Bus, now_ns: u64) -> Result<i32, String> {
        let mut p = Vec::with_capacity(160);
        p.extend_from_slice(&now_ns.to_le_bytes());
        let present = bus.mag.is_some() as u8 | (bus.gyro.is_some() as u8) << 1 | (bus.sun.is_some() as u8) << 2 | (bus.es.is_some() as u8) << 3;
        p.push(present);
        p.extend_from_slice(&bus.mag.unwrap_or([0; 7]));
        p.extend_from_slice(&bus.gyro.unwrap_or([0; 13]));
        p.extend_from_slice(&bus.sun.unwrap_or([0; 7]));
        p.extend_from_slice(&bus.es.unwrap_or([0; 7]));
        for port in 1..=2 {
            let q: Vec<u8> = bus.uart[port].drain(..).collect();
            p.extend_from_slice(&(q.len() as u16).to_le_bytes());
            p.extend_from_slice(&q);
        }
        let n = bus.can_rx.len().min(32);
        p.push(n as u8);
        for f in bus.can_rx.drain(..n) {
            p.extend_from_slice(&f.id.to_le_bytes());
            p.push(f.dlc);
            p.extend_from_slice(&f.data);
        }
        bus.can_rx.clear();
        self.send(TICK, &p)?;
        let (ty, r) = self.recv()?;
        if ty != OUT { return Err(format!("link: expected OUT, got 0x{ty:02x}")); }
        let rc = i32::from_le_bytes([r[0], r[1], r[2], r[3]]);
        let mut k = 4;
        for i in 0..8 { bus.pwm[i] = i16::from_le_bytes([r[k], r[k + 1]]); k += 2; }
        let nc = r[k] as usize; k += 1;
        for _ in 0..nc {
            let mut data = [0u8; 8];
            data.copy_from_slice(&r[k + 5..k + 13]);
            bus.can_tx.push(CanFrame { id: u32::from_le_bytes([r[k], r[k + 1], r[k + 2], r[k + 3]]), extended: 0, dlc: r[k + 4], data });
            k += 13;
        }
        let nd = r[k] as usize; k += 1;
        self.debug.clear();
        for _ in 0..nd { let mut x = [0u8; 8]; x.copy_from_slice(&r[k..k + 8]); self.debug.push(f64::from_le_bytes(x)); k += 8; }
        if r.len() >= k + 8 {
            self.exec_ticks = u32::from_le_bytes([r[k], r[k + 1], r[k + 2], r[k + 3]]);
            self.clock_hz = u32::from_le_bytes([r[k + 4], r[k + 5], r[k + 6], r[k + 7]]);
        }
        // the plugin wrote this step's count before the firmware sent OUT
        if let Some((f, _)) = self.counts.as_mut() {
            let mut b = [0u8; 8];
            // a missing count must stop the run: falling back to another clock would mix time bases
            f.read_exact(&mut b).map_err(|e| format!("link: no instruction count from the QEMU plugin ({e}); is the disk full?"))?;
            self.insn = Some(u64::from_le_bytes(b));
        }
        Ok(rc)
    }

    pub fn command(&mut self, tc: &[u8]) -> Result<i32, String> {
        self.send(CMD, tc)?;
        let (_, r) = self.recv()?;
        Ok(i32::from_le_bytes([r[0], r[1], r[2], r[3]]))
    }
}

impl Drop for Link {
    fn drop(&mut self) {
        let _ = self.send(BYE, &[]);
        let _ = self.recv();
        if let Some(c) = self.child.as_mut() { let _ = c.wait(); }
        if let Some((_, p)) = self.counts.take() { let _ = std::fs::remove_file(p); }
    }
}
