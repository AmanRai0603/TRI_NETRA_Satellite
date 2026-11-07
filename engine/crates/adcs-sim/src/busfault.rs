//! Bus-level faults of the rig (O11, docs/S11_INVENTORY.md; soft OILS and SILS alike): what the interface emulation unit does to a
//! device's bytes on the way to the flight software, and to the flight software's words on the way to the device. Glue and no model:
//! each fault says which device (a name of the device map, fsw/devices/example_3u.toml) and which of six things happens to its bus,
//! and for how long; nothing here is a relation of the spacecraft. The twin does the same on the same bytes
//! (matlab_sils/+asils/+faults/bus_sensors.m, bus_actuators.m).
//!
//!   bus_nack        the device does not answer (an I2C or SPI transfer fails, a UART or CAN stream is silent); on the output side the
//!                   write fails with a bus status and the device keeps its previous command
//!   bus_crc         a UART frame's CRC is wrong (the star tracker's and the GNSS receiver's: the other buses carry none)
//!   bus_stuck       the register, frame or command stops changing: the last good one is repeated, flagged as good
//!   bus_delay       the answer comes `value` seconds late: the command reaches the actuators that much later (the soft-OILS latency)
//!   bus_brownout    the device's supply drops for `value` seconds: it does not answer, and a load it drives (coils, rotors) gets no command
//!   bus_can_error   the CAN controller sees an error frame: nothing is received or sent while it lasts (a Bus status), commands are held
//!
//! Owner: Agastya. Copyright (c) 2026 Agastya. All rights reserved.
use crate::config::Fault;
use adcs_fsw::hal::CanFrame;
use adcs_fsw_abi::Bus;
use adcs_sim_core::emu::{proto, Commands};

/// The device map's devices (fsw/devices/example_3u.toml), in its order: one fault point each.
pub const DEVICES: [&str; 8] = ["magnetometer", "sun_sensor_plus_x", "earth_sensor", "star_tracker", "gyro", "gnss", "coils", "momentum_devices"];
pub const KINDS: [&str; 6] = ["bus_nack", "bus_crc", "bus_stuck", "bus_delay", "bus_brownout", "bus_can_error"];
const MAG: usize = 0;
const SUN: usize = 1;
const ES: usize = 2;
const ST: usize = 3;
const GYRO: usize = 4;
const GNSS: usize = 5;
const COILS: usize = 6;
const MEX: usize = 7;

/// The bus a device sits on, by the map's `hal.kind`.
pub fn bus_of(device: &str) -> Option<&'static str> {
    Some(match device {
        "magnetometer" | "sun_sensor_plus_x" | "earth_sensor" => "i2c",
        "gyro" => "spi",
        "star_tracker" | "gnss" => "uart",
        "coils" => "pwm",
        "momentum_devices" => "can",
        _ => return None,
    })
}
/// Which of the six a device's bus can suffer: a CRC where the bus carries one (the UART frames), an error frame where it has one (CAN).
pub fn allowed(kind: &str, device: &str) -> bool {
    let Some(b) = bus_of(device) else { return false };
    match kind {
        "bus_crc" => b == "uart",
        "bus_can_error" => b == "can",
        k => KINDS.contains(&k),
    }
}
/// The device's index in DEVICES.
pub fn device_index(device: &str) -> Option<usize> { DEVICES.iter().position(|d| *d == device) }

#[derive(Clone, Debug)]
struct Win { kind: usize, dev: usize, t0: f64, t1: f64, delay: f64 }

/// Where the bytes of a tick start in the bus's queues, before the emulators write their frames.
#[derive(Clone, Copy, Debug, Default)]
pub struct Mark { st: usize, gnss: usize, can: usize }

#[derive(Clone, Debug, Default)]
pub struct BusFaults {
    w: Vec<Win>,
    mag: Option<[u8; 7]>, sun: Option<[u8; 7]>, es: Option<[u8; 7]>, gyro: Option<[u8; 13]>,
    st: Vec<u8>, gnss: Vec<u8>, can: Vec<CanFrame>, prev: Commands,
}

impl BusFaults {
    pub fn new(faults: &[Fault]) -> BusFaults {
        let mut w = vec![];
        for f in faults {
            let Some(kind) = KINDS.iter().position(|k| *k == f.kind) else { continue };
            let Some(dev) = device_index(&f.device) else { continue };
            let t1 = match f.kind.as_str() { "bus_brownout" => f.t_s + f.value[0], _ => f.end_s.unwrap_or(f64::INFINITY) };
            w.push(Win { kind, dev, t0: f.t_s, t1, delay: if f.kind == "bus_delay" { f.value[0] } else { 0.0 } });
        }
        BusFaults { w, ..Default::default() }
    }
    fn on(&self, t: f64, dev: usize, kinds: &[&str]) -> bool {
        self.w.iter().any(|x| x.dev == dev && t >= x.t0 && t < x.t1 && kinds.contains(&KINDS[x.kind]))
    }
    /// The delay the answers carry at t [s] (the longest of those in force).
    pub fn delay_s(&self, t: f64) -> f64 {
        self.w.iter().filter(|x| KINDS[x.kind] == "bus_delay" && t >= x.t0 && t < x.t1).fold(0.0, |a, x| a.max(x.delay))
    }

    pub fn mark(bus: &Bus) -> Mark {
        Mark { st: bus.uart[proto::ST_UART as usize].len(), gnss: bus.uart[proto::GPS_UART as usize].len(), can: bus.can_rx.len() }
    }

    /// Before the flight software's step: what the devices put on the buses this tick (written by the emulators after `mark`) as the
    /// fault leaves it, and the status the output side will answer.
    pub fn sensors(&mut self, t: f64, bus: &mut Bus, m: Mark) {
        if self.w.is_empty() { return; }
        fn block<const N: usize>(blk: &mut Option<[u8; N]>, last: &mut Option<[u8; N]>, dead: bool, stuck: bool) {
            if dead { *blk = None; }
            else if stuck {
                if last.is_none() { *last = *blk; }
                *blk = *last;
            } else { *last = *blk; }
        }
        let dead = |s: &Self, d: usize| s.on(t, d, &["bus_nack", "bus_brownout"]);
        let stuck = |s: &Self, d: usize| s.on(t, d, &["bus_stuck"]);
        let (d, s) = (dead(self, MAG), stuck(self, MAG));
        block(&mut bus.mag, &mut self.mag, d, s);
        let (d, s) = (dead(self, SUN), stuck(self, SUN));
        block(&mut bus.sun, &mut self.sun, d, s);
        let (d, s) = (dead(self, ES), stuck(self, ES));
        block(&mut bus.es, &mut self.es, d, s);
        let (d, s) = (dead(self, GYRO), stuck(self, GYRO));
        block(&mut bus.gyro, &mut self.gyro, d, s);
        // the UART frames written this tick: silent, with a bad CRC (the frame's last byte), or the last one again
        for (port, dev, from) in [(proto::ST_UART as usize, ST, m.st), (proto::GPS_UART as usize, GNSS, m.gnss)] {
            if bus.uart[port].len() <= from { continue; }
            let (is_dead, crc, is_stuck) = (dead(self, dev), self.on(t, dev, &["bus_crc"]), stuck(self, dev));
            let last = if dev == ST { &mut self.st } else { &mut self.gnss };
            let q = &mut bus.uart[port];
            if is_dead { q.truncate(from); continue; }
            if crc { let n = q.len(); q[n - 1] ^= 0x01; }
            if is_stuck {
                if last.is_empty() { last.extend(q.iter().skip(from).copied()); }
                q.truncate(from);
                q.extend(last.iter().copied());
            } else { *last = q.iter().skip(from).copied().collect(); }
        }
        // the rotors' telemetry frames: lost (no answer, an error frame) or the last of each identifier again
        let (cerr, cdead, cstuck) = (self.on(t, MEX, &["bus_can_error"]), dead(self, MEX), stuck(self, MEX));
        bus.can_error = cerr;
        if cdead || cerr { bus.can_rx.truncate(m.can); }
        else {
            for i in m.can..bus.can_rx.len() {
                let f = bus.can_rx[i];
                match (self.can.iter().position(|x| x.id == f.id), cstuck) {
                    (Some(j), true) => bus.can_rx[i].data = self.can[j].data,
                    (Some(j), false) => self.can[j] = f,
                    (None, _) => self.can.push(f),
                }
            }
        }
        bus.pwm_fail = self.on(t, COILS, &["bus_nack"]);
        bus.can_tx_fail = self.on(t, MEX, &["bus_nack", "bus_can_error"]);
    }

    /// After the step: the commands that reach the actuators. A failed or stuck write leaves the device on its previous command; a
    /// device without supply gets none.
    pub fn commands(&mut self, t: f64, cmd: &mut Commands) {
        if self.w.is_empty() { return; }
        if self.on(t, COILS, &["bus_nack", "bus_stuck"]) { cmd.m_body = self.prev.m_body; }
        else if self.on(t, COILS, &["bus_brownout"]) { cmd.m_body = [0.0; 3]; }
        if self.on(t, MEX, &["bus_nack", "bus_stuck", "bus_can_error"]) { cmd.cmd_r = self.prev.cmd_r; cmd.cmd_g = self.prev.cmd_g; cmd.duty = self.prev.duty; }
        else if self.on(t, MEX, &["bus_brownout"]) { cmd.cmd_r = Default::default(); cmd.cmd_g = Default::default(); cmd.duty = Default::default(); }
        self.prev = *cmd;
    }
}
