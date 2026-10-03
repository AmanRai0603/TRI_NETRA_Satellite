
---

## 11. The loop contract — `adcs-bus::loop`

The bus crate is written so that "the identical message types reach the rig and a flight target" (`adcs-bus/src/lib.rs`). The loop contract is a new module in it, `no_std` with `alloc`, `#![forbid(unsafe_code)]`, one struct per message.

### 11.1 Messages

```rust
pub struct TimeSync      { pub tick: u64, pub t_ns: u64, pub pps_edge: bool }
pub struct DeviceBytes   { pub device: DeviceId, pub port: PortRef, pub t_ns: u64, pub bytes: Bytes<256> }  // plant -> emulator -> OBC
pub struct CanFrame      { pub port: u8, pub t_ns: u64, pub id: u32, pub extended: bool, pub dlc: u8, pub data: [u8; 8] }
pub struct PwmCapture    { pub channel: u8, pub t_ns: u64, pub duty_q15: i16, pub period_ns: u32 }              // OBC -> rig
pub struct GpioEdge      { pub pin: u16, pub t_ns: u64, pub level: u8 }
pub struct TelemetryPacket { pub apid: u16, pub t_ns: u64, pub payload: Bytes<1024> }                          // OBC -> rig, recorded verbatim
pub struct Telecommand   { pub t_ns: u64, pub payload: Bytes<512> }                                            // rig -> OBC
pub struct FaultInject   { pub at_tick: u64, pub kind: FaultKind, pub target: DeviceId, pub duration_ticks: u32 }
pub struct TruthSample   { pub tick: u64, pub q_bi: [f64; 4], pub w_b: [f64; 3], pub b_b: [f64; 3], pub h_int: [f64; 3] }
pub struct RunControl    { pub command: RunCommand /* Arm | Start | Pause | Resume | Stop | Abort */, pub run_id: RunId }
pub struct Heartbeat     { pub node: NodeId, pub seq: u32, pub deadline_misses: u32, pub worst_latency_ns: u32 }
```

`DeviceId`, `PortRef`, `FaultKind`, `RunId` and `NodeId` are small enums and newtypes. `Bytes<N>` is a fixed-capacity byte buffer.

### 11.2 Framing and transport

Between the rig host and the interface emulation unit, the contract travels over UDP on a dedicated link:

- one datagram per rig cycle, carrying every message for that cycle;
- a header with version, sequence number and cycle tick;
- a CRC-32 over the payload.

A lost or late datagram is counted, never retried within the cycle. The emulator holds its last value and flags it, and both sides record the event. The version is negotiated at `Arm`; a mismatch refuses the run by name.

In SILS the same messages pass in-process through a channel, so the emulator code is the same code.

### 11.3 Timing

The rig cycle is `plant_step_s` from the device map, 1 kHz in the example. Every message carries the tick it belongs to. The OBC runs on its own clock, disciplined by the rig's PPS line and a `TimeSync` packet. The rig records the offset each cycle, and the recorder puts OBC telemetry on the rig's time base using it.
