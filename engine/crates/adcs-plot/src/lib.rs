//! The one plotting module of the project: a small figure model, drawn the same way as SVG (a
//! self-contained string) and as PDF (PDF 1.4 written by hand, one page per figure, the
//! Helvetica base-14 font), for the engine's runs and the MATLAB twin's runs alike
//! (docs/RELEASE_PLAN.md, P11). `run` holds the figure set of one run and `report` the per-run
//! report; Python describes any other figure as JSON and draws it with `adcs plot`.
//!
//! A figure is panels stacked vertically (optionally with height ratios, the x axis shared) or in
//! a grid. A panel has axes (linear or log10, fixed or automatic limits, 1-2-5 or decade ticks),
//! series (line, post step, scatter, histogram, horizontal bars), dashed reference lines with a
//! label, text notes and a legend. NaN breaks a line; on a log axis a value at or below zero is
//! dropped. Long series are thinned for drawing to the smallest and largest value per pixel
//! column, so a 17,000-sample run stays a small file.
//!
//! ## The JSON description (`adcs plot SPEC.json`, `Figure::from_json`)
//!
//! A document is one figure object, a list of them, or `{"figures": [...]}`. Every key is
//! optional unless marked; `null` in a number list is NaN. Colours are `"#rrggbb"` or a palette
//! name: `S1` `S2` `S3` `S4` (blue, orange, green, amber), `INK`, `INK2`, `GRID`, `SURF`, `MUTED`.
//!
//! ```text
//! figure: {"title": str, "width": 720, "height": 446,         size in points (1/72 in)
//!          "layout": "stack" | "grid", "rows": n, "cols": n,   grid: panels fill row by row
//!          "ratios": [h1, h2, ...],                            stack: relative panel heights
//!          "sharex": true,                                     stack: one x range, labels on the last panel
//!          "panels": [panel, ...]}                             (required)
//! panel:  {"title": str ("\n" for a second line), "xlabel": str, "ylabel": str,
//!          "xscale": "linear" | "log", "yscale": "linear" | "log",
//!          "xlim": [lo, hi], "ylim": [lo, hi],
//!          "xticks": [[value, "label"], ...], "yticks": [[value, "label"], ...],   fixed (categorical) ticks
//!          "legend": true | false | "upper right" | "upper left" | "lower right" | "lower left",
//!          "off": true,                                        an empty cell of a grid
//!          "series": [series, ...],
//!          "refs": [{"axis": "y" (horizontal line) | "x" (vertical line), "at": v (required),
//!                    "label": str, "color": c, "dashed": true, "width": 1.2}, ...],
//!          "notes": [{"x": v, "y": v, "text": str, "color": c, "anchor": "start" | "middle" | "end"}, ...]}
//! series: {"kind": "line" | "step" | "scatter" | "hist" | "hbar"  (required),
//!          "x": [...], "y": [...],                             line, step (post), scatter
//!          "label": str (legend), "color": c, "width": 1.4, "dashed": false,
//!          "marker": "none" | "circle" | "ring" | "diamond" | "square", "size": 3 (marker radius),
//!          "values": [...], "bins": n,                         hist from samples (n bins, default sqrt(n)+2)
//!          "edges": [...], "counts": [...],                    hist from counts (edges one longer)
//!          "labels": [...], "values": [...],                   hbar: one bar per category, first at the top;
//!          "texts": [...], "colors": [...]}                    text after each bar (default the value), bar colours
//! ```
//!
//! A panel with an `hbar` series has categorical y (bar i at y = i, first at the top), so a
//! scatter on the same panel (each seed of a candidate, say) uses y = the bar index.
//!
//! Owner: Agastya. Copyright (c) 2026 Agastya. All rights reserved.
mod draw;
pub mod pdf;
pub mod report;
pub mod run;
mod spec;
pub mod svg;
mod text;

pub use draw::{render, Op, Page};
pub use report::{report_html, report_pdf};
pub use run::{run_figures, Channels};

/// The palette (tools/report_base.py): series S1..S4 in fixed order, ink, grid and surface.
pub const S1: &str = "#2a78d6";
pub const S2: &str = "#eb6834";
pub const S3: &str = "#1baf7a";
pub const S4: &str = "#eda100";
pub const INK: &str = "#0b0b0b";
pub const INK2: &str = "#52514e";
pub const GRID: &str = "#e4e3df";
pub const SURF: &str = "#fcfcfb";
pub const MUTED: &str = "#b9bec4";

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum Scale { #[default] Linear, Log }

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum Marker { #[default] None, Circle, Ring, Diamond, Square }

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum Corner { #[default] UpperRight, UpperLeft, LowerRight, LowerLeft }

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum Anchor { #[default] Start, Middle, End }

#[derive(Clone, Debug)]
pub enum Kind {
    Line,
    Step,
    Scatter,
    Hist { edges: Vec<f64>, counts: Vec<f64> },
    HBar { labels: Vec<String>, values: Vec<f64>, texts: Vec<String>, colors: Vec<String> },
}

#[derive(Clone, Debug)]
pub struct Series {
    pub kind: Kind, pub x: Vec<f64>, pub y: Vec<f64>, pub label: String, pub color: String,
    pub width: f64, pub dashed: bool, pub marker: Marker, pub size: f64,
}

impl Series {
    fn of(kind: Kind, x: Vec<f64>, y: Vec<f64>, color: &str) -> Self {
        Series { kind, x, y, label: String::new(), color: color.into(), width: 1.4, dashed: false, marker: Marker::None, size: 3.0 }
    }
    pub fn line(x: &[f64], y: &[f64], color: &str) -> Self { Self::of(Kind::Line, x.to_vec(), y.to_vec(), color) }
    pub fn step(x: &[f64], y: &[f64], color: &str) -> Self { Self::of(Kind::Step, x.to_vec(), y.to_vec(), color) }
    pub fn scatter(x: &[f64], y: &[f64], color: &str) -> Self {
        Series { marker: Marker::Circle, size: 0.9, ..Self::of(Kind::Scatter, x.to_vec(), y.to_vec(), color) }
    }
    /// A histogram of the finite samples in `bins` equal bins between their smallest and largest.
    pub fn hist_values(values: &[f64], bins: usize, color: &str) -> Self {
        let v: Vec<f64> = values.iter().copied().filter(|x| x.is_finite()).collect();
        let (lo, hi) = v.iter().fold((f64::INFINITY, f64::NEG_INFINITY), |(a, b), &x| (a.min(x), b.max(x)));
        let n = bins.max(1);
        let (lo, hi) = if v.is_empty() { (0.0, 1.0) } else if hi > lo { (lo, hi) } else { (lo - 0.5, hi + 0.5) };
        let edges: Vec<f64> = (0..=n).map(|i| lo + (hi - lo) * i as f64 / n as f64).collect();
        let mut counts = vec![0.0; n];
        for x in v { counts[(((x - lo) / (hi - lo) * n as f64) as usize).min(n - 1)] += 1.0; }
        Self::hist(&edges, &counts, color)
    }
    pub fn hist(edges: &[f64], counts: &[f64], color: &str) -> Self {
        Self::of(Kind::Hist { edges: edges.to_vec(), counts: counts.to_vec() }, vec![], vec![], color)
    }
    pub fn hbar(labels: &[&str], values: &[f64], color: &str) -> Self {
        Self::of(Kind::HBar { labels: labels.iter().map(|s| s.to_string()).collect(), values: values.to_vec(), texts: vec![], colors: vec![] },
                 vec![], vec![], color)
    }
    pub fn label(mut self, l: &str) -> Self { self.label = l.into(); self }
    pub fn width(mut self, w: f64) -> Self { self.width = w; self }
}

/// A horizontal (`vertical: false`, at y) or vertical (at x) reference line with its label.
#[derive(Clone, Debug)]
pub struct RefLine { pub vertical: bool, pub at: f64, pub label: String, pub color: String, pub dashed: bool, pub width: f64 }

impl RefLine {
    pub fn h(at: f64, label: &str) -> Self { RefLine { vertical: false, at, label: label.into(), color: INK.into(), dashed: true, width: 1.3 } }
    pub fn v(at: f64, label: &str) -> Self { RefLine { vertical: true, ..Self::h(at, label) } }
}

/// Text at a point in data coordinates.
#[derive(Clone, Debug)]
pub struct Note { pub x: f64, pub y: f64, pub text: String, pub color: String, pub anchor: Anchor }

#[derive(Clone, Debug, Default)]
pub struct Panel {
    pub title: String, pub xlabel: String, pub ylabel: String,
    pub xscale: Scale, pub yscale: Scale,
    pub xlim: Option<(f64, f64)>, pub ylim: Option<(f64, f64)>,
    pub xticks: Vec<(f64, String)>, pub yticks: Vec<(f64, String)>,
    pub series: Vec<Series>, pub refs: Vec<RefLine>, pub notes: Vec<Note>,
    pub legend: Option<Corner>, pub off: bool,
}

impl Panel {
    pub fn new(title: &str) -> Self { Panel { title: title.into(), ..Default::default() } }
    pub fn logy(mut self) -> Self { self.yscale = Scale::Log; self }
    pub fn ylabel(mut self, s: &str) -> Self { self.ylabel = s.into(); self }
    pub fn xlabel(mut self, s: &str) -> Self { self.xlabel = s.into(); self }
    pub fn ylim(mut self, lo: f64, hi: f64) -> Self { self.ylim = Some((lo, hi)); self }
    pub fn legend(mut self, c: Corner) -> Self { self.legend = Some(c); self }
    pub fn with(mut self, s: Series) -> Self { self.series.push(s); self }
    pub fn refl(mut self, r: RefLine) -> Self { self.refs.push(r); self }
}

#[derive(Clone, Debug)]
pub enum Layout { Stack(Vec<f64>), Grid { rows: usize, cols: usize } }

#[derive(Clone, Debug)]
pub struct Figure { pub title: String, pub width: f64, pub height: f64, pub layout: Layout, pub sharex: bool, pub panels: Vec<Panel> }

impl Figure {
    /// Panels stacked top to bottom sharing one x axis; `ratios` empty for equal heights.
    pub fn stack(width: f64, height: f64, ratios: &[f64], panels: Vec<Panel>) -> Self {
        Figure { title: String::new(), width, height, layout: Layout::Stack(ratios.to_vec()), sharex: true, panels }
    }
    pub fn grid(width: f64, height: f64, rows: usize, cols: usize, panels: Vec<Panel>) -> Self {
        Figure { title: String::new(), width, height, layout: Layout::Grid { rows, cols }, sharex: false, panels }
    }
    pub fn titled(mut self, t: &str) -> Self { self.title = t.into(); self }
    pub fn to_svg(&self) -> String { svg::svg(&render(self)) }
    pub fn to_pdf(&self) -> Vec<u8> { pdf::pdf(&[render(self)]) }
    /// Figures from a JSON description (the schema above): one object, a list, or {"figures": [...]}.
    pub fn from_json(v: &serde_json::Value) -> Result<Vec<Figure>, String> { spec::figures(v) }
}

/// Several figures as one PDF, a page each.
pub fn pdf_of(figs: &[Figure]) -> Vec<u8> { pdf::pdf(&figs.iter().map(render).collect::<Vec<_>>()) }

/// `x` as C's `%.{sig}g` prints it, without trailing zeros: 0.01, 37.74, 1e-05.
pub fn fmt_g(x: f64, sig: usize) -> String {
    if !x.is_finite() { return if x.is_nan() { "NaN".into() } else if x > 0.0 { "inf".into() } else { "-inf".into() }; }
    if x == 0.0 { return "0".into(); }
    let sig = sig.max(1);
    let e = format!("{:.*e}", sig - 1, x);
    let (m, ex) = e.split_once('e').unwrap();
    let ex: i32 = ex.parse().unwrap();
    if ex < -4 || ex >= sig as i32 {
        let m = if m.contains('.') { m.trim_end_matches('0').trim_end_matches('.') } else { m };
        format!("{m}e{}{:02}", if ex < 0 { "-" } else { "+" }, ex.abs())
    } else {
        let s = format!("{:.*}", (sig as i32 - 1 - ex).max(0) as usize, x);
        if s.contains('.') { s.trim_end_matches('0').trim_end_matches('.').to_string() } else { s }
    }
}

#[cfg(test)]
mod t {
    use super::*;

    #[test]
    fn numbers_print_as_percent_g() {
        for (x, s, want) in [(0.01, 4, "0.01"), (37.74, 4, "37.74"), (1e-5, 4, "1e-05"), (123456.0, 4, "1.235e+05"), (20.0, 6, "20"),
                             (0.5, 6, "0.5"), (-2.5, 3, "-2.5"), (284.0, 6, "284"), (0.000123, 3, "0.000123")] {
            assert_eq!(fmt_g(x, s), want, "{x}");
        }
    }
}
