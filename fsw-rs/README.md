# adcs-fsw — TRI-NETRA ADCS flight software in Rust

Owner: Agastya. The Rust twin of `fsw/` (C99). Both implement `fsw/pseudocode`
over the same parameter blob (`adcs-fswcfg/1`, generated from
`fsw/params/params.toml` by `tools/gen_fsw_params.py`) and the same byte-level
HAL (`fsw/include/adcs_hal.h`, here the `Hal` trait).

| module | pseudocode | C twin |
|---|---|---|
| `math` | 01 | `adcs_math.c` |
| `env` | 02 | `adcs_env.c` |
| `est` | 03 | `adcs_est.c` |
| `guid` | 04 | `adcs_guid.c` |
| `ctl` | 05–06 | `adcs_ctl.c` |
| `alloc` | 07 | `adcs_alloc.c` |
| `fsw` (shell), `fsw::modes`, `fsw::fdir` | 08, 07 (rotor FDIR) | `adcs_fsw.c`, `adcs_modes.c`, `adcs_fdir.c` (state in `adcs_fsw_int.h`) |
| `devices`, `drv`, `hal` | 09 | `adcs_devices.h`, `adcs_drv.c`, `adcs_hal.h` |
| `params` | generated | `adcs_params.c` |
| `alg` (and `ALG_ID`) | written from the design by `tools/flight_build.py` | `fsw/alg` (and `adcs_alg_id.h`) |

* `no_std`, no heap, no panics on the flight path; `libm` on targets, the
  platform libm on host builds (then bit-identical to the C build — the test
  `abi_detumble` pins the PWM words both builds produce).
* Feature `cabi` exports `adcs_fsw_init/step/command/peek/build_id` (+
  `adcs_fsw_debug`) over `extern "C" adcs_hal_*`: the static library is a
  drop-in for `libadcs_fsw.a` in the SILS engine, OILS or on the OBC.

```
cargo test --release                                   # host tests (same vectors as fsw/tests)
cargo build --release --no-default-features --features cabi --target thumbv7em-none-eabihf
```
