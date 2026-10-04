//! The report of one run: what it flew (scenario, case, product, flight software, seed, engine,
//! result id, input fingerprints), every metric against its requirement with the verdict, and
//! the run's figures. As HTML (figures inline as SVG, a print stylesheet so a browser prints it
//! to PDF) and as PDF (a first page of text, then a page per figure).
//! Owner: Agastya. Copyright (c) 2026 Agastya. All rights reserved.
use crate::draw::{render, Page};
use crate::run::metrics;
use crate::svg::esc;
use crate::*;
use serde_json::Value;

fn txt(v: Option<&Value>) -> String {
    match v {
        None | Some(Value::Null) => "—".into(),
        Some(Value::String(s)) if s.is_empty() => "—".into(),
        Some(Value::String(s)) => s.clone(),
        Some(Value::Number(n)) => n.as_f64().map(|x| fmt_g(x, 6)).unwrap_or_else(|| n.to_string()),
        Some(Value::Array(a)) => a.iter().map(|x| txt(Some(x))).collect::<Vec<_>>().join(", "),
        Some(o) => o.to_string(),
    }
}

/// The run's provenance as (what, value) rows; what a manifest does not say shows as "—".
pub fn provenance(man: &Value) -> Vec<(&'static str, String)> {
    let g = |p: &str| man.pointer(p);
    let fsw = g("/fsw/impl").or(g("/inputs/fsw")).or(g("/fsw"));
    let fsw = fsw.filter(|v| !v.is_object()).cloned();
    let dur = g("/duration_s").and_then(Value::as_f64).map(|d| format!("{} s ({:.1} min)", fmt_g(d, 6), d / 60.0)).unwrap_or("—".into());
    vec![
        ("scenario", txt(g("/scenario"))), ("label", txt(g("/label"))), ("case", txt(g("/case"))), ("case title", txt(g("/case_title"))),
        ("product", txt(g("/product"))), ("flight software", txt(fsw.as_ref())), ("flight software build", txt(g("/fsw/build_id"))),
        ("seed", txt(g("/seed"))), ("duration", dur), ("step", g("/dt_s").and_then(Value::as_f64).map(|d| format!("{} s", fmt_g(d, 6))).unwrap_or("—".into())),
        ("engine", txt(g("/engine"))), ("engine version", txt(g("/engine_version"))), ("engine source", txt(g("/engine_source"))),
        ("result id", txt(g("/result_id"))), ("created (UTC)", txt(g("/created_utc"))),
        ("case file", txt(g("/inputs/case_file"))), ("case fingerprint", txt(g("/inputs/case_fingerprint"))),
        ("scenario fingerprint", txt(g("/inputs/scenario_fingerprint"))), ("product fingerprint", txt(g("/product_fingerprint"))),
        ("overrides", txt(g("/inputs/overrides"))),
    ]
}

/// Every metric: id, value, unit, requirement, verdict ("pass", "FAIL" or "—" when it only reports).
pub fn verdicts(man: &Value) -> Vec<[String; 5]> {
    metrics(man).iter().map(|m| {
        let v = |k: &str| m.get(k).and_then(Value::as_f64).map(|x| fmt_g(x, 4)).unwrap_or("—".into());
        let p = match m.get("pass") { Some(Value::Bool(b)) => Some(*b), Some(Value::Number(n)) => n.as_f64().map(|x| x != 0.0), _ => None };
        [txt(m.get("id")), v("value"), txt(m.get("unit")), v("req"), match p { Some(true) => "pass", Some(false) => "FAIL", None => "—" }.into()]
    }).collect()
}

fn title(man: &Value) -> String {
    format!("{} on {}", txt(man.get("scenario")), txt(man.get("case")))
}

/// The report as one self-contained HTML page.
pub fn report_html(man: &Value, figs: &[(String, Figure)]) -> String {
    let css = "\
:root{--bg:#fcfcfb;--text:#0b0b0b;--muted:#52514e;--line:#e4e3df;--pass:#0a7a3a;--fail:#b3261e}
body{background:var(--bg);color:var(--text);font:14px/1.5 Helvetica,Arial,sans-serif;margin:0}
main{max-width:1000px;margin:0 auto;padding:24px 16px 48px}
h1{font-size:24px;margin:0 0 4px}h2{font-size:17px;margin:28px 0 6px;border-top:1px solid var(--line);padding-top:14px}
.muted{color:var(--muted)}table{border-collapse:collapse;width:100%;font-variant-numeric:tabular-nums}
th,td{border-bottom:1px solid var(--line);padding:5px 8px;text-align:left;vertical-align:top}th{color:var(--muted);font-weight:600;font-size:12px}
td.n{text-align:right}.pass{color:var(--pass);font-weight:600}.fail{color:var(--fail);font-weight:600}code{font-family:ui-monospace,Menlo,monospace;font-size:.92em}
figure{margin:12px 0 20px}figure svg{display:block;width:100%;height:auto;border:1px solid var(--line);border-radius:4px}figcaption{color:var(--muted);font-size:12px}
@media print{@page{size:A4 landscape;margin:12mm}body{font-size:11px}main{max-width:none;padding:0}h2{break-after:avoid}
figure{break-inside:avoid;page-break-inside:avoid}figure svg{border:none;max-height:170mm}tr{break-inside:avoid}}
";
    let mut o = format!("<!doctype html>\n<html lang=\"en\"><head><meta charset=\"utf-8\"><meta name=\"viewport\" content=\"width=device-width, initial-scale=1\">\
                         <title>{t}</title><style>{css}</style></head><body><main>\n<h1>{t}</h1>\n<p class=\"muted\">{}</p>\n",
                        esc(&txt(man.get("label"))), t = esc(&format!("Run report: {}", title(man))));
    o += "<h2>What was flown</h2>\n<table>\n";
    for (k, v) in provenance(man) { o += &format!("<tr><th>{}</th><td><code>{}</code></td></tr>\n", esc(k), esc(&v)); }
    o += "</table>\n<h2>Requirements and verdicts</h2>\n<table>\n<tr><th>metric</th><th>value</th><th>unit</th><th>requirement</th><th>verdict</th></tr>\n";
    for r in verdicts(man) {
        let cls = match r[4].as_str() { "pass" => "pass", "FAIL" => "fail", _ => "muted" };
        o += &format!("<tr><td><code>{}</code></td><td class=\"n\">{}</td><td>{}</td><td class=\"n\">{}</td><td class=\"{cls}\">{}</td></tr>\n",
                      esc(&r[0]), esc(&r[1]), esc(&r[2]), esc(&r[3]), esc(&r[4]));
    }
    o += "</table>\n<h2>Figures</h2>\n";
    for (name, f) in figs { o += &format!("<figure>\n{}<figcaption>{}</figcaption></figure>\n", f.to_svg(), esc(name)); }
    o += "<p class=\"muted\">Drawn by adcs-plot (<code>adcs report</code>) from the run's manifest.json and channels.csv.</p>\n</main></body></html>\n";
    o
}

/// The report as one PDF: the provenance and the verdicts as text (more pages when they run
/// long), then a page per figure.
pub fn report_pdf(man: &Value, figs: &[(String, Figure)]) -> Vec<u8> {
    let (w, h) = (842.0, 595.0);
    let mut pages: Vec<Page> = Vec::new();
    let mut pg = Page::new(w, h);
    let mut y = 50.0;
    pg.text(40.0, y, &format!("Run report: {}", title(man)), 18.0, INK, Anchor::Start, true);
    y += 18.0;
    pg.text(40.0, y, &txt(man.get("label")), 10.0, INK2, Anchor::Start, false);
    y += 26.0;
    let mut row = |pg: &mut Page, y: &mut f64, cells: &[(f64, String, &str, bool)]| {
        if *y > h - 40.0 { pages.push(std::mem::replace(pg, Page::new(w, h))); *y = 50.0; }
        for (x, s, c, b) in cells { pg.text(*x, *y, s, 9.0, c, Anchor::Start, *b); }
        *y += 13.0;
    };
    row(&mut pg, &mut y, &[(40.0, "What was flown".into(), INK, true)]);
    for (k, v) in provenance(man) { row(&mut pg, &mut y, &[(40.0, k.into(), INK2, false), (170.0, v, INK, false)]); }
    y += 10.0;
    row(&mut pg, &mut y, &[(40.0, "Requirements and verdicts".into(), INK, true)]);
    let x = [40.0, 250.0, 340.0, 420.0, 520.0];
    row(&mut pg, &mut y, &[(x[0], "metric".into(), INK2, true), (x[1], "value".into(), INK2, true), (x[2], "unit".into(), INK2, true),
                           (x[3], "requirement".into(), INK2, true), (x[4], "verdict".into(), INK2, true)]);
    for r in verdicts(man) {
        let c = match r[4].as_str() { "pass" => "#0a7a3a", "FAIL" => "#b3261e", _ => INK2 };
        row(&mut pg, &mut y, &[(x[0], r[0].clone(), INK, false), (x[1], r[1].clone(), INK, false), (x[2], r[2].clone(), INK, false),
                               (x[3], r[3].clone(), INK, false), (x[4], r[4].clone(), c, true)]);
    }
    pages.push(pg);
    pages.extend(figs.iter().map(|(_, f)| render(f)));
    pdf::pdf(&pages)
}

#[cfg(test)]
mod t {
    use super::*;

    #[test]
    fn a_report_has_provenance_verdicts_and_figures() {
        let (man, ch) = crate::run::t::synthetic(2000, false);
        let figs = run_figures(&man, &ch, true);
        let h = report_html(&man, &figs);
        assert!(h.contains("detumble_ais") && h.contains("power_mean") && h.contains("class=\"pass\"") && h.contains("@media print"));
        assert_eq!(h.matches("<svg ").count(), figs.len());
        let p = report_pdf(&man, &figs);
        assert_eq!(crate::pdf::t::check(&p), Ok(5 + 2 * (figs.len() + 1)), "the xref: free entry, catalog, pages, two fonts, a page and its content per page");
    }
}
