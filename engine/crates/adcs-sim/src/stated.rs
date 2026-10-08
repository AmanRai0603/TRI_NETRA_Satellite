//! The design's stated values the engine reads (docs/S7_INVENTORY.md S7.11): `data/stated.json`, written from the
//! design's stated nodes (tools/design_build.py), one number per node. The engine names the node it needs and keeps no
//! copy of its value: the device defaults (act), the plant's constants (dyn), the axes a product flies when it states
//! none (gdn), the space weather, the orbit's step and the models' degrees a run takes when it states none (env).
//! Read like every other input, from the design database in use or the data folder (source.rs); a node the file does
//! not state is refused by name.
//! Owner: Agastya. Copyright (c) 2026 Agastya. All rights reserved.
use crate::error::Error;
use std::collections::BTreeMap;
use std::path::Path;

/// Where the stated values sit in the data folder.
pub const FILE: &str = "data/stated.json";

/// The stated values of the design, by node.
#[derive(Clone, Debug, Default)]
pub struct Stated { file: String, value: BTreeMap<String, f64> }

impl Stated {
    /// Read the stated values from the data folder `root` (or the design database in use).
    pub fn load(root: &Path) -> Result<Stated, Error> {
        let f = root.join(FILE);
        let v = crate::json::read(&f)?;
        if v.get("schema").and_then(|s| s.as_str()) != Some("adcs-stated/1") {
            return Err(Error::malformed(format!("{}: not an adcs-stated/1 file", f.display())));
        }
        let mut value = BTreeMap::new();
        for (k, x) in v.get("value").and_then(|o| o.as_object()).into_iter().flatten() {
            let n = x.as_f64().filter(|n| n.is_finite())
                .ok_or_else(|| Error::malformed(format!("{}: {k} is not a finite number", f.display())))?;
            value.insert(k.clone(), n);
        }
        Ok(Stated { file: f.display().to_string(), value })
    }
    /// The number a node states, refused by name when the design states none.
    pub fn get(&self, node: &str) -> Result<f64, Error> {
        self.value.get(node).copied()
            .ok_or_else(|| Error::refused(format!("the design states no value for {node} ({}), which the engine needs", self.file)))
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
