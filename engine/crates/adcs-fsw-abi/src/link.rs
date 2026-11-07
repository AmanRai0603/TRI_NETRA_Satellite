//! adcs-link/1, engine side (fsw/targets/link/adcs_link.h, docs/VIRTUAL_OBC.md): the
//! flight software runs on a virtual OBC (a host process, QEMU Cortex-M4) or a real OBC
//! (TCP / serial bridge), and every tick the engine sends it the bus images and reads
//! back what the flight software wrote, in lockstep.
use crate::Bus;
use adcs_fsw::hal::CanFrame;
use std::io::{BufReader, BufWriter, Read, Write};
use std::net::TcpStream;
use std::process::{Child, Command, Stdio};
use std::sync::mpsc::{sync_channel, Receiver, RecvTimeoutError};
use std::time::Duration;

pub const CONFIG: u8 = 0x01;
pub const TICK: u8 = 0x02;
pub const CMD: u8 = 0x03;
pub const BYE: u8 = 0x04;
pub const ACK: u8 = 0x81;
pub const OUT: u8 = 0x82;
/// the largest payload either side takes (ADCS_LINK_MAX)
pub const MAX: usize = 4096;

/// What an OBC's refusal code means (LINK_E_* in adcs_link.h).
pub fn refusal(rc: i32) -> &'static str {
    match rc {
        -91 => "a frame arrived with a bad CRC",
        -92 => "a frame was longer than the link allows",
        -93 => "a payload did not match its own counts",
        -94 => "more UART bytes or CAN frames than the OBC holds",
        -99 => "a frame of a type the OBC does not know",
        _ => "an unknown refusal",
    }
}

/// How long the engine waits for any reply before it calls the OBC gone: $ADCS_LINK_TIMEOUT_S,
/// else 60 s (QEMU boots in well under that; a real OBC answers a tick in milliseconds).
pub fn timeout() -> Result<Duration, crate::FswError> {
    match std::env::var("ADCS_LINK_TIMEOUT_S") {
        Err(_) => Ok(Duration::from_secs(60)),
        Ok(v) => match v.trim().parse::<f64>() {
            Ok(x) if x.is_finite() && x > 0.0 => Ok(Duration::from_secs_f64(x)),
            _ => Err(crate::FswError::Refused(format!("ADCS_LINK_TIMEOUT_S = {v:?}: must be a positive number of seconds"))),
        },
    }
}

/// A reader that gives up after a timeout: a thread reads the OBC's stream and hands the bytes
/// over, so a silent or hung OBC stops the run with a message instead of blocking it forever.
struct Timed { rx: Receiver<std::io::Result<Vec<u8>>>, buf: Vec<u8>, pos: usize, wait: Duration }

impl Timed {
    fn new(mut r: Box<dyn Read + Send>, wait: Duration) -> Timed {
        let (tx, rx) = sync_channel(64);
        std::thread::spawn(move || {
            let mut b = vec![0u8; 8192];
            loop {
                match r.read(&mut b) {
                    Ok(0) => { let _ = tx.send(Ok(vec![])); return; }
                    Ok(n) => { if tx.send(Ok(b[..n].to_vec())).is_err() { return; } }
                    Err(e) if e.kind() == std::io::ErrorKind::Interrupted => {}
                    Err(e) => { let _ = tx.send(Err(e)); return; }
                }
            }
        });
        Timed { rx, buf: vec![], pos: 0, wait }
    }
}

impl Read for Timed {
    fn read(&mut self, out: &mut [u8]) -> std::io::Result<usize> {
        use std::io::{Error, ErrorKind};
        if self.pos == self.buf.len() {
            match self.rx.recv_timeout(self.wait) {
                Ok(Ok(b)) => { self.buf = b; self.pos = 0; }
                Ok(Err(e)) => return Err(e),
                Err(RecvTimeoutError::Timeout) => return Err(Error::new(ErrorKind::TimedOut, format!("no reply in {:.0} s", self.wait.as_secs_f64()))),
                Err(RecvTimeoutError::Disconnected) => return Ok(0),
            }
        }
        let n = out.len().min(self.buf.len() - self.pos);
        out[..n].copy_from_slice(&self.buf[self.pos..self.pos + n]);
        self.pos += n;
        Ok(n)
    }
}

/// Reads a reply payload field by field; a reply shorter than its own counts is an error.
struct Cur<'a> { r: &'a [u8], k: usize, what: &'static str }

impl Cur<'_> {
    fn take(&mut self, n: usize) -> Result<&[u8], crate::FswError> {
        let s = self.r.get(self.k..self.k + n).ok_or_else(|| format!("link: the OBC's {} is cut short ({} bytes)", self.what, self.r.len()))?;
        self.k += n;
        Ok(s)
    }
    fn u8(&mut self) -> Result<u8, crate::FswError> { Ok(self.take(1)?[0]) }
    fn i32(&mut self) -> Result<i32, crate::FswError> { Ok(i32::from_le_bytes(self.take(4)?.try_into().unwrap())) }
}

pub fn crc(p: &[u8], mut c: u16) -> u16 {
    for &b in p {
        c ^= (b as u16) << 8;
        for _ in 0..8 { c = if c & 0x8000 != 0 { (c << 1) ^ 0x1021 } else { c << 1 }; }
    }
    c
}

// ---------------- the recording (adcs-rec/1) ----------------
// `adcs run --record FILE` writes every frame of the link, both ways, as it crosses: the TICK the engine sent
// and the OUT the OBC answered (and CONFIG, CMD, BYE with their ACKs). A hardware-in-the-loop anomaly that
// was recorded can be replayed in soft OILS with `adcs replay FILE --fsw qemu`.
//   file    = "ADCSREC1\n" record*
//   record  = wall_ns u64 | dir u8 | len u32 | bytes[len]          (all little-endian)
//   dir 0   = engine to OBC, bytes = the whole frame (A5 5A type len payload crc)
//   dir 1   = OBC to engine, the same
//   dir 2   = a note (JSON text): the verdict of the recorded run
// wall_ns is wall time since the recording began.
pub const REC_MAGIC: &[u8] = b"ADCSREC1\n";

static REC: std::sync::Mutex<Option<(std::fs::File, std::time::Instant)>> = std::sync::Mutex::new(None);

/// Start recording the link to `path` (the file is created, or replaced).
pub fn record_to(path: &std::path::Path) -> Result<(), crate::FswError> {
    let mut f = std::fs::File::create(path).map_err(|e| format!("--record {}: {e}", path.display()))?;
    f.write_all(REC_MAGIC).map_err(|e| format!("--record {}: {e}", path.display()))?;
    *REC.lock().unwrap() = Some((f, std::time::Instant::now()));
    Ok(())
}

fn rec(dir: u8, bytes: &[u8]) {
    if let Some((f, t0)) = REC.lock().unwrap().as_mut() {
        let mut r = Vec::with_capacity(bytes.len() + 13);
        r.extend_from_slice(&(t0.elapsed().as_nanos() as u64).to_le_bytes());
        r.push(dir);
        r.extend_from_slice(&(bytes.len() as u32).to_le_bytes());
        r.extend_from_slice(bytes);
        let _ = f.write_all(&r);
    }
}

/// Add a note (the run's verdict) to the recording; nothing when no recording is on.
pub fn record_note(text: &str) { rec(2, text.as_bytes()); }

/// Stop recording (flush and close).
pub fn record_stop() { *REC.lock().unwrap() = None; }

/// One record of a recording.
#[derive(Debug, Clone, PartialEq)]
pub struct Rec { pub wall_ns: u64, pub dir: u8, pub bytes: Vec<u8> }

pub fn read_recording(path: &std::path::Path) -> Result<Vec<Rec>, crate::FswError> {
    let b = std::fs::read(path).map_err(|e| format!("{}: {e}", path.display()))?;
    if !b.starts_with(REC_MAGIC) { return Err(crate::FswError::Refused(format!("{}: not an adcs-rec/1 recording", path.display()))); }
    let (mut k, mut out) = (REC_MAGIC.len(), vec![]);
    while k < b.len() {
        let h = b.get(k..k + 13).ok_or_else(|| format!("{}: cut short at byte {k}", path.display()))?;
        let n = u32::from_le_bytes(h[9..13].try_into().unwrap()) as usize;
        let body = b.get(k + 13..k + 13 + n).ok_or_else(|| format!("{}: a record is cut short at byte {k}", path.display()))?;
        out.push(Rec { wall_ns: u64::from_le_bytes(h[..8].try_into().unwrap()), dir: h[8], bytes: body.to_vec() });
        k += 13 + n;
    }
    Ok(out)
}

/// What a replay found.
#[derive(Debug)]
pub struct Replay { pub ticks: usize, pub frames: usize, pub mismatches: usize, pub first_mismatch: Option<usize>, pub recorded_verdict: Option<String> }

/// Feed a recording's engine frames to an OBC and compare what it answers with what was recorded. The
/// OBC's answers (the commands) decide the plant's response and so the run's verdict: the same answers
/// to the same inputs reproduce the recorded verdict. The timing trailer of an OUT (the OBC's own counter
/// ticks) and the build id of a CONFIG's ACK are not compared: a different target counts differently.
pub fn replay(path: &std::path::Path, t: &Target) -> Result<Replay, crate::FswError> {
    let recs = read_recording(path)?;
    let mut l = Link::open(t)?;
    let mut rep = Replay { ticks: 0, frames: 0, mismatches: 0, first_mismatch: None, recorded_verdict: None };
    let mut answers = recs.iter().filter(|x| x.dir == 1);
    for r in &recs {
        if r.dir == 2 { rep.recorded_verdict = Some(String::from_utf8_lossy(&r.bytes).into_owned()); continue; }
        if r.dir != 0 { continue; }
        let ty = *r.bytes.get(2).ok_or("recording: a frame shorter than its header")?;
        l.wr.write_all(&r.bytes).and_then(|_| l.wr.flush()).map_err(|e| format!("link write: {e}"))?;
        let (got_ty, got) = l.recv()?;
        rep.frames += 1;
        if ty == TICK { rep.ticks += 1; }
        let Some(want) = answers.next() else { continue };
        let (wty, wp) = (want.bytes[2], &want.bytes[5..want.bytes.len() - 2]);
        let same = got_ty == wty && match wty {
            OUT if got.len() >= 8 && wp.len() >= 8 => got[..got.len() - 8] == wp[..wp.len() - 8],
            ACK if ty == CONFIG => got.get(..4) == wp.get(..4),
            _ => got == wp,
        };
        if !same {
            rep.mismatches += 1;
            rep.first_mismatch.get_or_insert(rep.frames - 1);
        }
    }
    Ok(rep)
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
    rd: Timed,
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
    /// why the last tick or command failed, for the run's error message
    pub error: Option<String>,
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
    pub fn open(t: &Target) -> Result<Link, crate::FswError> { Self::open_with(t, timeout()?) }

    /// open, waiting at most `wait` for any reply
    pub fn open_with(t: &Target, wait: Duration) -> Result<Link, crate::FswError> {
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
        let rd = Timed::new(rd, wait);
        Ok(Link { child, rd, wr, build_id: String::new(), debug: vec![], bytes_tx: 0, bytes_rx: 0, exec_ticks: 0, clock_hz: 0, counts, insn: None, error: None })
    }

    fn send(&mut self, ty: u8, p: &[u8]) -> Result<(), crate::FswError> {
        if p.len() > MAX { return Err(format!("link: a {} byte frame is over the link's {MAX} byte limit", p.len()).into()); }
        let mut f = Vec::with_capacity(p.len() + 7);
        f.extend_from_slice(&[0xA5, 0x5A, ty]);
        f.extend_from_slice(&(p.len() as u16).to_le_bytes());
        f.extend_from_slice(p);
        let c = crc(&f[2..], 0xFFFF);
        f.extend_from_slice(&c.to_le_bytes());
        self.wr.write_all(&f).and_then(|_| self.wr.flush()).map_err(|e| format!("link write: {e}"))?;
        rec(0, &f);
        self.bytes_tx += f.len() as u64;
        Ok(())
    }

    fn recv(&mut self) -> Result<(u8, Vec<u8>), crate::FswError> {
        let mut b = [0u8; 1];
        let mut prev = 0u16;
        loop {
            self.rd.read_exact(&mut b).map_err(|e| format!("link read (the OBC is gone or hung): {e}"))?;
            if prev == 0xA5 && b[0] == 0x5A { break; }
            prev = b[0] as u16;
        }
        let mut h = [0u8; 3];
        self.rd.read_exact(&mut h).map_err(|e| format!("link read (the OBC is gone or hung): {e}"))?;
        let n = u16::from_le_bytes([h[1], h[2]]) as usize;
        if n > MAX + 8 { return Err(format!("link: the OBC sent a {n} byte frame, over the link's limit").into()); }
        let mut p = vec![0u8; n + 2];
        self.rd.read_exact(&mut p).map_err(|e| format!("link read (the OBC is gone or hung): {e}"))?;
        let c = crc(&p[..n], crc(&h, 0xFFFF));
        if c != u16::from_le_bytes([p[n], p[n + 1]]) { return Err("link: CRC error from the OBC".into()); }
        self.bytes_rx += (n + 7) as u64;
        let mut whole = vec![0xA5, 0x5A];
        whole.extend_from_slice(&h);
        whole.extend_from_slice(&p);
        rec(1, &whole);
        p.truncate(n);
        Ok((h[0], p))
    }

    /// CONFIG: boot the flight software with its adcs-fswcfg/1 blob.
    pub fn config(&mut self, blob: &[u8], start_ns: u64) -> Result<i32, crate::FswError> {
        let mut p = start_ns.to_le_bytes().to_vec();
        p.extend_from_slice(blob);
        self.send(CONFIG, &p)?;
        let (ty, r) = self.recv()?;
        if ty != ACK { return Err(format!("link: expected ACK to CONFIG, got 0x{ty:02x}").into()); }
        let mut c = Cur { r: &r, k: 0, what: "ACK to CONFIG" };
        let rc = c.i32()?;
        let n = c.u8()? as usize;
        self.build_id = String::from_utf8_lossy(c.take(n)?).into_owned();
        Ok(rc)
    }

    /// TICK: bus images in, flight-software writes out (PWM, CAN) back onto the bus.
    pub fn tick(&mut self, bus: &mut Bus, now_ns: u64) -> Result<i32, crate::FswError> {
        let mut p = Vec::with_capacity(160);
        p.extend_from_slice(&now_ns.to_le_bytes());
        let present = bus.mag.is_some() as u8 | (bus.gyro.is_some() as u8) << 1 | (bus.sun.is_some() as u8) << 2 | (bus.es.is_some() as u8) << 3
            | (bus.pwm_fail as u8) << 4 | (bus.can_tx_fail as u8) << 5;
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
        // the OBC holds 32 CAN frames a tick: more would be lost, so the run stops instead
        let n = bus.can_rx.len();
        if n > 32 { return Err(format!("link: {n} CAN frames for one tick; the OBC takes at most 32").into()); }
        p.push(n as u8);
        for f in bus.can_rx.drain(..) {
            p.extend_from_slice(&f.id.to_le_bytes());
            p.push(f.dlc);
            p.extend_from_slice(&f.data);
        }
        self.send(TICK, &p)?;
        let (ty, r) = self.recv()?;
        if ty == ACK {
            let rc = Cur { r: &r, k: 0, what: "refusal" }.i32()?;
            return Err(format!("link: the OBC refused the tick ({rc}: {})", refusal(rc)).into());
        }
        if ty != OUT { return Err(format!("link: expected OUT, got 0x{ty:02x}").into()); }
        // read the whole reply before anything reaches the bus: a short OUT changes nothing
        let mut c = Cur { r: &r, k: 0, what: "OUT" };
        let rc = c.i32()?;
        let mut pwm = [0i16; 8];
        for w in pwm.iter_mut() { *w = i16::from_le_bytes(c.take(2)?.try_into().unwrap()); }
        let nc = c.u8()? as usize;
        let mut tx = Vec::with_capacity(nc);
        for _ in 0..nc {
            let f = c.take(13)?;
            tx.push(CanFrame { id: u32::from_le_bytes(f[..4].try_into().unwrap()), extended: 0, dlc: f[4], data: f[5..13].try_into().unwrap() });
        }
        let nd = c.u8()? as usize;
        let mut debug = Vec::with_capacity(nd);
        for _ in 0..nd { debug.push(f64::from_le_bytes(c.take(8)?.try_into().unwrap())); }
        // the timing trailer is optional (an OBC that predates it), but never partial
        let (ticks, hz) = match r.len() - c.k {
            0 => (0, 0),
            8 => (c.i32()? as u32, c.i32()? as u32),
            x => return Err(format!("link: the OBC's OUT has {x} bytes after its debug values; the timing trailer is 8").into()),
        };
        bus.pwm = pwm;
        bus.can_tx.extend(tx);
        self.debug = debug;
        self.exec_ticks = ticks;
        self.clock_hz = hz;
        // the plugin wrote this step's count before the firmware sent OUT
        if let Some((f, _)) = self.counts.as_mut() {
            let mut b = [0u8; 8];
            // a missing count must stop the run: falling back to another clock would mix time bases
            f.read_exact(&mut b).map_err(|e| format!("link: no instruction count from the QEMU plugin ({e}); is the disk full?"))?;
            self.insn = Some(u64::from_le_bytes(b));
        }
        Ok(rc)
    }

    pub fn command(&mut self, tc: &[u8]) -> Result<i32, crate::FswError> {
        self.send(CMD, tc)?;
        let (ty, r) = self.recv()?;
        if ty != ACK { return Err(format!("link: expected ACK to a command, got 0x{ty:02x}").into()); }
        Cur { r: &r, k: 0, what: "ACK to a command" }.i32()
    }
}

impl Drop for Link {
    fn drop(&mut self) {
        // the reader times out, so a dead OBC cannot hold the engine here; one that did not
        // say goodbye is stopped rather than waited on
        let bye = self.send(BYE, &[]).and_then(|_| self.recv());
        if let Some(c) = self.child.as_mut() {
            if !matches!(bye, Ok((ACK, _))) { let _ = c.kill(); }
            let _ = c.wait();
        }
        if let Some((_, p)) = self.counts.take() { let _ = std::fs::remove_file(p); }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_silent_obc_stops_the_run_instead_of_hanging_it() {
        let t = Target::Spawn(vec!["sleep".into(), "30".into()]);
        let mut l = Link::open_with(&t, Duration::from_millis(300)).unwrap();
        let t0 = std::time::Instant::now();
        let e = l.config(&[], 0).unwrap_err();
        assert!(matches!(&e, crate::FswError::Link(m) if m.contains("gone or hung")), "{e}");
        drop(l);
        assert!(t0.elapsed() < Duration::from_secs(5), "the silent OBC is stopped, not waited on");
    }

    #[test]
    fn a_short_reply_is_an_error_not_a_panic() {
        let mut c = Cur { r: &[1, 2, 3], k: 0, what: "OUT" };
        assert!(c.i32().unwrap_err().message().contains("cut short"));
    }
}
