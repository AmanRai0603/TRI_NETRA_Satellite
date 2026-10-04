//! One flight, as every front end flies it: the command line (`adcs run`), the app and
//! `adcs results refly` call this, so a run means the same thing wherever it is started.
//!
//! case for the scenario -> configuration -> the closed loop -> derived channels -> metrics and
//! verdicts -> the record (channels.csv, manifest.json, the inputs kept) -> the store's retention.
//! Owner: Agastya. Copyright (c) 2026 Agastya. All rights reserved.
use crate::config::{check_id, Config};
use crate::error::Error;
use crate::metrics::{self, Derived};
use crate::run::{self, Opts, Record};
use serde_json::Value;
use std::path::{Path, PathBuf};

/// One flown run and everything judged from it.
pub struct Flight { pub record: Record, pub derived: Derived, pub metrics: Vec<Value>, pub retained: (Vec<PathBuf>, u64) }

/// The case file a scenario flies: the one given, else the one the scenario names
/// (data/scenarios/<id>.json -> cases/<case>.csv). Refused by name when there is neither.
pub fn case_file(root: &Path, scenario: &str, case: Option<&str>) -> Result<PathBuf, Error> {
    if let Some(c) = case.filter(|c| !c.is_empty()) {
        // an id names a shipped case; anything else is a file
        if !c.contains(['/', '\\', '.']) {
            check_id("case", c)?;
            let f = root.join("cases").join(format!("{c}.csv"));
            if !crate::source::is_file(&f) { return Err(Error::refused(format!("no case {c}: {} does not exist", f.display()))); }
            return Ok(f);
        }
        return Ok(PathBuf::from(c));
    }
    if !scenario.ends_with(".json") { check_id("scenario", scenario)?; }
    let sp = if scenario.ends_with(".json") { PathBuf::from(scenario) } else { root.join("data/scenarios").join(format!("{scenario}.json")) };
    if !crate::source::is_file(&sp) {
        return Err(Error::refused(format!("no scenario {scenario}: {} does not exist (the scenarios are data/scenarios/*.json)", sp.display())));
    }
    let s: Value = serde_json::from_str(&crate::source::read_to_string(&sp)?)
        .map_err(|e| Error::malformed(format!("{}: {e}", sp.display())))?;
    let id = s["case"].as_str().ok_or_else(|| Error::refused(format!("{} names no case: give --case F", sp.display())))?;
    check_id("case", id)?;
    Ok(root.join("cases").join(format!("{id}.csv")))
}

/// Fly `c` and write its record into `out`. When `out` is inside the store, the store's retention
/// is applied after the write (the runs it thinned are returned).
pub fn fly(c: &Config, o: &Opts, out: &Path) -> Result<Flight, Error> {
    let record = run::run(c, o)?;
    let derived = metrics::derive(c, &record);
    let mut metrics = metrics::evaluate(c, &record, &derived);
    // a soft-OILS run is also judged on its timing: no overrun, the worst case inside the deadline
    if let Some(o) = &record.oils { metrics.extend(o.metrics(c.dt)); }
    crate::rec::write(out, c, &record, &derived, &metrics)?;
    let store = crate::store_root();
    let inside = std::fs::canonicalize(out).ok().zip(std::fs::canonicalize(&store).ok()).is_some_and(|(p, s)| p.starts_with(s));
    let retained = if inside { crate::store::apply_retention(&store)? } else { (vec![], 0) };
    Ok(Flight { record, derived, metrics, retained })
}
