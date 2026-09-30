//! What each command does, how it is called, and what its options mean: `adcs help`,
//! `adcs help <command>` and `adcs <command> --help` print these. The same commands are
//! described, with their steps and files, in docs/commands.toml.
//! Owner: Agastya. Copyright (c) 2026 Agastya. All rights reserved.

/// (command, one line, usage, the options: (name, what it means, continuation lines))
type Opt = (&'static str, &'static str);
pub const COMMANDS: [(&str, &str, &str, &[Opt]); 5] = [
    ("run", "fly one scenario on one case and judge it against the case's requirements",
     "adcs run <scenario> [--case F] [--fsw T] [--seed N] [--out DIR] [--set k=v]... [--alg slot=id]... [--realtime] [--oils ...] [--quiet]",
     &[("<scenario>", "an id from matlab_sils/data/scenarios, or a path to a .json file"),
       ("--case F", "the case CSV (default: cases/<the scenario's case>.csv)"),
       ("--fsw T", "where the flight software runs: c | rust (in-process); obc-posix | obc-posix-rs (a separate"),
       ("", "process); qemu | qemu-rs (Cortex-M4 firmware in QEMU); spawn:<cmd> | tcp:<host:port> (a real OBC)"),
       ("--seed N", "the sensor-noise seed (default 1)"),
       ("--out DIR", "where the run is written (default: store/results_engine/<scenario>)"),
       ("--set k=v", "change the scenario (fsw.rw_bandwidth=0.5) or the engine (engine.duration_s=600, engine.kp=3);"),
       ("", "a value keeps the type it replaces, and a setting out of range is refused by name"),
       ("--alg slot=id", "fly another algorithm in a slot (--alg pointing=lqr)"),
       ("--realtime", "pace the loop to wall time (for a real OBC)"),
       ("--oils", "soft OILS: each command lands after the OBC's execution time (--obc-mhz, --cpi) and its bus"),
       ("", "time (--i2c-khz, --can-kbps); --latency-ms fixes the latency instead"),
       ("--quiet", "print only the verdicts")]),
    ("params", "write the flight software's parameter blob (adcs-fswcfg/1) for a scenario",
     "adcs params <scenario> [--case F] [--set k=v]... --out blob.bin",
     &[("--out FILE", "the blob an OBC boots from: every parameter in table order, with its CRC-32"),
       ("--case, --set", "as for `adcs run`: the blob is built as a run is")]),
    ("parity", "fly one scenario with two flight-software targets and compare them step by step",
     "adcs parity <scenario> [--fsw A --against B] [--case F] [--set k=v]...",
     &[("(default)", "C against Rust, in-process: bit-identical trajectories are the expected answer"),
       ("--fsw A --against B", "compare any two targets (c against qemu, say)")]),
    ("size", "survey a case's demand, then size every actuator option to it",
     "adcs size <case> [--knobs k.json] [--out DIR]",
     &[("<case>", "an id from matlab_sils/cases (ais_3u)"),
       ("--knobs F", "the sizing knobs (authority scales, margins, pump type, star tracker) as JSON"),
       ("--out DIR", "where the sized parts and products go (default: store/design/<case>/sized)")]),
    ("results", "list, show, export or import runs",
     "adcs results list [DIR] | show <run> | export <run> --out F.trinetra | import F.trinetra --out DIR",
     &[("list [DIR]", "every run under DIR (default: store/results_engine), one line each"),
       ("show <run>", "a run's provenance (what it flew, when, on which engine) and its requirement metrics"),
       ("export", "the run as one .trinetra file, a zip any unzip tool opens"),
       ("import", "a .trinetra file back into a folder, each entry's name and checksum checked")]),
];

/// `adcs help`: every command, one line each.
pub fn overview() -> String {
    let mut s = String::from("adcs -- the TRI-NETRA ADCS engine\n\nusage: adcs <command> ...\n\n");
    for (c, line, _, _) in COMMANDS { s += &format!("  {c:<9} {line}\n"); }
    s + "\n`adcs help <command>` (or `adcs <command> --help`) says more. The data folder is\nmatlab_sils (or $ADCS_ROOT); crash reports go to ~/.trinetra/log (or $TRINETRA_LOG).\n"
}

/// `adcs help <command>`, or None for a command that does not exist.
pub fn command(name: &str) -> Option<String> {
    COMMANDS.iter().find(|c| c.0 == name).map(|(c, line, usage, opts)| {
        let mut s = format!("adcs {c}: {line}\n\nusage: {usage}\n\n");
        for (o, what) in opts.iter() { s += &format!("  {o:<20} {what}\n"); }
        s
    })
}

/// The one-line usage of a command, for an error message.
pub fn usage(name: &str) -> String {
    COMMANDS.iter().find(|c| c.0 == name).map(|c| format!("usage: {}", c.2)).unwrap_or_else(overview)
}

#[cfg(test)]
mod t {
    #[test]
    fn every_command_has_help_and_a_usage_that_names_it() {
        for (c, _, u, opts) in super::COMMANDS {
            assert!(u.starts_with(&format!("adcs {c} ")), "{c}");
            assert!(!opts.is_empty(), "{c}");
            assert!(super::command(c).unwrap().contains(u));
        }
        assert!(super::command("bogus").is_none());
    }
}
