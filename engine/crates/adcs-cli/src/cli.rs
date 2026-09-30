//! The command line, declared once: every command, its options and what they mean. clap
//! parses it, refuses anything unknown or malformed by name (exit status 2), and prints
//! `adcs help`, `adcs help <command>` and `adcs <command> --help` from these declarations.
//! The same commands are described, with their steps and files, in docs/commands.toml.
//! Owner: Agastya. Copyright (c) 2026 Agastya. All rights reserved.
use crate::args::Args;
use adcs_fsw_abi::Impl;
use adcs_sim::run;
use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(name = "adcs", version, about = "adcs -- the TRI-NETRA ADCS engine",
          after_help = "The data folder is matlab_sils (or $ADCS_ROOT); runs go to the store (matlab_sils/store in a checkout,\n\
                        ~/.trinetra/store for a release, or $TRINETRA_STORE); crash reports to ~/.trinetra/log (or $TRINETRA_LOG).")]
pub struct Cli {
    #[command(subcommand)]
    pub cmd: Cmd,
}

#[derive(Subcommand, Debug)]
pub enum Cmd {
    /// fly one scenario on one case and judge it against the case's requirements
    Run(RunArgs),
    /// write the flight software's parameter blob (adcs-fswcfg/1) for a scenario
    Params(ParamsArgs),
    /// fly one scenario with two flight-software targets and compare them step by step
    #[command(after_help = "C against Rust, in-process, by default: bit-identical trajectories are the expected answer.\n\
                            Exit status 0 when the trajectories are bit-identical, 1 when they differ.")]
    Parity(ParityArgs),
    /// survey a case's demand, then size every actuator option to it
    Size(SizeArgs),
    /// list, show, keep, thin, export or import runs
    #[command(subcommand)]
    Results(ResultsCmd),
}

/// What every command that builds a configuration takes.
#[derive(clap::Args, Debug)]
pub struct Scenario {
    /// an id from matlab_sils/data/scenarios, or a path to a .json file
    pub scenario: String,
    /// the case CSV (default: cases/<the scenario's case>.csv)
    #[arg(long, value_name = "F")]
    pub case: Option<String>,
    /// change the scenario (fsw.rw_bandwidth=0.5) or the engine (engine.duration_s=600, engine.kp=3);
    /// a value keeps the type it replaces, and a setting out of range is refused by name
    #[arg(long = "set", value_name = "k=v", value_parser = key_value)]
    pub set: Vec<(String, String)>,
    /// fly another algorithm in a slot (--alg pointing=lqr)
    #[arg(long = "alg", value_name = "slot=id", value_parser = key_value)]
    pub alg: Vec<(String, String)>,
}

#[derive(clap::Args, Debug)]
pub struct RunArgs {
    #[command(flatten)]
    pub s: Scenario,
    /// where the flight software runs: c | rust (in-process); obc-posix | obc-posix-rs (a separate process);
    /// qemu | qemu-rs (Cortex-M4 firmware in QEMU); spawn:<cmd> | tcp:<host:port> (a real OBC)
    #[arg(long, value_name = "T", default_value = "c", value_parser = fsw)]
    pub fsw: Impl,
    /// the sensor-noise seed
    #[arg(long, value_name = "N", default_value_t = 1)]
    pub seed: u64,
    /// where the run is written (default: <store>/results_engine/<scenario>)
    #[arg(long, value_name = "DIR")]
    pub out: Option<PathBuf>,
    /// pace the loop to wall time (for a real OBC)
    #[arg(long)]
    pub realtime: bool,
    /// soft OILS: each command lands after the OBC's execution time and its bus time
    #[arg(long)]
    pub oils: bool,
    /// soft OILS with this fixed latency instead [ms]
    #[arg(long, value_name = "MS", value_parser = finite)]
    pub latency_ms: Option<f64>,
    /// soft OILS: the OBC's clock [MHz]
    #[arg(long, value_name = "MHZ", value_parser = finite)]
    pub obc_mhz: Option<f64>,
    /// soft OILS: cycles per instruction
    #[arg(long, value_name = "CPI", value_parser = finite)]
    pub cpi: Option<f64>,
    /// soft OILS: the I2C bus clock [kHz]
    #[arg(long, value_name = "KHZ", value_parser = finite)]
    pub i2c_khz: Option<f64>,
    /// soft OILS: the CAN bus rate [kbit/s]
    #[arg(long, value_name = "KBPS", value_parser = finite)]
    pub can_kbps: Option<f64>,
    /// print only the verdicts
    #[arg(long, short)]
    pub quiet: bool,
}

#[derive(clap::Args, Debug)]
pub struct ParamsArgs {
    #[command(flatten)]
    pub s: Scenario,
    /// the blob an OBC boots from: every parameter in table order, with its CRC-32
    #[arg(long, value_name = "FILE")]
    pub out: PathBuf,
}

#[derive(clap::Args, Debug)]
pub struct ParityArgs {
    #[command(flatten)]
    pub s: Scenario,
    /// the first target (with --against: compare any two, c against qemu, say)
    #[arg(long, value_name = "A", value_parser = fsw, requires = "against")]
    pub fsw: Option<Impl>,
    /// the second target
    #[arg(long, value_name = "B", value_parser = fsw, requires = "fsw")]
    pub against: Option<Impl>,
    /// the sensor-noise seed
    #[arg(long, value_name = "N", default_value_t = 1)]
    pub seed: u64,
}

#[derive(clap::Args, Debug)]
pub struct SizeArgs {
    /// an id from matlab_sils/cases (ais_3u)
    pub case: String,
    /// the sizing knobs (authority scales, margins, pump type, star tracker) as JSON
    #[arg(long, value_name = "F")]
    pub knobs: Option<String>,
    /// where the sized parts and products go (default: <store>/design/<case>/sized)
    #[arg(long, value_name = "DIR")]
    pub out: Option<PathBuf>,
}

#[derive(Subcommand, Debug)]
pub enum ResultsCmd {
    /// every run under DIR (default: <store>/results_engine), one line each
    List { dir: Option<PathBuf> },
    /// a run's provenance (what it flew, when, on which engine), its requirement metrics, and how to fly it again
    Show { run: PathBuf },
    /// keep a run whole: thin leaves it alone
    Pin { run: PathBuf },
    /// let thin treat a run like any other
    Unpin { run: PathBuf },
    /// remove the time series of unpinned runs older than DAYS; every manifest (verdicts, provenance) stays
    Thin {
        /// runs older than this many days
        #[arg(long, value_name = "DAYS")]
        older_than: u64,
        /// the folder (default: <store>/results_engine)
        dir: Option<PathBuf>,
        /// say what it would do, change nothing
        #[arg(long)]
        dry_run: bool,
    },
    /// the run as one .trinetra file, a zip any unzip tool opens
    Export {
        run: PathBuf,
        #[arg(long, value_name = "F.trinetra")]
        out: PathBuf,
    },
    /// a .trinetra file back into a folder, each entry's name and checksum checked
    Import {
        file: PathBuf,
        #[arg(long, value_name = "DIR")]
        out: PathBuf,
    },
}

fn key_value(s: &str) -> Result<(String, String), String> {
    s.split_once('=').filter(|(k, _)| !k.is_empty()).map(|(k, v)| (k.to_string(), v.to_string()))
        .ok_or_else(|| format!("{s:?} is not key=value"))
}

fn finite(s: &str) -> Result<f64, String> {
    match s.parse::<f64>() { Ok(x) if x.is_finite() => Ok(x), _ => Err(format!("{s:?} is not a finite number")) }
}

fn fsw(s: &str) -> Result<Impl, String> { s.parse() }

/// The engine's settings from the scenario options: --alg is a scenario setting too.
fn settings(s: &Scenario) -> Vec<(String, String)> {
    let mut set = s.set.clone();
    set.extend(s.alg.iter().map(|(k, v)| (format!("fsw.algorithms.{k}"), format!("\"{v}\""))));
    set
}

/// A flying command as the engine takes it.
pub fn args(cmd: &Cmd) -> Option<Args> {
    let base = |name: &str, s: &Scenario| Args {
        cmd: name.into(), scenario: s.scenario.clone(), case: s.case.clone(), fsw: Impl::C, fsw_b: None, seed: 1,
        out: None, set: settings(s), quiet: false, realtime: false, oils: None,
    };
    Some(match cmd {
        Cmd::Run(r) => {
            let any = r.oils || r.latency_ms.is_some() || r.obc_mhz.is_some() || r.cpi.is_some() || r.i2c_khz.is_some() || r.can_kbps.is_some();
            let oils = any.then(|| {
                let mut o = run::OilsModel::default();
                if let Some(x) = r.latency_ms { o.fixed_s = Some(x*1e-3); }
                if let Some(x) = r.obc_mhz { o.cpu_hz = x*1e6; }
                if let Some(x) = r.cpi { o.cpi = x; }
                if let Some(x) = r.i2c_khz { o.i2c_hz = x*1e3; }
                if let Some(x) = r.can_kbps { o.can_bps = x*1e3; }
                o
            });
            Args { fsw: r.fsw.clone(), seed: r.seed, out: r.out.clone(), quiet: r.quiet, realtime: r.realtime, oils, ..base("run", &r.s) }
        }
        Cmd::Params(p) => Args { out: Some(p.out.clone()), ..base("params", &p.s) },
        Cmd::Parity(p) => Args { fsw: p.fsw.clone().unwrap_or(Impl::C), fsw_b: p.against.clone(), seed: p.seed, ..base("parity", &p.s) },
        Cmd::Size(z) => Args {
            cmd: "size".into(), scenario: z.case.clone(), case: None, fsw: Impl::C, fsw_b: None, seed: 1, out: z.out.clone(),
            set: z.knobs.iter().map(|f| ("knobs".to_string(), f.clone())).collect(), quiet: false, realtime: false, oils: None,
        },
        Cmd::Results(_) => return None,
    })
}

#[cfg(test)]
mod t {
    use super::*;
    use clap::CommandFactory;

    fn parse(line: &str) -> Result<Cli, clap::Error> { Cli::try_parse_from(line.split_whitespace()) }

    #[test]
    fn the_declarations_are_consistent() { Cli::command().debug_assert(); }

    #[test]
    fn every_command_has_help() {
        for sc in Cli::command().get_subcommands() {
            assert!(sc.get_about().is_some(), "{} has no one-line help", sc.get_name());
        }
    }

    #[test]
    fn a_run_reads_as_before() {
        let a = args(&parse("adcs run detumble_ais --seed 3 --set engine.duration_s=300 --alg pointing=lqr --latency-ms 2 -q").unwrap().cmd).unwrap();
        assert_eq!((a.cmd.as_str(), a.scenario.as_str(), a.seed, a.quiet), ("run", "detumble_ais", 3, true));
        assert_eq!(a.set, [("engine.duration_s".to_string(), "300".to_string()), ("fsw.algorithms.pointing".into(), "\"lqr\"".into())]);
        assert_eq!(a.oils.unwrap().fixed_s, Some(2e-3));
    }

    #[test]
    fn what_a_command_cannot_take_is_refused() {
        for bad in ["adcs run", "adcs run s --seed x", "adcs run s --set novalue", "adcs run s --latency-ms nan", "adcs run s --bogus",
                    "adcs params s", "adcs parity s --fsw rust", "adcs parity s --out x", "adcs size", "adcs results thin",
                    "adcs results export r", "adcs run s --fsw nowhere"] {
            assert!(parse(bad).is_err(), "accepted: {bad}");
        }
    }
}
