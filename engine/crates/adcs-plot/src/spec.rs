//! Figures from their JSON description (the schema is in the crate doc): what Python hands
//! `adcs plot`. An unknown kind, scale or layout is refused by name, never guessed.
//! Owner: Agastya. Copyright (c) 2026 Agastya. All rights reserved.
use crate::*;
use serde_json::Value;

fn s(v: &Value, k: &str) -> String { v.get(k).and_then(Value::as_str).unwrap_or("").to_string() }
fn f(v: &Value, k: &str, d: f64) -> f64 { v.get(k).and_then(Value::as_f64).unwrap_or(d) }
fn nums(v: &Value, k: &str) -> Vec<f64> {
    v.get(k).and_then(Value::as_array).map(|a| a.iter().map(|x| x.as_f64().unwrap_or(f64::NAN)).collect()).unwrap_or_default()
}
fn strs(v: &Value, k: &str) -> Vec<String> {
    v.get(k).and_then(Value::as_array).map(|a| a.iter().map(|x| x.as_str().map(String::from).unwrap_or_else(|| x.to_string())).collect()).unwrap_or_default()
}
fn color(v: &Value, d: &str) -> String { v.get("color").and_then(Value::as_str).unwrap_or(d).to_string() }

fn scale(v: &Value, k: &str) -> Result<Scale, String> {
    match v.get(k).and_then(Value::as_str).unwrap_or("linear") {
        "linear" | "lin" => Ok(Scale::Linear), "log" | "log10" => Ok(Scale::Log), o => Err(format!("{k} {o:?}: linear or log")),
    }
}

fn lim(v: &Value, k: &str) -> Result<Option<(f64, f64)>, String> {
    match v.get(k) {
        None | Some(Value::Null) => Ok(None),
        Some(x) => match x.as_array().map(|a| a.iter().map(Value::as_f64).collect::<Vec<_>>()).as_deref() {
            Some([Some(a), Some(b)]) if a < b => Ok(Some((*a, *b))),
            _ => Err(format!("{k} must be [lo, hi] with lo < hi")),
        },
    }
}

fn ticks(v: &Value, k: &str) -> Result<Vec<(f64, String)>, String> {
    let Some(a) = v.get(k).and_then(Value::as_array) else { return Ok(vec![]) };
    a.iter().map(|t| match t.as_array().map(|p| (p.first().and_then(Value::as_f64), p.get(1))) {
        Some((Some(x), Some(l))) => Ok((x, l.as_str().map(String::from).unwrap_or_else(|| l.to_string()))),
        Some((Some(x), None)) => Ok((x, fmt_g(x, 6))),
        _ => Err(format!("{k}: each tick is [value, \"label\"]")),
    }).collect()
}

fn series(v: &Value) -> Result<Series, String> {
    let c = color(v, S1);
    let mut s = match v.get("kind").and_then(Value::as_str).unwrap_or("") {
        "line" => Series::line(&nums(v, "x"), &nums(v, "y"), &c),
        "step" => Series::step(&nums(v, "x"), &nums(v, "y"), &c),
        "scatter" => Series::scatter(&nums(v, "x"), &nums(v, "y"), &c),
        "hist" => if v.get("edges").is_some() {
            let (e, n) = (nums(v, "edges"), nums(v, "counts"));
            if e.len() != n.len() + 1 { return Err(format!("hist: {} edges for {} counts (one more edge than counts)", e.len(), n.len())); }
            Series::hist(&e, &n, &c)
        } else {
            let x = nums(v, "values");
            let n = v.get("bins").and_then(Value::as_u64).map(|b| b as usize).unwrap_or(((x.len() as f64).sqrt() as usize) + 2);
            Series::hist_values(&x, n.clamp(1, 1000), &c)
        },
        "hbar" => {
            let vals = nums(v, "values");
            let mut s = Series::hbar(&[], &vals, &c);
            if let Kind::HBar { labels, texts, colors, .. } = &mut s.kind { *labels = strs(v, "labels"); *texts = strs(v, "texts"); *colors = strs(v, "colors"); }
            s
        }
        o => return Err(format!("series kind {o:?}: line, step, scatter, hist or hbar")),
    };
    s.label = self::s(v, "label");
    s.width = f(v, "width", s.width);
    s.dashed = v.get("dashed").and_then(Value::as_bool).unwrap_or(false);
    if let Some(m) = v.get("marker").and_then(Value::as_str) {
        s.marker = match m { "none" => Marker::None, "circle" | "o" => Marker::Circle, "ring" => Marker::Ring, "diamond" | "D" => Marker::Diamond,
                             "square" | "s" => Marker::Square, o => return Err(format!("marker {o:?}: none, circle, ring, diamond or square")) };
    }
    s.size = f(v, "size", s.size);
    if s.x.len() != s.y.len() { return Err(format!("series {:?}: {} x values, {} y values", s.label, s.x.len(), s.y.len())); }
    Ok(s)
}

fn panel(v: &Value) -> Result<Panel, String> {
    let mut p = Panel::new(&s(v, "title"));
    p.xlabel = s(v, "xlabel");
    p.ylabel = s(v, "ylabel");
    p.xscale = scale(v, "xscale")?;
    p.yscale = scale(v, "yscale")?;
    p.xlim = lim(v, "xlim")?;
    p.ylim = lim(v, "ylim")?;
    p.xticks = ticks(v, "xticks")?;
    p.yticks = ticks(v, "yticks")?;
    p.off = v.get("off").and_then(Value::as_bool).unwrap_or(false);
    p.legend = match v.get("legend") {
        None | Some(Value::Null) | Some(Value::Bool(false)) => None,
        Some(Value::Bool(true)) => Some(Corner::UpperRight),
        Some(x) => Some(match x.as_str().unwrap_or("") {
            "upper right" => Corner::UpperRight, "upper left" => Corner::UpperLeft, "lower right" => Corner::LowerRight, "lower left" => Corner::LowerLeft,
            o => return Err(format!("legend {o:?}: upper right, upper left, lower right or lower left")),
        }),
    };
    for x in v.get("series").and_then(Value::as_array).into_iter().flatten() { p.series.push(series(x)?); }
    for r in v.get("refs").and_then(Value::as_array).into_iter().flatten() {
        let at = r.get("at").and_then(Value::as_f64).ok_or("a reference line needs \"at\"")?;
        let vertical = match r.get("axis").and_then(Value::as_str).unwrap_or("y") { "x" => true, "y" => false, o => return Err(format!("ref axis {o:?}: x or y")) };
        p.refs.push(RefLine { vertical, at, label: s(r, "label"), color: color(r, INK), dashed: r.get("dashed").and_then(Value::as_bool).unwrap_or(true), width: f(r, "width", 1.3) });
    }
    for n in v.get("notes").and_then(Value::as_array).into_iter().flatten() {
        let anchor = match n.get("anchor").and_then(Value::as_str).unwrap_or("start") { "middle" => Anchor::Middle, "end" => Anchor::End, _ => Anchor::Start };
        p.notes.push(Note { x: f(n, "x", f64::NAN), y: f(n, "y", f64::NAN), text: s(n, "text"), color: color(n, INK), anchor });
    }
    Ok(p)
}

fn figure(v: &Value) -> Result<Figure, String> {
    let panels = v.get("panels").and_then(Value::as_array).ok_or("a figure needs \"panels\"")?
        .iter().map(panel).collect::<Result<Vec<_>, _>>()?;
    let (w, h) = (f(v, "width", 720.0), f(v, "height", 446.0));
    if !(w >= 50.0 && h >= 50.0 && w <= 20000.0 && h <= 20000.0) { return Err(format!("figure size {w} x {h}: 50 to 20000 points")); }
    let mut fig = match v.get("layout").and_then(Value::as_str).unwrap_or("stack") {
        "stack" => Figure::stack(w, h, &nums(v, "ratios"), panels),
        "grid" => {
            let cols = v.get("cols").and_then(Value::as_u64).unwrap_or(1).max(1) as usize;
            let rows = v.get("rows").and_then(Value::as_u64).map(|r| r as usize).unwrap_or(panels.len().div_ceil(cols)).max(1);
            if rows * cols < panels.len() { return Err(format!("a {rows} x {cols} grid holds fewer than {} panels", panels.len())); }
            Figure::grid(w, h, rows, cols, panels)
        }
        o => return Err(format!("layout {o:?}: stack or grid")),
    };
    fig.title = s(v, "title");
    if let Some(b) = v.get("sharex").and_then(Value::as_bool) { fig.sharex = b; }
    Ok(fig)
}

pub fn figures(v: &Value) -> Result<Vec<Figure>, String> {
    let list = match v {
        Value::Array(a) => a.iter().collect::<Vec<_>>(),
        Value::Object(o) if o.contains_key("figures") => o["figures"].as_array().ok_or("\"figures\" must be a list")?.iter().collect(),
        Value::Object(_) => vec![v],
        _ => return Err("a figure description is an object or a list of them".into()),
    };
    if list.is_empty() { return Err("no figures".into()); }
    list.into_iter().enumerate().map(|(i, x)| figure(x).map_err(|e| format!("figure {}: {e}", i + 1))).collect()
}

#[cfg(test)]
mod t {
    use crate::*;
    use serde_json::json;

    #[test]
    fn a_description_draws_and_a_bad_one_is_refused_by_name() {
        let v = json!({"figures": [
            {"title": "campaign", "layout": "grid", "cols": 2, "width": 720, "height": 260, "panels": [
                {"title": "hist", "series": [{"kind": "hist", "values": [1, 2, 2, 3, null, 4], "bins": 4}], "refs": [{"axis": "x", "at": 2.5, "label": "req"}]},
                {"title": "cdf", "series": [{"kind": "step", "x": [1, 2, 3], "y": [0.3, 0.6, 1.0], "color": "S2"}], "legend": "lower right"}]},
            {"panels": [{"xscale": "log", "series": [{"kind": "hbar", "labels": ["a", "b"], "values": [0.01, 3], "colors": ["S3", "#b9bec4"]},
                                                     {"kind": "scatter", "x": [0.02], "y": [0], "marker": "diamond"}],
                         "notes": [{"x": 1, "y": 1, "text": "note"}]}]}]});
        let f = Figure::from_json(&v).unwrap();
        assert_eq!(f.len(), 2);
        assert!(svg::t::well_formed(&f[0].to_svg()).is_ok() && f[1].to_svg().contains(">a</text>"));
        assert!(pdf::t::check(&pdf_of(&f)).is_ok());
        for (bad, why) in [(json!({"panels": [{"series": [{"kind": "pie"}]}]}), "pie"), (json!({"panels": [{"yscale": "ln"}]}), "ln"),
                           (json!({"title": "x"}), "panels"), (json!({"panels": [{"xlim": [3, 1]}]}), "xlim"),
                           (json!({"panels": [{"series": [{"kind": "line", "x": [1, 2], "y": [1]}]}]}), "y values"),
                           (json!({"layout": "grid", "rows": 1, "cols": 1, "panels": [{}, {}]}), "grid")] {
            let e = Figure::from_json(&bad).unwrap_err();
            assert!(e.contains(why), "{e}");
        }
    }
}
