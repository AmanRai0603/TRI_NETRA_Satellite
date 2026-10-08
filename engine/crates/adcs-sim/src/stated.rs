//! The design's stated values the engine reads (docs/S7_INVENTORY.md S7.11, S7.13): `data/stated.json`, written from the
//! design's stated nodes (tools/design_build.py), a number or a list of numbers per node. The engine names the node it needs
//! and keeps no copy of its value: the device defaults (act), the plant's constants (dyn), the axes a product flies when it
//! states none (gdn), the space weather, the orbit's step and the models' degrees a run takes when it states none (env);
//! since S7.13 the flight software's constant parameters and the values a run takes when its scenario states none (fsw's
//! fsw_param_* and fsw_tune_*), and the run's defaults: its models' switches (env), its length and record step (vv) and
//! its start (dyn).
//! Read like every other input, from the design database in use or the data folder (source.rs); a node the file does
//! not state is refused by name.
//! Owner: Agastya. Copyright (c) 2026 Agastya. All rights reserved.
use crate::error::Error;
use std::collections::BTreeMap;
use std::path::Path;

/// Where the stated values sit in the data folder.
pub const FILE: &str = "data/stated.json";

/// The schema the file states (design_build.py STATED_SCHEMA): a node's value a number, or a list of numbers.
pub const SCHEMA: &str = "adcs-stated/2";

/// The stated values of the design, by node: a number, or a list of numbers.
#[derive(Clone, Debug, Default)]
pub struct Stated { file: String, value: BTreeMap<String, Vec<f64>>, list: BTreeMap<String, bool> }

impl Stated {
    /// Read the stated values from the data folder `root` (or the design database in use).
    pub fn load(root: &Path) -> Result<Stated, Error> {
        let f = root.join(FILE);
        let v = crate::json::read(&f)?;
        if v.get("schema").and_then(|s| s.as_str()) != Some(SCHEMA) {
            return Err(Error::malformed(format!("{}: not an {SCHEMA} file", f.display())));
        }
        let (mut value, mut list) = (BTreeMap::new(), BTreeMap::new());
        for (k, x) in v.get("value").and_then(|o| o.as_object()).into_iter().flatten() {
            let bad = || Error::malformed(format!("{}: {k} is not a finite number or a list of them", f.display()));
            let num = |x: &serde_json::Value| x.as_f64().filter(|n| n.is_finite());
            let xs = match x {
                serde_json::Value::Array(a) if !a.is_empty() => a.iter().map(|y| num(y).ok_or_else(bad)).collect::<Result<Vec<f64>, Error>>()?,
                y => vec![num(y).ok_or_else(bad)?],
            };
            list.insert(k.clone(), x.is_array());
            value.insert(k.clone(), xs);
        }
        Ok(Stated { file: f.display().to_string(), value, list })
    }
    /// The number a node states, refused by name when the design states none (or states a list).
    pub fn get(&self, node: &str) -> Result<f64, Error> {
        match (self.value.get(node), self.list.get(node)) {
            (Some(x), Some(false)) => Ok(x[0]),
            (Some(_), _) => Err(Error::refused(format!("{node} states a list ({}), where the engine needs a number", self.file))),
            _ => Err(Error::refused(format!("the design states no value for {node} ({}), which the engine needs", self.file))),
        }
    }
    /// The n numbers a node states as a list, refused by name when it states none or another length.
    pub fn list<const N: usize>(&self, node: &str) -> Result<[f64; N], Error> {
        match (self.value.get(node), self.list.get(node)) {
            (Some(x), Some(true)) if x.len() == N => { let mut a = [0.0; N]; a.copy_from_slice(x); Ok(a) }
            (Some(x), _) => Err(Error::refused(format!("{node} states {} number(s) ({}), where the engine needs a list of {N}", x.len(), self.file))),
            _ => Err(Error::refused(format!("the design states no value for {node} ({}), which the engine needs", self.file))),
        }
    }
    /// A yes or no a node states as 1 or 0.
    pub fn flag(&self, node: &str) -> Result<bool, Error> {
        let x = self.get(node)?;
        if x != 0.0 && x != 1.0 { return Err(Error::refused(format!("{node} = {x} ({}): 1 (yes) or 0 (no)", self.file))); }
        Ok(x == 1.0)
    }
    /// The three numbers `<prefix>_x`, `_y` and `_z` state.
    pub fn v3(&self, prefix: &str) -> Result<[f64; 3], Error> {
        Ok([self.get(&format!("{prefix}_x"))?, self.get(&format!("{prefix}_y"))?, self.get(&format!("{prefix}_z"))?])
    }
    /// A whole number a node states, within lo..=hi.
    pub fn whole(&self, node: &str, lo: usize, hi: usize) -> Result<usize, Error> {
        let x = self.get(node)?;
        if x.fract() != 0.0 || x < lo as f64 || x > hi as f64 {
            return Err(Error::refused(format!("{node} = {x} ({}): a whole number from {lo} to {hi}", self.file)));
        }
        Ok(x as usize)
    }
}
