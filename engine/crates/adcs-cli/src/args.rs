//! The options of `run`, `params`, `parity` and `size`, parsed; anything unknown is refused.
//! Owner: Agastya. Copyright (c) 2026 Agastya. All rights reserved.
use adcs_fsw_abi::Impl;
use adcs_sim::run;
use std::path::PathBuf;

pub struct Args {
    pub cmd: String, pub scenario: String, pub case: Option<String>, pub fsw: Impl, pub fsw_b: Option<Impl>, pub seed: u64,
    pub out: Option<PathBuf>, pub set: Vec<(String, String)>, pub quiet: bool, pub realtime: bool, pub oils: Option<run::OilsModel>,
}

pub fn parse(cmd: &str, argv: &[String]) -> Result<Args, String> {
    let usage = || crate::help::usage(cmd);
    let mut it = argv.iter().cloned();
    let scenario = it.next().filter(|s| !s.starts_with("--")).ok_or_else(usage)?;
    let mut a = Args { cmd: cmd.into(), scenario, case: None, fsw: Impl::C, fsw_b: None, seed: 1, out: None, set: vec![], quiet: false, realtime: false, oils: None };
    while let Some(k) = it.next() {
        let mut val = || it.next().ok_or(format!("{k} needs a value"));
        let num = |s: String| -> Result<f64, String> {
            match s.parse::<f64>() { Ok(x) if x.is_finite() => Ok(x), _ => Err(format!("{k}: not a finite number")) }
        };
        match k.as_str() {
            "--oils" => { a.oils.get_or_insert_with(Default::default); }
            "--latency-ms" => { let x = num(val()?)?; a.oils.get_or_insert_with(Default::default).fixed_s = Some(x*1e-3); }
            "--obc-mhz" => { let x = num(val()?)?; a.oils.get_or_insert_with(Default::default).cpu_hz = x*1e6; }
            "--cpi" => { let x = num(val()?)?; a.oils.get_or_insert_with(Default::default).cpi = x; }
            "--i2c-khz" => { let x = num(val()?)?; a.oils.get_or_insert_with(Default::default).i2c_hz = x*1e3; }
            "--can-kbps" => { let x = num(val()?)?; a.oils.get_or_insert_with(Default::default).can_bps = x*1e3; }
            "--case" => a.case = Some(val()?),
            "--fsw" => a.fsw = val()?.parse()?,
            "--against" => a.fsw_b = Some(val()?.parse()?),
            "--realtime" => a.realtime = true,
            "--seed" => a.seed = val()?.parse().map_err(|_| "--seed: not a whole number")?,
            "--out" => a.out = Some(val()?.into()),
            "--set" => { let s = val()?; let (k, v) = s.split_once('=').ok_or("--set k=v")?; a.set.push((k.into(), v.into())); }
            "--alg" => { let s = val()?; let (k, v) = s.split_once('=').ok_or("--alg slot=id")?; a.set.push((format!("fsw.algorithms.{k}"), format!("\"{v}\""))); }
            "--knobs" => { let f = val()?; a.set.push(("knobs".into(), f)); }
            "--quiet" | "-q" => a.quiet = true,
            _ => return Err(format!("unknown option {k}\n{}", usage())),
        }
    }
    Ok(a)
}
