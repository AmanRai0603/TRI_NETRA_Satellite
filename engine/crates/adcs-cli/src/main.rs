//! adcs -- the engine's command line.
//!
//!   adcs run <scenario> [--case F] [--fsw c|rust] [--seed N] [--out DIR] [--set k=v]... [--alg slot=id]... [--quiet]
//!   adcs params <scenario> [--case F] [--set k=v]... --out blob.bin     the adcs-fswcfg/1 blob (OILS / OBC upload)
//!   adcs size <case> [--knobs k.json] [--out DIR]                        demand survey + every actuator option sized (adcs-design)
//!   adcs parity <scenario> [--fsw A --against B] ...                  two flight-software targets, same loop, same bytes
//!   adcs results list [DIR] | show <run> | export <run> --out F.trinetra | import F.trinetra --out DIR
//!   adcs help [command]                                                   what each command does (clap, from cli.rs)
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
mod cli;
mod fly;
mod results;

use clap::Parser;
use std::process::ExitCode;

fn main() -> ExitCode {
    adcs_sim::fsio::install_crash_report("adcs");
    // help and malformed command lines: clap prints them (exit 0 for help, 2 for an error)
    let cli = cli::Cli::parse();
    let r = match &cli.cmd {
        cli::Cmd::Results(c) => results::main(c),
        other => fly::main(&cli::args(other).expect("a flying command")),
    };
    // a refused input exits 2 (as a malformed command line does), a failure 1
    match r { Ok(()) => ExitCode::SUCCESS, Err(e) => { eprintln!("error: {e}"); ExitCode::from(e.exit_code()) } }
}
