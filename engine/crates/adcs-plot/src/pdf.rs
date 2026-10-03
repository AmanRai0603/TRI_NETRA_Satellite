//! Drawings as one PDF 1.4 file, written by hand: a page per drawing, vector paths, text in the
//! Helvetica and Helvetica-Bold base-14 fonts with WinAnsi encoding (nothing embedded), content
//! streams uncompressed, and a cross-reference table whose offsets are the objects' byte positions.
//! Owner: Agastya. Copyright (c) 2026 Agastya. All rights reserved.
use crate::draw::{Op, Page, Rgb, P};
use crate::text::{width, win_ansi};
use crate::Anchor;
use std::fmt::Write;

fn n(v: f64) -> String {
    let s = format!("{:.2}", v);
    let s = s.trim_end_matches('0').trim_end_matches('.').to_string();
    if s == "-0" || s.is_empty() { "0".into() } else { s }
}

fn col(c: Rgb) -> String { format!("{} {} {}", n(c.0 as f64 / 255.0), n(c.1 as f64 / 255.0), n(c.2 as f64 / 255.0)) }

/// A PDF string literal of WinAnsi bytes: ( ) \ escaped, bytes outside printable ASCII in octal.
fn lit(s: &str) -> String {
    let mut o = String::from("(");
    for b in win_ansi(s) {
        match b {
            b'(' | b')' | b'\\' => { o.push('\\'); o.push(b as char); }
            32..=126 => o.push(b as char),
            _ => { let _ = write!(o, "\\{:03o}", b); }
        }
    }
    o.push(')');
    o
}

fn content(pg: &Page) -> String {
    let h = pg.h;
    let pt = |p: &P| format!("{} {}", n(p.0), n(h - p.1));
    let mut s = String::from("1 J 1 j\n");
    let poly = |s: &mut String, pts: &[P], close: bool| {
        for (i, p) in pts.iter().enumerate() { let _ = writeln!(s, "{} {}", pt(p), if i == 0 { "m" } else { "l" }); }
        if close { s.push_str("h\n"); }
    };
    for op in &pg.ops {
        match op {
            Op::Line { pts, color, width, dash } => {
                if pts.len() < 2 { continue; }
                let _ = writeln!(s, "{} RG {} w {}", col(*color), n(*width), if *dash { "[5 3] 0 d" } else { "[] 0 d" });
                poly(&mut s, pts, false);
                s.push_str("S\n");
            }
            Op::Shape { pts, fill, stroke } => {
                if let Some(f) = fill { let _ = writeln!(s, "{} rg", col(*f)); }
                if let Some((c, w)) = stroke { let _ = writeln!(s, "{} RG {} w [] 0 d", col(*c), n(*w)); }
                poly(&mut s, pts, true);
                s.push_str(match (fill.is_some(), stroke.is_some()) { (true, true) => "B\n", (true, false) => "f\n", (false, true) => "S\n", _ => "n\n" });
            }
            Op::Dots { pts, r, color } => {
                if pts.is_empty() { continue; }
                let _ = writeln!(s, "{} RG {} w [] 0 d", col(*color), n(2.0 * r));
                for p in pts { let q = pt(p); let _ = writeln!(s, "{q} m {q} l"); }
                s.push_str("S\n");
            }
            Op::Circle { c, r, fill, stroke } => {
                let k = 0.5523 * r;
                let (x, y) = (c.0, h - c.1);
                if let Some(f) = fill { let _ = writeln!(s, "{} rg", col(*f)); }
                if let Some((c, w)) = stroke { let _ = writeln!(s, "{} RG {} w [] 0 d", col(*c), n(*w)); }
                let _ = writeln!(s, "{} {} m", n(x + r), n(y));
                for (a, b, e) in [((x + r, y + k), (x + k, y + r), (x, y + r)), ((x - k, y + r), (x - r, y + k), (x - r, y)),
                                  ((x - r, y - k), (x - k, y - r), (x, y - r)), ((x + k, y - r), (x + r, y - k), (x + r, y))] {
                    let _ = writeln!(s, "{} {} {} {} {} {} c", n(a.0), n(a.1), n(b.0), n(b.1), n(e.0), n(e.1));
                }
                s.push_str(match (fill.is_some(), stroke.is_some()) { (true, true) => "b\n", (true, false) => "f\n", (false, true) => "s\n", _ => "n\n" });
            }
            Op::Text { x, y, s: t, size, color, anchor, bold, up } => {
                let w = width(t, *size, *bold);
                let d = match anchor { Anchor::Start => 0.0, Anchor::Middle => w / 2.0, Anchor::End => w };
                let (x, y) = (*x, h - *y);
                let m = if *up { format!("0 1 -1 0 {} {}", n(x), n(y - d)) } else { format!("1 0 0 1 {} {}", n(x - d), n(y)) };
                let _ = writeln!(s, "BT /{} {} Tf {} rg {m} Tm {} Tj ET", if *bold { "F2" } else { "F1" }, n(*size), col(*color), lit(t));
            }
        }
    }
    s
}

/// The pages as one PDF file.
pub fn pdf(pages: &[Page]) -> Vec<u8> {
    // objects 1 catalog, 2 page tree, 3-4 the fonts, then each page and its content stream
    let mut objs: Vec<Vec<u8>> = Vec::new();
    let kids: Vec<String> = (0..pages.len()).map(|i| format!("{} 0 R", 5 + 2 * i)).collect();
    objs.push(b"<< /Type /Catalog /Pages 2 0 R >>".to_vec());
    objs.push(format!("<< /Type /Pages /Kids [{}] /Count {} >>", kids.join(" "), pages.len()).into_bytes());
    for f in ["Helvetica", "Helvetica-Bold"] {
        objs.push(format!("<< /Type /Font /Subtype /Type1 /BaseFont /{f} /Encoding /WinAnsiEncoding >>").into_bytes());
    }
    for (i, pg) in pages.iter().enumerate() {
        objs.push(format!("<< /Type /Page /Parent 2 0 R /MediaBox [0 0 {} {}] /Resources << /Font << /F1 3 0 R /F2 4 0 R >> >> /Contents {} 0 R >>",
                          n(pg.w), n(pg.h), 6 + 2 * i).into_bytes());
        let c = content(pg);
        let mut o = format!("<< /Length {} >>\nstream\n", c.len()).into_bytes();
        o.extend_from_slice(c.as_bytes());
        o.extend_from_slice(b"\nendstream");
        objs.push(o);
    }
    let mut out: Vec<u8> = b"%PDF-1.4\n%\xe2\xe3\xcf\xd3\n".to_vec();
    let mut at = Vec::with_capacity(objs.len());
    for (i, o) in objs.iter().enumerate() {
        at.push(out.len());
        out.extend_from_slice(format!("{} 0 obj\n", i + 1).as_bytes());
        out.extend_from_slice(o);
        out.extend_from_slice(b"\nendobj\n");
    }
    let xref = out.len();
    let mut x = format!("xref\n0 {}\n0000000000 65535 f \n", objs.len() + 1);
    for a in at { let _ = writeln!(x, "{a:010} 00000 n "); }
    let _ = write!(x, "trailer\n<< /Size {} /Root 1 0 R >>\nstartxref\n{xref}\n%%EOF\n", objs.len() + 1);
    out.extend_from_slice(x.as_bytes());
    out
}

#[cfg(test)]
pub(crate) mod t {
    use crate::*;

    /// Every xref entry points at its "n 0 obj", and startxref at the table.
    pub fn check(b: &[u8]) -> Result<usize, String> {
        if !b.starts_with(b"%PDF-1.4\n") { return Err("no %PDF- header".into()); }
        if !b.ends_with(b"%%EOF\n") { return Err("no %%EOF".into()); }
        let s: String = b.iter().map(|&c| if c < 128 { c as char } else { '?' }).collect();    // byte offsets kept
        let sx = s.rfind("startxref\n").ok_or("no startxref")?;
        let at: usize = s[sx + 10..].lines().next().unwrap().trim().parse().map_err(|_| "bad startxref")?;
        if !s.as_bytes()[at..].starts_with(b"xref\n") { return Err("startxref does not point at xref".into()); }
        let mut lines = s[at..].lines().skip(1);
        let n: usize = lines.next().unwrap().split_whitespace().nth(1).unwrap().parse().unwrap();
        let entries: Vec<&str> = lines.take(n).collect();
        for (i, e) in entries.iter().enumerate().skip(1) {
            let off: usize = e[..10].parse().map_err(|_| format!("bad offset {e}"))?;
            if !b[off..].starts_with(format!("{i} 0 obj").as_bytes()) { return Err(format!("object {i} is not at {off}")); }
        }
        // every stream's /Length is its byte count
        let mut k = 0;
        while let Some(p) = s[k..].find("/Length ") {
            let p = k + p + 8;
            let len: usize = s[p..].split_whitespace().next().unwrap().parse().unwrap();
            let st = s[p..].find("stream\n").unwrap() + p + 7;
            if &b[st + len..st + len + 10] != b"\nendstream" { return Err(format!("stream at {st}: /Length {len} is wrong")); }
            k = st + len;
        }
        Ok(n)
    }

    #[test]
    fn the_pdf_has_its_header_trailer_and_true_offsets() {
        let x: Vec<f64> = (0..300).map(|i| i as f64).collect();
        let y: Vec<f64> = x.iter().map(|v| v.sin()).collect();
        let a = Figure::stack(720.0, 446.0, &[], vec![Panel::new("rate (ω) — 10°, A m²").ylabel("(x) \\ y").with(Series::line(&x, &y, S1).label("ωx"))
            .with(Series::scatter(&x, &y, S2)).refl(RefLine::h(0.5, "req")).legend(Corner::LowerLeft)]);
        let b = Figure::grid(720.0, 300.0, 1, 2, vec![Panel::new("h").with(Series::hist_values(&y, 9, S1)),
            Panel::new("bars").with(Series::hbar(&["a", "b"], &[1.0, f64::NAN], S1))]);
        let one = a.to_pdf();
        assert_eq!(check(&one), Ok(7));
        let two = pdf_of(&[a, b]);
        assert_eq!(check(&two), Ok(9), "two pages, two more objects");
        let s: String = two.iter().map(|&c| c as char).collect();
        assert!(s.contains("/Count 2") && s.contains("/BaseFont /Helvetica ") && s.contains("/WinAnsiEncoding"));
        assert!(s.contains("(rate \\(w\\) \\227 10\\260, A m\\262)"), "WinAnsi text with escapes");
    }
}
