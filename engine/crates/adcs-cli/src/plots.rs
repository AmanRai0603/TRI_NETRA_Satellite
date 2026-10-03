//! The drawing commands (adcs-plot): `figures` and `report` for one run folder, engine or MATLAB
//! twin alike (manifest.json + channels.csv), and `plot` for any figure described as JSON.
//! Owner: Agastya. Copyright (c) 2026 Agastya. All rights reserved.
use crate::cli::{FiguresArgs, Format, PlotArgs, ReportArgs};
use adcs_plot::{run_figures, Channels, Figure};
use adcs_sim::{fsio, Error};
use serde_json::Value;
use std::path::{Path, PathBuf};

fn read(p: &Path) -> Result<String, Error> {
    if !p.is_file() { return Err(Error::refused(format!("{}: no such file", p.display()))); }
    std::fs::read_to_string(p).map_err(|e| Error::io(p, e))
}

/// A run folder's manifest and time series.
fn load(dir: &Path) -> Result<(Value, Channels), Error> {
    let m = dir.join("manifest.json");
    if !m.is_file() { return Err(Error::refused(format!("{}: not a run folder (no manifest.json)", dir.display()))); }
    let man: Value = serde_json::from_str(&read(&m)?).map_err(|e| Error::malformed(format!("{}: {e}", m.display())))?;
    let c = dir.join("channels.csv");
    if !c.is_file() {
        return Err(Error::refused(format!("{}: no channels.csv (thinned or not kept); `adcs results refly {}` flies it again", dir.display(), dir.display())));
    }
    let ch = Channels::parse(&read(&c)?).map_err(|e| Error::malformed(format!("{}: {e}", c.display())))?;
    if ch.is_empty() { return Err(Error::malformed(format!("{}: no samples", c.display()))); }
    Ok((man, ch))
}

fn write(p: &Path, b: impl AsRef<[u8]>) -> Result<(), Error> {
    fsio::write(p, b)?;
    println!("{}", p.display());
    Ok(())
}

pub fn figures(a: &FiguresArgs) -> Result<(), Error> {
    let (man, ch) = load(&a.run)?;
    let prefix = a.prefix.clone().unwrap_or_else(|| a.run.canonicalize().unwrap_or(a.run.clone()).file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or("run".into()));
    for (name, f) in run_figures(&man, &ch, a.full) {
        let p = a.out.join(format!("{prefix}_{name}.{}", a.format.ext()));
        match a.format { Format::Svg => write(&p, f.to_svg())?, Format::Pdf => write(&p, f.to_pdf())? }
    }
    Ok(())
}

pub fn report(a: &ReportArgs) -> Result<(), Error> {
    let (man, ch) = load(&a.run)?;
    let figs = run_figures(&man, &ch, true);
    let out = a.out.clone().unwrap_or(a.run.clone());
    write(&out.join("report.html"), adcs_plot::report_html(&man, &figs))?;
    write(&out.join("report.pdf"), adcs_plot::report_pdf(&man, &figs))
}

pub fn plot(a: &PlotArgs) -> Result<(), Error> {
    let v: Value = serde_json::from_str(&read(&a.spec)?).map_err(|e| Error::malformed(format!("{}: {e}", a.spec.display())))?;
    let figs = Figure::from_json(&v).map_err(|e| Error::refused(format!("{}: {e}", a.spec.display())))?;
    match a.out.extension().and_then(|e| e.to_str()) {
        Some("pdf") => write(&a.out, adcs_plot::pdf_of(&figs)),
        Some("svg") if figs.len() == 1 => write(&a.out, figs[0].to_svg()),
        Some("svg") => {
            let stem = a.out.with_extension("");
            for (i, f) in figs.iter().enumerate() { write(&PathBuf::from(format!("{}_{}.svg", stem.display(), i + 1)), f.to_svg())?; }
            Ok(())
        }
        _ => Err(Error::refused(format!("{}: --out ends in .svg or .pdf", a.out.display()))),
    }
}
