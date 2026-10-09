//! `adcs design derive FILE...` | `adcs design call MODULE::FN`: the design's methods a Python tool asks for, so a tool
//! keeps no relation of its own (docs/S7_INVENTORY.md S7.15): a catalogue model's parameters from its datasheet
//! (tools/catalogue.py), and any method generated into the sizing by its name, through the translator's dispatcher
//! (the design loop's rules, tools/pipeline_design.py).
//! Owner: Agastya. Copyright (c) 2026 Agastya. All rights reserved.
use crate::cli::DesignCmd;
use adcs_sim::Error;

pub fn main(cmd: &DesignCmd) -> Result<(), Error> {
    match cmd {
        DesignCmd::Derive { files } => {
            for f in files {
                let c = adcs_sim::json::read(f)?;
                let d = adcs_design::catalogue::derive(&c).map_err(|e| Error::refused(format!("{}: {e}", f.display())))?;
                println!("{}", serde_json::to_string(&d).map_err(|e| Error::run(e.to_string()))?);
            }
        }
        DesignCmd::Call { name } => {
            let mut text = String::new();
            std::io::Read::read_to_string(&mut std::io::stdin(), &mut text).map_err(|e| Error::run(format!("stdin: {e}")))?;
            let x = text.split_whitespace().map(|w| w.parse::<f64>().map_err(|_| Error::refused(format!("{name}: {w:?} is not a number"))))
                .collect::<Result<Vec<f64>, Error>>()?;
            let out = adcs_design::gen::dispatch::call(name, &x)
                .ok_or_else(|| Error::refused(format!("{name}: no such method in the sizing, or not its {} input number(s)", x.len())))?;
            let mut s = String::with_capacity(out.len()*24);
            for v in out { s += &format!("{v:?}\n"); }
            print!("{s}");
        }
    }
    Ok(())
}
