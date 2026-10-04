//! A figure laid out as drawing operations in points, y down, which svg.rs and pdf.rs then write
//! as they are: the layout, the ticks, the thinning of long series and the clipping to each panel
//! happen here once, so the two backends draw the same picture.
//! Owner: Agastya. Copyright (c) 2026 Agastya. All rights reserved.
use crate::text::width;
use crate::*;
use std::collections::HashSet;

pub type P = (f64, f64);

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Rgb(pub u8, pub u8, pub u8);

/// "#rrggbb" or a palette name; anything else is ink.
pub fn rgb(s: &str) -> Rgb {
    let s = match s { "S1" => S1, "S2" => S2, "S3" => S3, "S4" => S4, "INK" => INK, "INK2" => INK2, "GRID" => GRID, "SURF" => SURF, "MUTED" => MUTED, _ => s };
    let h = |i: usize| s.get(i..i + 2).and_then(|x| u8::from_str_radix(x, 16).ok());
    match (s.len(), s.starts_with('#'), h(1), h(3), h(5)) {
        (7, true, Some(r), Some(g), Some(b)) => Rgb(r, g, b),
        _ => Rgb(11, 11, 11),
    }
}

#[derive(Clone, Debug)]
pub enum Op {
    /// A stroked polyline.
    Line { pts: Vec<P>, color: Rgb, width: f64, dash: bool },
    /// A closed shape, filled and/or outlined.
    Shape { pts: Vec<P>, fill: Option<Rgb>, stroke: Option<(Rgb, f64)> },
    /// Filled round dots of radius `r`.
    Dots { pts: Vec<P>, r: f64, color: Rgb },
    Circle { c: P, r: f64, fill: Option<Rgb>, stroke: Option<(Rgb, f64)> },
    /// Text on its baseline at (x, y); `up` sets it reading bottom to top.
    Text { x: f64, y: f64, s: String, size: f64, color: Rgb, anchor: Anchor, bold: bool, up: bool },
}

/// One figure's drawing: `w` x `h` points.
#[derive(Clone, Debug)]
pub struct Page { pub w: f64, pub h: f64, pub ops: Vec<Op> }

impl Page {
    pub fn new(w: f64, h: f64) -> Self { Page { w, h, ops: vec![rect((0.0, 0.0, w, h), Some(rgb(SURF)), None)] } }
    #[allow(clippy::too_many_arguments)]
    pub fn text(&mut self, x: f64, y: f64, s: &str, size: f64, color: &str, anchor: Anchor, bold: bool) {
        if !s.is_empty() { self.ops.push(Op::Text { x, y, s: s.into(), size, color: rgb(color), anchor, bold, up: false }); }
    }
    pub fn line(&mut self, pts: Vec<P>, color: &str, width: f64) { self.ops.push(Op::Line { pts, color: rgb(color), width, dash: false }); }
}

fn rect((x, y, w, h): (f64, f64, f64, f64), fill: Option<Rgb>, stroke: Option<(Rgb, f64)>) -> Op {
    Op::Shape { pts: vec![(x, y), (x + w, y), (x + w, y + h), (x, y + h)], fill, stroke }
}

/// One axis: data range [lo, hi] onto pixels [a, b].
#[derive(Clone, Copy)]
struct Ax { lo: f64, hi: f64, a: f64, b: f64, log: bool }

impl Ax {
    fn t(&self, v: f64) -> f64 { if self.log { v.log10() } else { v } }
    fn ok(&self, v: f64) -> bool { v.is_finite() && (!self.log || v > 0.0) }
    fn px(&self, v: f64) -> f64 { self.a + (self.t(v) - self.t(self.lo)) / (self.t(self.hi) - self.t(self.lo)) * (self.b - self.a) }
}

/// 1-2-5 steps, about `n` of them, over [lo, hi].
pub fn lin_ticks(lo: f64, hi: f64, n: usize) -> Vec<(f64, String)> {
    let span = hi - lo;
    if span.is_nan() || span <= 0.0 || span.is_infinite() { return vec![(lo, fmt_g(lo, 6))]; }
    let raw = span / n.max(2) as f64;
    let mag = 10f64.powf(raw.log10().floor());
    let r = raw / mag;
    let step = mag * if r <= 1.0 { 1.0 } else if r <= 2.0 { 2.0 } else if r <= 5.0 { 5.0 } else { 10.0 };
    let e = lo.abs().max(hi.abs()).log10().floor();
    let sig = ((e - step.log10().floor()) as i64 + 1).max(e as i64 + 1).clamp(1, 12) as usize;
    let (k0, k1) = ((lo / step - 1e-9).ceil() as i64, (hi / step + 1e-9).floor() as i64);
    (k0..=k1).map(|k| { let v = k as f64 * step; (v, fmt_g(v, sig)) }).collect()
}

/// Decades over [lo, hi] (every k-th when there are many); 1, 2, 5 in each decade when fewer than three fit.
pub fn log_ticks(lo: f64, hi: f64) -> Vec<(f64, String)> {
    let lab = |e: i32, m: f64| if (-3..=4).contains(&e) { fmt_g(m * 10f64.powi(e), 6) } else if m == 1.0 { format!("1e{e}") } else { format!("{m}e{e}") };
    let (a, b) = ((lo.log10() - 1e-9).ceil() as i32, (hi.log10() + 1e-9).floor() as i32);
    if b - a >= 2 {
        let k = (((b - a) as f64) / 7.0).ceil().max(1.0) as i32;
        return (a..=b).filter(|e| (e - a) % k == 0).map(|e| (10f64.powi(e), lab(e, 1.0))).collect();
    }
    let mut out = vec![];
    for e in (a - 1)..=(b + 1) {
        for m in [1.0, 2.0, 5.0] {
            let v = m * 10f64.powi(e);
            if v >= lo * (1.0 - 1e-9) && v <= hi * (1.0 + 1e-9) { out.push((v, lab(e, m))); }
        }
    }
    if out.len() >= 2 { out } else { lin_ticks(lo, hi, 3) }
}

fn hbar_n(p: &Panel) -> Option<usize> {
    p.series.iter().filter_map(|s| if let Kind::HBar { values, .. } = &s.kind { Some(values.len()) } else { None }).max()
}

/// The axis range a panel's data, reference lines and notes need (x: `along_x`), or its fixed limits.
fn range(p: &Panel, along_x: bool) -> (f64, f64) {
    if let Some(l) = if along_x { p.xlim } else { p.ylim } { return l; }
    if !along_x { if let Some(n) = hbar_n(p) { return (-0.6, n as f64 - 0.4); } }
    let log = (if along_x { p.xscale } else { p.yscale }) == Scale::Log;
    let (mut lo, mut hi) = (f64::INFINITY, f64::NEG_INFINITY);
    let mut eat = |v: f64| if v.is_finite() && (!log || v > 0.0) { lo = lo.min(v); hi = hi.max(v); };
    let (mut bars, mut sticky0, mut tight) = (false, false, along_x);
    for s in &p.series {
        match &s.kind {
            Kind::Line | Kind::Step => (if along_x { &s.x } else { &s.y }).iter().for_each(|&v| eat(v)),
            Kind::Scatter => { tight = false; (if along_x { &s.x } else { &s.y }).iter().for_each(|&v| eat(v)) }
            Kind::Hist { edges, counts } => {
                tight = false;
                if along_x { edges.iter().for_each(|&v| eat(v)) } else { counts.iter().for_each(|&v| eat(v)); if !log { eat(0.0); sticky0 = true; } }
            }
            Kind::HBar { values, .. } => if along_x { tight = false; bars = true; values.iter().for_each(|&v| eat(v)); if !log { eat(0.0); sticky0 = true; } },
        }
    }
    for r in &p.refs { if r.vertical == along_x { eat(r.at); } }
    for n in &p.notes { eat(if along_x { n.x } else { n.y }); }
    for t in if along_x { &p.xticks } else { &p.yticks } { eat(t.0); }      // given ticks widen the view, as matplotlib's set_yticks
    if lo > hi { return if log { (0.1, 10.0) } else { (0.0, 1.0) }; }
    let top = if bars { 0.22 } else if tight { 0.0 } else { 0.05 };
    let bot = if tight { 0.0 } else { 0.05 };
    if log {
        let (mut a, mut b) = (lo.log10(), hi.log10());
        if b - a < 1e-9 { a -= 0.5; b += 0.5; }
        let d = b - a;
        (10f64.powf(a - bot * d), 10f64.powf(b + top * d))
    } else {
        if hi - lo < 1e-300f64.max(hi.abs() * 1e-12) { let d = (lo.abs() * 0.1).max(1.0); lo -= d; hi += d; }
        let d = hi - lo;
        (if sticky0 && lo == 0.0 { 0.0 } else { lo - bot * d }, if sticky0 && hi == 0.0 { 0.0 } else { hi + top * d })
    }
}

/// Points of one run of samples thinned to at most four per pixel column: the first, the
/// smallest, the largest and the last, in the order they come.
fn thin(pts: &[P]) -> Vec<P> {
    if pts.len() < 8 { return pts.to_vec(); }
    let mut out = Vec::new();
    let mut i = 0;
    while i < pts.len() {
        let col = pts[i].0.floor();
        let mut j = i;
        while j + 1 < pts.len() && pts[j + 1].0.floor() == col { j += 1; }
        if j - i < 4 { out.extend_from_slice(&pts[i..=j]); } else {
            let g = &pts[i..=j];
            let (mut lo, mut hi) = (0, 0);
            for (k, p) in g.iter().enumerate() { if p.1 < g[lo].1 { lo = k; } else if p.1 > g[hi].1 { hi = k; } }
            out.push(g[0]);
            let (a, b) = if lo < hi { (lo, hi) } else { (hi, lo) };
            for k in [a, b] { if k != 0 && k != g.len() - 1 { out.push(g[k]); } }
            out.push(g[g.len() - 1]);
        }
        i = j + 1;
    }
    out
}

/// The parts of a polyline inside the rectangle (Liang-Barsky, segment by segment).
fn clip(pts: &[P], (x0, y0, x1, y1): (f64, f64, f64, f64)) -> Vec<Vec<P>> {
    let mut runs = Vec::new();
    let mut cur: Vec<P> = Vec::new();
    for w in pts.windows(2) {
        let ((ax, ay), (bx, by)) = (w[0], w[1]);
        let (dx, dy) = (bx - ax, by - ay);
        let (mut t0, mut t1) = (0.0f64, 1.0f64);
        let mut keep = true;
        for (p, q) in [(-dx, ax - x0), (dx, x1 - ax), (-dy, ay - y0), (dy, y1 - ay)] {
            if p == 0.0 { if q < 0.0 { keep = false; break; } continue; }
            let r = q / p;
            if p < 0.0 { t0 = t0.max(r) } else { t1 = t1.min(r) }
            if t0 > t1 { keep = false; break; }
        }
        if !keep { if cur.len() > 1 { runs.push(std::mem::take(&mut cur)); } cur.clear(); continue; }
        let s = (ax + t0 * dx, ay + t0 * dy);
        let e = (ax + t1 * dx, ay + t1 * dy);
        if cur.last().is_none_or(|l| (l.0 - s.0).abs() + (l.1 - s.1).abs() > 1e-9) {
            if cur.len() > 1 { runs.push(std::mem::take(&mut cur)); }
            cur = vec![s];
        }
        cur.push(e);
    }
    if cur.len() > 1 { runs.push(cur); }
    runs
}

fn clamp_rect(x0: f64, x1: f64, y0: f64, y1: f64, r: (f64, f64, f64, f64)) -> Option<(f64, f64, f64, f64)> {
    let (a, b) = (x0.min(x1).max(r.0), x0.max(x1).min(r.2));
    let (c, d) = (y0.min(y1).max(r.1), y0.max(y1).min(r.3));
    (b - a > 0.05 && d - c > 0.05).then_some((a, c, b - a, d - c))
}

fn marker(ops: &mut Vec<Op>, m: Marker, c: P, r: f64, col: Rgb) {
    match m {
        Marker::None => {}
        Marker::Circle => ops.push(Op::Dots { pts: vec![c], r, color: col }),
        Marker::Ring => ops.push(Op::Circle { c, r, fill: None, stroke: Some((col, 1.2)) }),
        Marker::Diamond => ops.push(Op::Shape { pts: vec![(c.0, c.1 - r * 1.25), (c.0 + r, c.1), (c.0, c.1 + r * 1.25), (c.0 - r, c.1)], fill: Some(col), stroke: None }),
        Marker::Square => ops.push(rect((c.0 - r, c.1 - r, 2.0 * r, 2.0 * r), Some(col), None)),
    }
}

/// The drawing of a figure.
pub fn render(f: &Figure) -> Page {
    let mut pg = Page::new(f.width, f.height);
    let mut top = 4.0;
    for (i, l) in f.title.lines().enumerate() {
        pg.text(10.0, top + 13.0, l, if i == 0 { 11.5 } else { 9.5 }, INK, Anchor::Start, i == 0);
        top += 15.0;
    }
    if !f.title.is_empty() { top += 4.0; }
    let n = f.panels.len().max(1);
    let avail = f.height - top - 2.0;
    let cells: Vec<(f64, f64, f64, f64)> = match &f.layout {
        Layout::Stack(r) => {
            let r: Vec<f64> = (0..n).map(|i| r.get(i).copied().filter(|x| *x > 0.0).unwrap_or(1.0)).collect();
            let tot: f64 = r.iter().sum();
            let mut y = top;
            r.iter().map(|ri| { let h = avail * ri / tot; y += h; (0.0, y - h, f.width, h) }).collect()
        }
        Layout::Grid { rows, cols } => {
            let (rows, cols) = ((*rows).max(1), (*cols).max(1));
            let (w, h) = (f.width / cols as f64, avail / rows as f64);
            (0..n).map(|i| ((i % cols) as f64 * w, top + (i / cols) as f64 * h, w, h)).collect()
        }
    };
    let shared = matches!(f.layout, Layout::Stack(_)) && f.sharex && n > 1;
    let xr = shared.then(|| {
        f.panels.iter().filter(|p| !p.off).map(|p| range(p, true)).fold((f64::INFINITY, f64::NEG_INFINITY), |a, r| (a.0.min(r.0), a.1.max(r.1)))
    }).filter(|r| r.0 < r.1);
    let last = f.panels.iter().rposition(|p| !p.off).unwrap_or(0);
    // a stack's plots share their left edge, as they share x
    let left_min = if matches!(f.layout, Layout::Stack(_)) {
        f.panels.iter().zip(&cells).filter(|(p, _)| !p.off).map(|(p, c)| left_need(p, c.3)).fold(0.0, f64::max)
    } else { 0.0 };
    for (i, (p, c)) in f.panels.iter().zip(cells).enumerate() {
        if !p.off { panel(&mut pg.ops, p, c, xr, !shared || i == last, left_min); }
    }
    pg
}

/// The y ticks of a panel in a cell `ch` high: given, the bar categories, decades or 1-2-5.
fn y_ticks(p: &Panel, ch: f64) -> Vec<(f64, String)> {
    let (y0, y1) = range(p, false);
    if !p.yticks.is_empty() { p.yticks.clone() } else if hbar_n(p).is_some() {
        p.series.iter().find_map(|s| match &s.kind { Kind::HBar { labels, .. } if !labels.is_empty() => Some(labels.clone()), _ => None })
            .unwrap_or_default().into_iter().enumerate().map(|(i, l)| (i as f64, l)).collect()
    } else if p.yscale == Scale::Log && y0 > 0.0 { log_ticks(y0, y1) } else { lin_ticks(y0, y1, ((ch - 40.0) / 30.0).clamp(3.0, 8.0) as usize) }
}

/// The room left of a panel's plot: the y label and the widest y tick label.
fn left_need(p: &Panel, ch: f64) -> f64 {
    let tw = y_ticks(p, ch).iter().map(|(_, s)| width(s, 8.0, false)).fold(0.0, f64::max);
    8.0 + if p.ylabel.is_empty() { 0.0 } else { 14.0 } + tw + 6.0
}

#[allow(clippy::too_many_arguments)]
fn panel(ops: &mut Vec<Op>, p: &Panel, (cx, cy, cw, ch): (f64, f64, f64, f64), xr: Option<(f64, f64)>, xtl: bool, left_min: f64) {
    let text = |ops: &mut Vec<Op>, x: f64, y: f64, s: &str, size: f64, color: Rgb, anchor: Anchor, bold: bool, up: bool| {
        if !s.is_empty() { ops.push(Op::Text { x, y, s: s.into(), size, color, anchor, bold, up }); }
    };
    let (ink, ink2, grid) = (rgb(INK), rgb(INK2), rgb(GRID));
    let nb = hbar_n(p);
    let (x0, x1) = xr.filter(|_| p.xlim.is_none()).unwrap_or_else(|| range(p, true));
    let (y0, y1) = range(p, false);
    let xlog = p.xscale == Scale::Log && x0 > 0.0;
    let ylog = p.yscale == Scale::Log && y0 > 0.0;
    let yt = y_ticks(p, ch);
    let nt = p.title.lines().count() as f64;
    let left = cx + left_need(p, ch).max(left_min);
    let right = cx + cw - 12.0;
    let ptop = cy + if nt > 0.0 { 13.0 * nt + 5.0 } else { 6.0 };
    let bottom = cy + ch - if xtl { 15.0 } else { 6.0 } - if p.xlabel.is_empty() || !xtl { 0.0 } else { 13.0 } - 4.0;
    let (pw, ph) = (right - left, bottom - ptop);
    if pw < 10.0 || ph < 10.0 { return; }
    let ax = Ax { lo: x0, hi: x1, a: left, b: right, log: xlog };
    let ay = if nb.is_some() { Ax { lo: y0, hi: y1, a: ptop, b: bottom, log: false } } else { Ax { lo: y0, hi: y1, a: bottom, b: ptop, log: ylog } };
    let inside = |v: f64, a: &Ax| { let q = a.px(v); q >= a.a.min(a.b) - 0.01 && q <= a.a.max(a.b) + 0.01 };
    let xt: Vec<(f64, String)> = if !p.xticks.is_empty() { p.xticks.clone() } else if xlog { log_ticks(x0, x1) } else { lin_ticks(x0, x1, (pw / 70.0).clamp(3.0, 9.0) as usize) };
    let xt: Vec<_> = xt.into_iter().filter(|(v, _)| ax.ok(*v) && inside(*v, &ax)).collect();
    let yt: Vec<_> = yt.into_iter().filter(|(v, _)| ay.ok(*v) && inside(*v, &ay)).collect();
    let r = (left, ptop, right, bottom);
    // grid
    for (v, _) in &xt { let q = ax.px(*v); ops.push(Op::Line { pts: vec![(q, ptop), (q, bottom)], color: grid, width: 0.7, dash: false }); }
    if nb.is_none() { for (v, _) in &yt { let q = ay.px(*v); ops.push(Op::Line { pts: vec![(left, q), (right, q)], color: grid, width: 0.7, dash: false }); } }
    // series
    let mut legend: Vec<(String, Rgb, &Series)> = Vec::new();
    for s in &p.series {
        let col = rgb(&s.color);
        if !s.label.is_empty() { legend.push((s.label.clone(), col, s)); }
        match &s.kind {
            Kind::Line | Kind::Step | Kind::Scatter => {
                let n = s.x.len().min(s.y.len());
                let ok = |i: usize| ax.ok(s.x[i]) && ay.ok(s.y[i]);
                let at = |x: f64, y: f64| (ax.px(x), ay.px(y));
                if let Kind::Scatter = s.kind {
                    let mut seen = HashSet::new();
                    let pts: Vec<P> = (0..n).filter(|&i| ok(i)).map(|i| at(s.x[i], s.y[i]))
                        .filter(|q| q.0 >= left - 0.5 && q.0 <= right + 0.5 && q.1 >= ptop - 0.5 && q.1 <= bottom + 0.5)
                        .filter(|q| seen.insert(((q.0 * 2.0).round() as i64, (q.1 * 2.0).round() as i64))).collect();
                    match s.marker {
                        Marker::Circle | Marker::None => ops.push(Op::Dots { pts, r: s.size, color: col }),
                        m => for q in pts { marker(ops, m, q, s.size, col) },
                    }
                    continue;
                }
                let mut runs: Vec<Vec<P>> = vec![vec![]];
                for i in 0..n {
                    if !ok(i) { runs.push(vec![]); continue; }
                    let run = runs.last_mut().unwrap();
                    run.push(at(s.x[i], s.y[i]));
                    if let Kind::Step = s.kind {
                        if i + 1 < n && ax.ok(s.x[i + 1]) { run.push(at(s.x[i + 1], s.y[i])); }
                    }
                }
                let mut marks = Vec::new();
                for run in runs.iter().filter(|r| !r.is_empty()) {
                    for part in clip(&thin(run), r) { ops.push(Op::Line { pts: part, color: col, width: s.width, dash: s.dashed }); }
                    if s.marker != Marker::None && n <= 400 { marks.extend(run.iter().copied().filter(|q| q.0 >= left && q.0 <= right && q.1 >= ptop && q.1 <= bottom)); }
                }
                for q in marks { marker(ops, s.marker, q, s.size, col); }
            }
            Kind::Hist { edges, counts } => {
                let base = if ylog { y0 } else { 0.0 };
                for (i, &c) in counts.iter().enumerate() {
                    let (Some(&a), Some(&b)) = (edges.get(i), edges.get(i + 1)) else { break };
                    if !(ax.ok(a) && ax.ok(b) && ay.ok(c)) { continue; }
                    if let Some(q) = clamp_rect(ax.px(a), ax.px(b), ay.px(base), ay.px(c), r) { ops.push(rect(q, Some(col), Some((rgb(SURF), 1.2)))); }
                }
            }
            Kind::HBar { values, texts, colors, .. } => {
                let base = if xlog { x0 } else { 0.0f64.clamp(x0, x1) };
                for (i, &v) in values.iter().enumerate() {
                    let c = colors.get(i).map(|c| rgb(c)).unwrap_or(col);
                    let (ya, yb) = (ay.px(i as f64 - 0.3), ay.px(i as f64 + 0.3));
                    let good = ax.ok(v);
                    if good { if let Some(q) = clamp_rect(ax.px(base), ax.px(v), ya, yb, r) { ops.push(rect(q, Some(c), None)); } }
                    let t = texts.get(i).cloned().unwrap_or_else(|| if v.is_finite() { fmt_g(v, 3) } else { "no value".into() });
                    let tx = if good { ax.px(v).clamp(left, right) } else { ax.px(base).clamp(left, right) };
                    if ay.px(i as f64) >= ptop && ay.px(i as f64) <= bottom { text(ops, tx + 3.0, ay.px(i as f64) + 3.0, &t, 8.0, ink2, Anchor::Start, false, false); }
                }
            }
        }
    }
    // reference lines
    for rl in &p.refs {
        let (a, c) = if rl.vertical { (&ax, rgb(&rl.color)) } else { (&ay, rgb(&rl.color)) };
        if !a.ok(rl.at) || !inside(rl.at, a) { continue; }
        let q = a.px(rl.at);
        if rl.vertical {
            ops.push(Op::Line { pts: vec![(q, ptop), (q, bottom)], color: c, width: rl.width, dash: rl.dashed });
            let room = q + 3.0 + width(&rl.label, 8.5, false) <= right;
            text(ops, if room { q + 3.0 } else { q - 3.0 }, ptop + 10.0, &rl.label, 8.5, c, if room { Anchor::Start } else { Anchor::End }, false, false);
        } else {
            ops.push(Op::Line { pts: vec![(left, q), (right, q)], color: c, width: rl.width, dash: rl.dashed });
            text(ops, right - 4.0, q - 4.0, &rl.label, 8.5, c, Anchor::End, false, false);
        }
    }
    for nt in &p.notes {
        if ax.ok(nt.x) && ay.ok(nt.y) && inside(nt.x, &ax) && inside(nt.y, &ay) {
            text(ops, ax.px(nt.x) + if nt.anchor == Anchor::Start { 3.0 } else { 0.0 }, ay.px(nt.y) + 3.0, &nt.text, 8.0, rgb(&nt.color), nt.anchor, false, false);
        }
    }
    // axes: left and bottom spines, ticks and their labels
    ops.push(Op::Line { pts: vec![(left, ptop), (left, bottom), (right, bottom)], color: ink2, width: 0.8, dash: false });
    for (v, l) in &xt {
        let q = ax.px(*v);
        ops.push(Op::Line { pts: vec![(q, bottom), (q, bottom + 3.0)], color: ink2, width: 0.8, dash: false });
        if xtl { text(ops, q, bottom + 12.0, l, 8.0, ink2, Anchor::Middle, false, false); }
    }
    for (v, l) in &yt {
        let q = ay.px(*v);
        ops.push(Op::Line { pts: vec![(left - 3.0, q), (left, q)], color: ink2, width: 0.8, dash: false });
        text(ops, left - 5.0, q + 3.0, l, 8.0, ink2, Anchor::End, false, false);
    }
    if xtl { text(ops, (left + right) / 2.0, bottom + 25.0, &p.xlabel, 9.0, ink, Anchor::Middle, false, false); }
    text(ops, cx + 16.0, (ptop + bottom) / 2.0, &p.ylabel, 9.0, ink, Anchor::Middle, false, true);
    for (i, l) in p.title.lines().enumerate() { text(ops, left, cy + 11.0 + 13.0 * i as f64, l, 9.5, ink, Anchor::Start, true, false); }
    // legend
    if let (Some(corner), false) = (p.legend, legend.is_empty()) {
        let lw = 26.0 + legend.iter().map(|(l, ..)| width(l, 8.0, false)).fold(0.0, f64::max);
        let lh = 12.0 * legend.len() as f64 + 6.0;
        let (lx, ly) = match corner {
            Corner::UpperRight => (right - lw - 4.0, ptop + 4.0), Corner::UpperLeft => (left + 4.0, ptop + 4.0),
            Corner::LowerRight => (right - lw - 4.0, bottom - lh - 4.0), Corner::LowerLeft => (left + 4.0, bottom - lh - 4.0),
        };
        ops.push(rect((lx, ly, lw, lh), Some(rgb(SURF)), Some((grid, 0.6))));
        for (i, (l, c, s)) in legend.iter().enumerate() {
            let y = ly + 9.0 + 12.0 * i as f64;
            match s.kind {
                Kind::Line | Kind::Step => {
                    ops.push(Op::Line { pts: vec![(lx + 5.0, y - 3.0), (lx + 19.0, y - 3.0)], color: *c, width: s.width.max(1.2), dash: s.dashed });
                    if s.marker != Marker::None { marker(ops, s.marker, (lx + 12.0, y - 3.0), s.size, *c); }
                }
                Kind::Scatter => marker(ops, if s.marker == Marker::None { Marker::Circle } else { s.marker }, (lx + 12.0, y - 3.0), s.size.max(2.2), *c),
                _ => ops.push(rect((lx + 7.0, y - 7.0, 10.0, 8.0), Some(*c), None)),
            }
            text(ops, lx + 23.0, y, l, 8.0, ink, Anchor::Start, false, false);
        }
    }
}

#[cfg(test)]
mod t {
    use super::*;

    #[test]
    fn ticks_are_one_two_five_and_decades() {
        let v: Vec<f64> = lin_ticks(0.0, 10.0, 5).iter().map(|t| t.0).collect();
        assert_eq!(v, [0.0, 2.0, 4.0, 6.0, 8.0, 10.0]);
        let l: Vec<String> = lin_ticks(0.1, 0.7, 6).into_iter().map(|t| t.1).collect();
        assert_eq!(l, ["0.1", "0.2", "0.3", "0.4", "0.5", "0.6", "0.7"]);
        let l: Vec<String> = lin_ticks(0.0, 95.0, 5).into_iter().map(|t| t.1).collect();
        assert_eq!(l, ["0", "20", "40", "60", "80"]);
        let d: Vec<String> = log_ticks(1e-4, 20.0).into_iter().map(|t| t.1).collect();
        assert_eq!(d, ["1e-4", "0.001", "0.01", "0.1", "1", "10"]);
        assert!(log_ticks(1e-14, 1e-3).len() <= 8, "many decades are thinned");
        assert!(log_ticks(2.0, 9.0).len() >= 2, "inside one decade: 1-2-5");
        for (lo, hi) in [(-3.3, 117.0), (1e-9, 3e-9), (-180.0, 180.0), (0.0, 1.0)] {
            let t = lin_ticks(lo, hi, 6);
            assert!(t.len() >= 2 && t.len() <= 13 && t.iter().all(|x| x.0 >= lo - 1e-12 && x.0 <= hi + 1e-12), "{lo} {hi}: {t:?}");
        }
    }

    #[test]
    fn a_long_series_is_thinned_and_clipped() {
        let x: Vec<f64> = (0..17000).map(|i| i as f64).collect();
        let y: Vec<f64> = x.iter().map(|v| (v * 0.37).sin()).collect();
        let pg = render(&Figure::stack(720.0, 300.0, &[], vec![Panel::new("t").with(Series::line(&x, &y, S1)).ylim(-0.5, 0.5)]));
        let n: usize = pg.ops.iter().map(|o| if let Op::Line { pts, .. } = o { pts.len() } else { 0 }).sum();
        assert!(n < 8000, "{n} points drawn for 17000 samples");
        for o in &pg.ops { if let Op::Line { pts, .. } = o { assert!(pts.iter().all(|p| p.1 >= 0.0 && p.1 <= 300.0)); } }
    }

    #[test]
    fn a_log_axis_drops_what_is_not_positive() {
        let p = Panel::new("").logy().with(Series::line(&[0.0, 1.0, 2.0, 3.0], &[1.0, 0.0, -1.0, 10.0], S1));
        let (lo, hi) = range(&p, false);
        assert!(lo > 0.0 && lo < 1.0 && hi > 10.0, "{lo} {hi}");
        let pg = render(&Figure::stack(400.0, 300.0, &[], vec![p.with(Series::scatter(&[0.0, 1.0], &[-5.0, 0.0], S2))]));
        let dots: usize = pg.ops.iter().map(|o| if let Op::Dots { pts, .. } = o { pts.len() } else { 0 }).sum();
        assert_eq!(dots, 0, "nonpositive scatter points are dropped");
        // NaN breaks a line: two separate one-point runs draw nothing
        let q = Panel::new("").with(Series::line(&[0.0, 1.0, 2.0], &[1.0, f64::NAN, 2.0], S1));
        let pg = render(&Figure::stack(400.0, 300.0, &[], vec![q]));
        assert!(!pg.ops.iter().any(|o| matches!(o, Op::Line { color, .. } if *color == rgb(S1))));
    }

    #[test]
    fn clipping_keeps_the_inside() {
        let c = clip(&[(-10.0, 5.0), (20.0, 5.0)], (0.0, 0.0, 10.0, 10.0));
        assert_eq!(c, vec![vec![(0.0, 5.0), (10.0, 5.0)]]);
        assert!(clip(&[(-10.0, -5.0), (-1.0, -5.0)], (0.0, 0.0, 10.0, 10.0)).is_empty());
    }
}
