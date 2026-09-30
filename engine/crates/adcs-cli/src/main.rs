//! adcs -- the engine's command line.
//!
//!   adcs run <scenario> [--case F] [--fsw c|rust] [--seed N] [--out DIR] [--set k=v]... [--alg slot=id]... [--quiet]
//!   adcs params <scenario> [--case F] [--set k=v]... --out blob.bin     the adcs-fswcfg/1 blob (OILS / OBC upload)
//!   adcs size <case> [--knobs k.json] [--out DIR]                        demand survey + every actuator option sized (adcs-design)
//!   adcs parity <scenario> [--fsw A --against B] ...                  two flight-software targets, same loop, same bytes
//!   adcs results list [DIR] | show <run> | export <run> --out F.trinetra | import F.trinetra --out DIR
//!   adcs help [command]                                                   what each command does
//!
//! --fsw: c | rust (in-process) | obc-posix | obc-posix-rs (virtual OBC process) | qemu | qemu-rs
//! (virtual Cortex-M4 OBC in QEMU) | spawn:<cmd> | tcp:<host:port> (a real OBC); --realtime paces ticks to wall time.
//! --oils: soft OILS -- each command lands after the OBC's execution (exact QEMU instruction count x
//! --cpi / --obc-mhz) plus its bus time (--i2c-khz, --can-kbps), inside the control period (docs/SOFT_OILS.md).
//!
//! Scenario ids are matlab_sils/data/scenarios/*.json (or a path); the case defaults to
//! cases/<scenario.case>.csv. Overrides: `--set fsw.rw_bandwidth=0.5` edits the scenario,
//! `--set engine.duration_s=600` (also density_scale, orbit_step_s, zonal_max, igrf_nmax) the engine.
//! Owner: Agastya. Copyright (c) 2026 Agastya. All rights reserved.
mod args;
mod fly;
mod help;
mod results;

use std::process::ExitCode;

fn main() -> ExitCode {
    adcs_sim::fsio::install_crash_report("adcs");
    let argv: Vec<String> = std::env::args().skip(1).collect();
    let cmd = argv.first().map(String::as_str).unwrap_or("");
    let rest = if argv.is_empty() { &argv[..] } else { &argv[1..] };
    // help: `adcs`, `adcs help [command]`, `adcs --help`, `adcs <command> --help`
    if cmd.is_empty() || cmd == "--help" || cmd == "-h" || (cmd == "help" && rest.is_empty()) {
        print!("{}", help::overview());
        return if cmd.is_empty() { ExitCode::from(2) } else { ExitCode::SUCCESS };
    }
    if cmd == "help" || rest.iter().any(|a| a == "--help" || a == "-h") {
        let name = if cmd == "help" { rest[0].as_str() } else { cmd };
        return match help::command(name) {
            Some(h) => { print!("{h}"); ExitCode::SUCCESS }
            None => { eprintln!("error: no command {name}\n\n{}", help::overview()); ExitCode::from(2) }
        };
    }
    let r = match cmd {
        "results" => results::main(rest),
        "run" | "params" | "parity" | "size" => match args::parse(cmd, rest) {
            Ok(a) => fly::main(&a),
            Err(e) => { eprintln!("{e}"); return ExitCode::from(2); }
        },
        _ => { eprintln!("error: unknown command {cmd}\n\n{}", help::overview()); return ExitCode::from(2); }
    };
    match r { Ok(()) => ExitCode::SUCCESS, Err(e) => { eprintln!("error: {e}"); ExitCode::FAILURE } }
}
