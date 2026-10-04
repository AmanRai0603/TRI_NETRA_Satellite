//! Text as both backends set it: Helvetica's widths (the base-14 metrics; Arial, the SVG's
//! fallback, has the same) and the WinAnsi bytes the PDF writes, with the characters our titles
//! use mapped to their WinAnsi glyph (°, ², ·, —) or a close ASCII spelling (ω as w, ρ as rho).
//! Owner: Agastya. Copyright (c) 2026 Agastya. All rights reserved.

/// Widths of ' ' .. '~' in 1/1000 em, Helvetica and Helvetica-Bold.
const REG: [u16; 95] = [
    278, 278, 355, 556, 556, 889, 667, 191, 333, 333, 389, 584, 278, 333, 278, 278, 556, 556, 556, 556, 556, 556, 556, 556,
    556, 556, 278, 278, 584, 584, 584, 556, 1015, 667, 667, 722, 722, 667, 611, 778, 722, 278, 500, 667, 556, 833, 722, 778,
    667, 778, 722, 667, 611, 722, 667, 944, 667, 667, 611, 278, 278, 278, 469, 556, 333, 556, 556, 500, 556, 556, 278, 556,
    556, 222, 222, 500, 222, 833, 556, 556, 556, 556, 333, 500, 278, 556, 500, 722, 500, 500, 500, 334, 260, 334, 584,
];
const BOLD: [u16; 95] = [
    278, 333, 474, 556, 556, 889, 722, 238, 333, 333, 389, 584, 278, 333, 278, 278, 556, 556, 556, 556, 556, 556, 556, 556,
    556, 556, 333, 333, 584, 584, 584, 611, 975, 722, 722, 722, 722, 667, 611, 778, 722, 278, 556, 722, 611, 833, 722, 778,
    667, 778, 722, 667, 611, 722, 667, 944, 667, 667, 611, 333, 278, 333, 584, 556, 333, 556, 611, 556, 611, 556, 333, 611,
    611, 278, 278, 556, 278, 889, 611, 611, 611, 611, 389, 556, 333, 611, 556, 778, 556, 556, 500, 389, 280, 389, 584,
];

/// The WinAnsi bytes of `s`; a character WinAnsi lacks becomes its ASCII spelling, or '?'.
pub fn win_ansi(s: &str) -> Vec<u8> {
    let mut o = Vec::with_capacity(s.len());
    for c in s.chars() {
        let u = c as u32;
        let b: &[u8] = match c {
            _ if (32..127).contains(&u) => { o.push(u as u8); continue }
            _ if (0xA0..=0xFF).contains(&u) => { o.push(u as u8); continue }
            '\t' | '\n' | '\r' => b" ",
            '—' => &[0x97], '–' => &[0x96], '•' => &[0x95], '…' => &[0x85], '‘' => &[0x91], '’' => &[0x92],
            '“' => &[0x93], '”' => &[0x94], '€' => &[0x80], '™' => &[0x99], '†' => &[0x86], '‰' => &[0x89],
            '−' => b"-", '≤' => b"<=", '≥' => b">=", '≈' => b"~", '→' => b"->", '←' => b"<-", '√' => b"sqrt",
            '✔' | '✓' => b"ok", '✖' | '✗' => b"x", '∞' => b"inf", '∑' => b"sum", '∆' | 'Δ' => b"D",
            'α' => b"alpha", 'β' => b"beta", 'γ' => b"gamma", 'δ' => b"delta", 'ε' => b"eps", 'θ' => b"theta",
            'λ' => b"lambda", 'μ' => &[0xB5], 'ν' => b"nu", 'π' => b"pi", 'ρ' => b"rho", 'σ' => b"sigma",
            'τ' => b"tau", 'φ' => b"phi", 'ψ' => b"psi", 'ω' => b"w", 'Ω' => b"Ohm", 'η' => b"eta", 'κ' => b"kappa",
            '₀'..='₉' => { o.push(b'0' + (u - 0x2080) as u8); continue }
            '⁰' => b"0", '¹' => &[0xB9], '⁴'..='⁹' => { o.push(b'0' + (u - 0x2070) as u8); continue }
            '⁻' => b"-",
            _ => b"?",
        };
        o.extend_from_slice(b);
    }
    o
}

fn byte_width(b: u8, bold: bool) -> u16 {
    match b {
        32..=126 => if bold { BOLD[(b - 32) as usize] } else { REG[(b - 32) as usize] },
        0x97 | 0x89 => 1000, 0x96 | 0x80 | 0xB5 => 556, 0x95 => 350, 0x85 => 1000, 0x91..=0x94 => 333,
        0xB0 => 400, 0xB7 => 278, 0xB2 | 0xB3 | 0xB9 => 333, 0xD7 | 0xB1 => 584, 0xA0 => 278,
        _ => 556,
    }
}

/// The width of `s` set in Helvetica (bold or not) at `size` points.
pub fn width(s: &str, size: f64, bold: bool) -> f64 {
    win_ansi(s).iter().map(|&b| byte_width(b, bold) as f64).sum::<f64>() * size / 1000.0
}

#[cfg(test)]
mod t {
    use super::*;

    #[test]
    fn titles_map_to_win_ansi() {
        assert_eq!(win_ansi("10°"), b"10\xB0");
        assert_eq!(win_ansi("|ω| [deg/s]"), b"|w| [deg/s]");
        assert_eq!(win_ansi("A m² · ρ — x"), b"A m\xB2 \xB7 rho \x97 x");
        assert!(width("detumble", 10.0, false) > 30.0 && width("detumble", 10.0, true) > width("detumble", 10.0, false));
    }
}
