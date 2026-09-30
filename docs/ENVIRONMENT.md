# Environment variables

> **Answer first.** Nothing needs setting: every program finds its data, store and log by
> itself. These variables only move them or change a default. A variable set to nothing is
> the same as unset; a value that is not what the variable takes is refused by name.
>
> **Kind:** reference · **For:** anyone running the programs or the tools

## The engine (`adcs`) and the app

| variable | what it changes | default |
|---|---|---|
| `ADCS_ROOT` | the data folder (cases, scenarios, catalogue) | `matlab_sils` found from the working folder, or the kit beside the program |
| `TRINETRA_STORE` | where runs are kept | `~/.trinetra/store` for a kit, `matlab_sils/store` in a checkout |
| `TRINETRA_LOG` | where crash reports go | `~/.trinetra/log` |
| `ADCS_DE440` | the DE440 ephemeris kernel the orbit propagator reads | `pop/03_frames_time/ephemeris/data/de440s.bsp` in the data folder |
| `ADCS_REPO` | the repository root, for `adcs` commands that build or load the flight software | the ancestor of the working folder holding `fsw/` |
| `ADCS_SIZED_DIR` | a folder of sized products flown before the stored ones (set by the design loop for its current iteration) | none |

## The app (`TRI-NETRA ADCS`, `trinetra-app`)

| variable | what it changes | default |
|---|---|---|
| `TRINETRA_PORT` | the first local port tried (then the next 11) | 7788 |
| `TRINETRA_APP_IDLE_MINUTES` | minutes with no page open before the app ends, 1 to 10080 | 5 |
| `TRINETRA_NO_BROWSER` | any value: do not open a browser (for scripts and CI) | opens one |

## The Python tools (`tools/`)

| variable | what it changes | default |
|---|---|---|
| `TRINETRA_TRACE` | `0` stops the trace log `.trace/trinetra.log` | on |
| `SOURCE_DATE_EPOCH` | the date written into generated reports and packages (seconds since 1970), for reproducible builds | the last commit's date |

## Tests only

| variable | what it changes |
|---|---|
| `ADCS_PRINT_PWM` | `1` prints the actuator commands in the Rust flight-software tests |
