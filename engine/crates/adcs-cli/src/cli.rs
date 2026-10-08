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
    /// draw a run's figures (engine or MATLAB twin run folder) as SVG or PDF
    Figures(FiguresArgs),
    /// write a run's report: report.html (figures inline, prints to PDF) and report.pdf
    Report(ReportArgs),
    /// draw figures described as JSON (the schema: adcs-plot's crate documentation) as SVG or PDF
    #[command(after_help = "One figure object, a list of them, or {\"figures\": [...]}; each has panels (stacked or in a grid) with\n\
                            line, step, scatter, hist and hbar series, reference lines and notes. Several figures into one\n\
                            .svg are written as FILE_1.svg, FILE_2.svg, ...; into one .pdf as a page each.")]
    Plot(PlotArgs),
}

#[derive(clap::ValueEnum, Clone, Copy, Debug, PartialEq)]
pub enum Format { Svg, Pdf }

impl Format {
    pub fn ext(self) -> &'static str { match self { Format::Svg => "svg", Format::Pdf => "pdf" } }
}

#[derive(clap::Args, Debug)]
pub struct FiguresArgs {
    /// the run folder (manifest.json and channels.csv): <store>/results_engine/<id> or the twin's <store>/results/<id>
    pub run: PathBuf,
    /// where the figures go, <prefix>_<n>_<name>.<format>
    #[arg(long, value_name = "DIR")]
    pub out: PathBuf,
    /// svg or pdf
    #[arg(long, value_name = "F", value_enum, default_value = "svg")]
    pub format: Format,
    /// also the environment, ground track and mode figures
    #[arg(long)]
    pub full: bool,
    /// the start of every file name (default: the run folder's name)
    #[arg(long, value_name = "P")]
    pub prefix: Option<String>,
}

#[derive(clap::Args, Debug)]
pub struct ReportArgs {
    /// the run folder (manifest.json and channels.csv)
    pub run: PathBuf,
    /// where report.html and report.pdf go (default: the run folder)
    #[arg(long, value_name = "DIR")]
    pub out: Option<PathBuf>,
}

#[derive(clap::Args, Debug)]
pub struct PlotArgs {
    /// the figure description (JSON)
    pub spec: PathBuf,
    /// the file to write: .svg or .pdf
    #[arg(long, value_name = "FILE")]
    pub out: PathBuf,
}

/// What every command that builds a configuration takes.
#[derive(clap::Args, Debug)]
pub struct Scenario {
    /// an id from matlab_sils/data/scenarios, or a path to a .json file
    pub scenario: String,
    /// the case: a CSV file, or the id of a shipped case (default: the case the scenario names)
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
    #[arg(long, value_name = "MS", value_parser = non_negative)]
    pub latency_ms: Option<f64>,
    /// soft OILS: the OBC's clock [MHz]
    #[arg(long, value_name = "MHZ", value_parser = positive)]
    pub obc_mhz: Option<f64>,
    /// soft OILS: cycles per instruction
    #[arg(long, value_name = "CPI", value_parser = positive)]
    pub cpi: Option<f64>,
    /// soft OILS: the I2C bus clock [kHz]
    #[arg(long, value_name = "KHZ", value_parser = positive)]
    pub i2c_khz: Option<f64>,
    /// soft OILS: the CAN bus rate [kbit/s]
    #[arg(long, value_name = "KBPS", value_parser = positive)]
    pub can_kbps: Option<f64>,
    /// soft OILS: the SPI bus clock [kHz]
    #[arg(long, value_name = "KHZ", value_parser = positive)]
    pub spi_khz: Option<f64>,
    /// soft OILS: the worst cycles per instruction the deadline is judged at (default 2.0)
    #[arg(long, value_name = "CPI", value_parser = positive)]
    pub cpi_max: Option<f64>,
    /// soft OILS: interrupt time that may preempt one step [us] (default 50)
    #[arg(long, value_name = "US", value_parser = non_negative)]
    pub isr_us: Option<f64>,
    /// soft OILS: the share of the control period the command must land within (default 0.5)
    #[arg(long, value_name = "FRAC", value_parser = fraction)]
    pub deadline_frac: Option<f64>,
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
    /// ask the results index one read-only SQL question (tables runs and metrics)
    Query {
        /// one SELECT, e.g. "SELECT scenario, value FROM metrics JOIN runs USING (folder) WHERE id LIKE 'ape%'"
        #[arg(long, value_name = "SELECT")]
        sql: String,
        /// the folder (default: <store>/results_engine)
        dir: Option<PathBuf>,
    },
    /// every stored run another engine or other inputs flew, and why; exit 1 when any
    Stale {
        /// the folder (default: <store>/results_engine)
        dir: Option<PathBuf>,
    },
    /// a stored fine-pointing run's pointing error budget (pnt's gp_0 to gp_5: its terms, total, room and verdict), as JSON
    Budget {
        run: PathBuf,
        /// the run's metric of the flown APE across the boresight, p99.73
        #[arg(long, value_name = "ID")]
        flown: Option<String>,
        /// the run's metric of the AKE across the boresight, p99.73
        #[arg(long, value_name = "ID")]
        knowledge: Option<String>,
        /// the run's metric of the rotors' jitter [arcsec]
        #[arg(long, value_name = "ID")]
        jitter: Option<String>,
    },
    /// fly a stored run again from the inputs it kept, and show what changed
    Refly {
        run: PathBuf,
        /// where the new run goes (default: <store>/refly/<run folder name>)
        #[arg(long, value_name = "DIR")]
        out: Option<PathBuf>,
        /// fly it on this flight software instead of the one it flew (c | rust | obc-posix | qemu | ...)
        #[arg(long, value_name = "T")]
        fsw: Option<String>,
    },
}

fn key_value(s: &str) -> Result<(String, String), String> {
    s.split_once('=').filter(|(k, _)| !k.is_empty()).map(|(k, v)| (k.to_string(), v.to_string()))
        .ok_or_else(|| format!("{s:?} is not key=value"))
}

fn finite(s: &str) -> Result<f64, String> {
    match s.parse::<f64>() { Ok(x) if x.is_finite() => Ok(x), _ => Err(format!("{s:?} is not a finite number")) }
}

fn fraction(s: &str) -> Result<f64, String> {
    finite(s).and_then(|x| if x > 0.0 && x <= 1.0 { Ok(x) } else { Err(format!("{s:?} must be above 0 and at most 1")) })
}

fn positive(s: &str) -> Result<f64, String> {
    finite(s).and_then(|x| if x > 0.0 { Ok(x) } else { Err(format!("{s:?} must be above 0")) })
}

fn non_negative(s: &str) -> Result<f64, String> {
    finite(s).and_then(|x| if x >= 0.0 { Ok(x) } else { Err(format!("{s:?} must be 0 or more")) })
}

fn fsw(s: &str) -> Result<Impl, String> { s.parse().map_err(|e: adcs_fsw_abi::FswError| e.to_string()) }

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
            let any = r.oils || r.latency_ms.is_some() || r.obc_mhz.is_some() || r.cpi.is_some() || r.i2c_khz.is_some() || r.can_kbps.is_some()
                || r.spi_khz.is_some() || r.cpi_max.is_some() || r.isr_us.is_some() || r.deadline_frac.is_some();
            let oils = any.then(|| {
                let mut o = run::OilsModel::default();
                if let Some(x) = r.latency_ms { o.fixed_s = Some(x*1e-3); }
                if let Some(x) = r.obc_mhz { o.cpu_hz = x*1e6; }
                if let Some(x) = r.cpi { o.cpi = x; }
                if let Some(x) = r.i2c_khz { o.i2c_hz = x*1e3; }
                if let Some(x) = r.can_kbps { o.can_bps = x*1e3; }
                if let Some(x) = r.spi_khz { o.spi_hz = x*1e3; }
                if let Some(x) = r.cpi_max { o.cpi_max = x; }
                if let Some(x) = r.isr_us { o.isr_s = x*1e-6; }
                if let Some(x) = r.deadline_frac { o.deadline_frac = x; }
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
        Cmd::Results(_) | Cmd::Figures(_) | Cmd::Report(_) | Cmd::Plot(_) => return None,
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

    /// docs/commands.toml, read as TOML: the adcs commands it describes are the commands clap
    /// declares, and every option and subcommand its usage lines name exists.
    #[test]
    fn the_registry_describes_exactly_these_commands() {
        let reg: toml::Value = toml::from_str(&std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../docs/commands.toml")).unwrap()).unwrap();
        let cmd = Cli::command();
        let ours: std::collections::BTreeSet<String> = cmd.get_subcommands().map(|c| c.get_name().to_string()).filter(|n| n != "help").collect();
        let mut theirs = std::collections::BTreeSet::new();
        for c in reg["command"].as_array().unwrap().iter().filter(|c| c["tool"].as_str() == Some("adcs")) {
            let name = c["name"].as_str().unwrap();
            theirs.insert(name.to_string());
            let sc = cmd.find_subcommand(name).unwrap_or_else(|| panic!("the registry has adcs {name}; clap does not"));
            let usage = c["usage"].as_str().unwrap();
            assert!(usage.starts_with(&format!("adcs {name}")), "{name}: {usage}");
            // every --option in the usage is one clap knows, on the command or one of its subcommands
            let mut known: Vec<String> = sc.get_arguments().filter_map(|a| a.get_long().map(String::from)).collect();
            for sub in sc.get_subcommands() {
                known.extend(sub.get_arguments().filter_map(|a| a.get_long().map(String::from)));
                assert!(usage.contains(sub.get_name()), "adcs {name} {}: not in the registry's usage", sub.get_name());
            }
            for word in usage.split(|c: char| c.is_whitespace() || c == '[' || c == ']') {
                if let Some(opt) = word.strip_prefix("--") {
                    assert!(known.iter().any(|k| k == opt), "adcs {name}: the registry names --{opt}, which clap does not declare");
                }
            }
        }
        assert_eq!(ours, theirs, "clap's commands and the registry's differ");
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
                    "adcs results export r", "adcs run s --fsw nowhere", "adcs figures r", "adcs figures r --out d --format png", "adcs plot s.json", "adcs run s --cpi 0", "adcs run s --obc-mhz -5", "adcs run s --latency-ms -1"] {
            assert!(parse(bad).is_err(), "accepted: {bad}");
        }
    }
}
