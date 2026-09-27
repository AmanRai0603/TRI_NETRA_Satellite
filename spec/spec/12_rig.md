
---

## 12. The rig — PIL, OILS, HILS

### 12.1 Four rungs, one scenario

| Rung | Flight code runs on | Plant runs on | Link | Hardware in the loop | Timing | Who |
|---|---|---|---|---|---|---|
| SILS | linked into the loop engine process | loop engine, on a server | in-process | none | faster than real time | clients and engineers |
| PIL | the target processor (Raspberry Pi, PolarFire SoC) | loop engine on a PC | serial or UDP carrying the loop contract | the processor | soft real time | engineers |
| OILS | the flight OBC (OBC-in-the-loop) | the rig host, real time | the OBC's flight connectors, answered by the interface emulation unit | the OBC, and the GNSS receiver's output emulated for orbit determination | hard real time | engineers, billable campaigns |
| HILS | the flight OBC | the rig host, real time | real sensors and actuators | the full ADCS unit in the Helmholtz cage on the air bearing, with the stimulators | hard real time | qualification of each delivered unit |

The scenario file is the same at every rung. The device map (`rig/device_maps/*.toml`) is the only thing that says, device by device, whether a port is answered by an emulator or by the part. That lets a campaign move from OILS to HILS one device at a time.

### 12.2 Hardware roles

The constraint is to buy standard hardware and build the software, and it holds here.

| Role | What it is | Built or bought |
|---|---|---|
| Rig host | an x86-64 PC running Linux with PREEMPT_RT (in mainline since 6.12), running `adcs-rig` | bought |
| Interface emulation unit (IEU) | an FPGA or MCU board on the flight connectors. It answers I2C and SPI as a target, emulates UART, RS-422 and RS-485, captures PWM and speaks CAN, at electrical level, from `devices/*.toml`. The PolarFire SoC or Red Pitaya boards already in hand are candidates; the choice is D14. | board bought, gateware and firmware built |
| Environment simulator | the Helmholtz cage and its bipolar supplies, the sun simulator and shutter, the star stimulator (microdisplay and collimator), the GNSS RF simulator, the air-bearing platform with its balancer | bought; `adcs-rig` drives each |
| Metrology | the reference fluxgate in the cage, optical truth attitude of the platform, current and voltage probes, an oscilloscope or DAQ for timing | bought |
| Actuator test stands | a force dynamometer for wheel torque, imbalance and micro-vibration (Nadeem 2022), a magnetometer array or rotation rig for coil dipole and residual dipole (Springmann and Cutler 2010), a torque stand for the rings, and a thrust stand with impulse-bit resolution for the thrusters. Each is used only when the product carries that family. | bought |
| Power | programmable supplies for the OBC and the unit, with brownout injection | bought |

What each role must achieve is not written here. It is two sets of rows in the tree (§5.3). **What the lab can do** is layer 1's facility rows (OILS rig, magnetic field simulator, air-bearing platform, optical and RF stimulators, actuator test stands, rig safety and power), each read from the lab file by the `lab` supplier (§8.2), so the file stays the one place a measured capability is written. **What this case needs** is layer 2's "OILS rig needs" and "HILS rig needs" rows, derived from the case: the field range from the orbit's strongest field, the bearing torque allowed from the actuator authority and the disturbance torque, the truth accuracy from the knowledge requirement, and so on. `adcs rig fit` compares the two (§12.10). The figures a facility is specified by follow the literature: Helmholtz cages by field range per axis, magnitude and direction error and the homogeneous volume (da Silva et al. 2019); Sun simulators by the ASTM E927 classes; star stimulators by angular step, faintest magnitude and frame rate (Zhao et al. 2024); air bearings by residual torque, tilt range and balance (Schwartz et al. 2003); flat-sats by the interfaces emulated, missed ticks and isolation (Colagrossi et al. 2023).

### 12.3 Real-time design

- PREEMPT_RT kernel, `isolcpus` for the rig cores, `SCHED_FIFO` for the loop thread, `mlockall`.
- No allocation, no locks and no system calls in the loop except the UDP send and receive.
- The rig's `plant_step_s` is the dynamics step at OILS and HILS, replacing the scenario's (§9.3).
- Every cycle measures its own latency. A cycle that misses its deadline is counted. The run's policy (from the device map) is abort, or continue with the count recorded against the layer-1 row "deadline misses allowed per hour".
- Acceptance: `cyclictest`-style latency histograms are recorded for the rig host at commissioning. The run manifest carries the rig's measured worst-case latency, so a HILS result states the timing it was made under.

### 12.4 `adcs-rig`

- `rt/`: the real-time loop, running `adcs-sim-core` at `plant_step_s`.
- `transport/`: in-process, UDP to the IEU, serial, SocketCAN.
- `emulate/`: the same device emulators as `adcs-sim`, reused, not copied.
- `envsim/`: cage (SCPI over LAN to the supplies; the commanded field is `B_lab = R_lab←ECI · B_ECI(r, t)` from the real-time orbit), sun shutter, star stimulator renderer (stars from the catalogue the tracker carries, at the platform's truth attitude), GNSS simulator control.
- `metrology/`: truth-attitude ingest, reference magnetometer ingest.
- `record/`: the recorder, on the rig's time base.
- `safety/`: software limits that stop a run.

Every driver has a mock backend. The mocks are complete enough that a whole OILS or HILS campaign runs in CI with no hardware attached.

### 12.5 OILS

The flight OBC's own board support package implements `adcs_hal.h`, and the OBC runs the flight build under test. The rig host runs the plant and the device emulators. The IEU answers every sensor port with bytes from the plant and captures every actuator command on the flight connector: PWM duty to the coils, CAN to the ring modules. OILS includes orbit determination: the GNSS receiver's output is emulated from the plant's truth orbit at protocol level.

The pass criteria are the scenario's. The rung's own test is the parity ledger line against SILS for the same campaign.

### 12.6 HILS and the lab twin

In HILS the unit's real sensors and actuators are in the loop, and the environment simulator makes the environment. The cage follows the real-time orbit's field in the lab frame; the sun simulator and star stimulator stand in for the sky; the air bearing gives real rotational dynamics.

**An air bearing is not orbit.** Its residual gravity torque (m·g·|r|, the weight times the centre-of-mass offset, Schwartz et al. 2003; about 1.5 × 10⁻⁴ N·m for a 15 kg platform at a 1 µm offset) is larger than the orbital disturbances a CubeSat must reject, and its tilt is limited. So HILS is never compared with the orbit run. It is compared with the **lab twin**: campaign type `labtwin`, which runs the same scenario with the plant replaced by the lab from `rig/labs/*.toml`, using the platform inertia, the bearing's residual torque, the cage's measured field error and the serials actually fitted. `rig/labs/hils_bay1.toml` is the real bay; every value is `nan` until the facility measures it, so its lab twin is refused until then. `rig/labs/syn_lab.toml` is a synthetic lab with stated round numbers, so the lab-twin machinery runs from P4. The parity ledger carries both lines, SILS against the lab twin against HILS, so the orbit prediction and the lab measurement meet through a model of the lab rather than being compared directly.

### 12.7 Fault injection

The fault kinds in `campaigns/ring_freeze_fault.toml` and the device map's `[[fault_point]]` table are the same names at every rung:

- **SILS:** the emulator or the plant injects them.
- **OILS:** the IEU does: stop answering, corrupt a CRC, NACK, hold a value.
- **HILS:** relays and the programmable supply do: open a coil, brown out the OBC.

The fault-injection controls are internal only. A client sees the faults a campaign injected, never the console.

### 12.8 Safety

Cage current, bearing tilt and supply voltage have **hardware** interlocks and an emergency stop that do not depend on this software. The software limits in `safety/` stop a run earlier and more gently, and never replace the hardware. A person signs the rig's safety check before the first powered test on each configuration (H-rig, §17.2). The builder writes the checklist and does not sign it.

### 12.9 What needs hardware, and what the builder does instead

The builder writes every driver against its device's interface document and a mock backend, and proves the campaign runs end to end against mocks. Bring-up on the real device is `docs/RIG_BRINGUP.md`: a checklist per device of what a person does, measures and records. It is marked not done until a person has done it.

### 12.10 Rig fitness: `adcs rig fit`

**In one line:** before a rig campaign is armed, `adcs rig fit <case> --lab <file> [--product <p>] [--rung oils|hils]` compares what this case needs from the rig with what the lab has measured, row by row, and refuses by name whatever the lab cannot meet or has not measured.

The two sides are in different layers, and a row never reads across layers (C04), so the comparison is not a tree row. It is a read-only report over two sets of rows the tree already computes: layer 2's needs (`v3` OILS rig needs, `v4` HILS rig needs, for this case and product) and layer 1's capabilities (the facility rows: measured ones from the lab file, and the two policy rows marked declared).

| Need (layer 2) | Capability (layer 1) | Fits when |
|---|---|---|
| Real-time plant step needed | Real-time plant step (declared) | the rig's step is at most the need |
| Flight ports to answer | Interface emulation channels | the channels are at least the ports |
| Rig loop latency allowed | Rig loop latency, worst case measured | the measured latency is at most the allowance |
| Emulator link rate needed | Rig host to IEU link rate | the link rate is at least the need |
| Real-time run length | OILS rig hours per week (declared) | one run fits within a week's rig hours; a campaign's total is the booking's, not the fit's |
| Cage field range needed | Helmholtz cage field range | the range is at least the need |
| Cage field accuracy needed | Cage field error against command | the error is at most the need |
| Cage field slew rate needed | Cage field range over coil time constant | the slew the coils can make is at least the need |
| Bearing residual torque allowed | Air-bearing residual torque | the residual is at most the allowance |
| Bearing balance offset allowed | Balance residual centre-of-mass offset | the offset is at most the allowance |
| Bearing tilt range needed | Bearing tilt range | the range is at least the need |
| the unit's mass (`m`) | Platform payload capacity | the mass is at most the capacity |
| Platform inertia to match | Platform inertia | reported: the lab twin flies the sum, so no single comparison decides |
| Truth attitude accuracy needed | Truth attitude accuracy | the lab's accuracy is at most the need |
| Sun simulator irradiance needed | Sun simulator irradiance | the irradiance is at least the need |
| Sun simulator collimation needed | Sun simulator collimation half-angle | the half-angle is at most the need |
| Star stimulator error allowed | Star stimulator angular step | the step is at most the allowance |
| Star stimulator frame rate needed | Star stimulator frame rate | the rate is at least the need |
| Coil dipole to measure (`mtq`) | Coil dipole measurement resolution | the resolution is below the value to measure |
| Wheel torque and disturbance to measure (`rw`) | Dynamometer torque resolution | the resolution is below the value to measure; the dynamometer's bandwidth and the imbalance resolution are reported beside it |
| Ring torque to measure (`fmr`) | Ring torque measurement resolution | the resolution is below the value |
| Thrust to measure (`rcs`) | Thrust stand resolution; impulse bit resolution | the thrust resolution is below the thrust; the impulse bit resolution is reported |

- **One verdict per line:** fits, short (with both values and units), reported (a value shown with no comparison, where none decides), NotMeasured (the lab value is `nan`), NotStated (the case leaves a key the need reads blank), or not applicable (the product has no part of that family). An OILS fit reads the first five lines; a HILS fit reads every line, since HILS also runs on the rig host and the IEU.
- **No margins are invented.** "Fits" is the plain comparison. A margin a person wants on a line is a request on the need row, which then carries it.
- **It gates, and it is not evidence.** `adcs rig arm` refuses a campaign whose fit has a short, NotMeasured or NotStated line, naming each, as `inertial_hold_labtwin` is refused on `hils_bay1`'s `nan` fields. The fit report is filed with the campaign's manifest. It never supplies a row and never changes the lab file: a measured capability reaches the lab file through a person's bring-up record (`docs/RIG_BRINGUP.md`).
- **Built with the rig:** the OILS lines in P6, the HILS lines in P7. `syn_lab.toml` has a value in every field the fit reads, so the machinery runs end to end from the start, and whether it fits a case is recorded, not expected. The real bay answers NotMeasured on every line until it is measured, which is the honest state.

