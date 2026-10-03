//! A drawing as one self-contained SVG: a viewBox in points, Helvetica or the system sans-serif
//! (no font files), every number to a tenth of a point. It inlines into an HTML page as it is.
//! Owner: Agastya. Copyright (c) 2026 Agastya. All rights reserved.
use crate::draw::{Op, Page, Rgb, P};
use crate::Anchor;
use std::fmt::Write;

fn n(v: f64) -> String {
    let s = format!("{:.1}", v);
    let s = s.strip_suffix(".0").unwrap_or(&s).to_string();
    if s == "-0" { "0".into() } else { s }
}

fn hex(c: Rgb) -> String { format!("#{:02x}{:02x}{:02x}", c.0, c.1, c.2) }

/// `s` with the five XML specials escaped.
pub fn esc(s: &str) -> String {
    s.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;").replace('"', "&quot;").replace('\'', "&#39;")
}

fn path(pts: &[P], close: bool) -> String {
    let mut d = String::with_capacity(pts.len() * 10);
    for (i, p) in pts.iter().enumerate() { let _ = write!(d, "{}{} {}", if i == 0 { "M" } else { "L" }, n(p.0), n(p.1)); }
    if close { d.push('Z'); }
    d
}

pub fn svg(pg: &Page) -> String {
    let mut s = format!("<svg xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"0 0 {w} {h}\" width=\"{w}\" height=\"{h}\" \
                         font-family=\"Helvetica, Arial, 'Liberation Sans', sans-serif\" role=\"img\">\n", w = n(pg.w), h = n(pg.h));
    for op in &pg.ops {
        match op {
            Op::Line { pts, color, width, dash } => {
                let _ = writeln!(s, "<path d=\"{}\" fill=\"none\" stroke=\"{}\" stroke-width=\"{}\" stroke-linejoin=\"round\" stroke-linecap=\"round\"{}/>",
                                 path(pts, false), hex(*color), n(*width), if *dash { " stroke-dasharray=\"5 3\"" } else { "" });
            }
            Op::Shape { pts, fill, stroke } => {
                let f = fill.map(hex).unwrap_or("none".into());
                let st = stroke.map(|(c, w)| format!(" stroke=\"{}\" stroke-width=\"{}\"", hex(c), n(w))).unwrap_or_default();
                let _ = writeln!(s, "<path d=\"{}\" fill=\"{f}\"{st}/>", path(pts, true));
            }
            Op::Dots { pts, r, color } => {
                if pts.is_empty() { continue; }
                let mut d = String::with_capacity(pts.len() * 12);
                for p in pts { let _ = write!(d, "M{} {}h0", n(p.0), n(p.1)); }
                let _ = writeln!(s, "<path d=\"{d}\" fill=\"none\" stroke=\"{}\" stroke-width=\"{}\" stroke-linecap=\"round\"/>", hex(*color), n(2.0 * r));
            }
            Op::Circle { c, r, fill, stroke } => {
                let f = fill.map(hex).unwrap_or("none".into());
                let st = stroke.map(|(c, w)| format!(" stroke=\"{}\" stroke-width=\"{}\"", hex(c), n(w))).unwrap_or_default();
                let _ = writeln!(s, "<circle cx=\"{}\" cy=\"{}\" r=\"{}\" fill=\"{f}\"{st}/>", n(c.0), n(c.1), n(*r));
            }
            Op::Text { x, y, s: t, size, color, anchor, bold, up } => {
                let a = match anchor { Anchor::Start => "", Anchor::Middle => " text-anchor=\"middle\"", Anchor::End => " text-anchor=\"end\"" };
                let b = if *bold { " font-weight=\"bold\"" } else { "" };
                let r = if *up { format!(" transform=\"rotate(-90 {} {})\"", n(*x), n(*y)) } else { String::new() };
                let _ = writeln!(s, "<text x=\"{}\" y=\"{}\" font-size=\"{}\" fill=\"{}\"{a}{b}{r} xml:space=\"preserve\">{}</text>",
                                 n(*x), n(*y), n(*size), hex(*color), esc(t));
            }
        }
    }
    s.push_str("</svg>\n");
    s
}

#[cfg(test)]
pub(crate) mod t {
    use crate::*;

    /// A small XML well-formedness check: every tag closes in order, attributes are quoted,
    /// text holds no bare '<' or '&'.
    pub fn well_formed(s: &str) -> Result<usize, String> {
        let mut stack: Vec<String> = Vec::new();
        let mut i = 0;
        let b = s.as_bytes();
        let mut tags = 0;
        while i < b.len() {
            if b[i] == b'<' {
                let j = s[i..].find('>').ok_or("unclosed <")? + i;
                let tag = &s[i + 1..j];
                if let Some(name) = tag.strip_prefix('/') {
                    if stack.pop().as_deref() != Some(name.trim()) { return Err(format!("</{name}> closes the wrong tag")); }
                } else {
                    let selfc = tag.ends_with('/');
                    let body = tag.trim_end_matches('/');
                    let name = body.split_whitespace().next().ok_or("empty tag")?;
                    if !body.matches('"').count().is_multiple_of(2) { return Err(format!("unbalanced quotes in <{name}>")); }
                    if !selfc { stack.push(name.to_string()); }
                    tags += 1;
                }
                i = j + 1;
            } else {
                if b[i] == b'&' && !["&amp;", "&lt;", "&gt;", "&quot;", "&#39;"].iter().any(|e| s[i..].starts_with(e)) { return Err("bare &".into()); }
                i += 1;
            }
        }
        if stack.is_empty() { Ok(tags) } else { Err(format!("unclosed {stack:?}")) }
    }

    #[test]
    fn the_svg_is_well_formed_and_self_contained() {
        let x: Vec<f64> = (0..500).map(|i| i as f64 / 10.0).collect();
        let y: Vec<f64> = x.iter().map(|v| 1e-3 * (1.0 + v)).collect();
        let f = Figure::stack(720.0, 400.0, &[2.0, 1.0], vec![
            Panel::new("a <title> & more\nsecond line").logy().ylabel("|ω| [deg/s]").with(Series::line(&x, &y, S1).label("rate")).refl(RefLine::h(0.01, "req 0.01")).legend(Corner::UpperRight),
            Panel::new("modes").with(Series::step(&x, &x.iter().map(|v| (v / 10.0).floor()).collect::<Vec<_>>(), S2)).xlabel("time [min]"),
        ]).titled("fig \"quoted\"");
        let s = f.to_svg();
        assert!(s.starts_with("<svg xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"0 0 720 400\""));
        assert!(well_formed(&s).unwrap() > 20, "{}", well_formed(&s).unwrap_err());
        assert!(!s.contains("href") && !s.contains("@import") && !s.contains("<script"), "no external reference");
        assert!(s.contains("a &lt;title&gt; &amp; more") && s.contains("req 0.01"));
    }
}
