//! Gas-surface interaction panel coefficients: `+drag/sentman.m`, `cll.m`, `dria.m`,
//! `sesam.m`, `panelCoeffs.m`, `speedRatio.m`, `species.m`.
use crate::atmos::octave::{erf, pw};
use std::f64::consts::PI;

/// `drag.species(name)`: molar mass [kg/kmol]; `None` for an unknown species.
pub fn species(name: &str) -> Option<f64> {
    match name {
        "O" => Some(15.9994),
        "O2" => Some(31.9988),
        "N2" => Some(28.0134),
        "He" => Some(4.0026),
        "H" => Some(1.0079),
        "N" => Some(14.0067),
        "Ar" => Some(39.948),
        _ => None,
    }
}

/// Molar masses of the DTM species order H, He, O, N2, O2, N.
pub const M_DTM: [f64; 6] = [1.0079, 4.0026, 15.9994, 28.0134, 31.9988, 14.0067];

/// `drag.speedRatio(Vrel, T, Mmol)`: `s = Vrel / sqrt(2 R T / Mmol)`, R = 8314.462.
#[inline]
pub fn speed_ratio(vrel: f64, t: f64, mmol: f64) -> f64 {
    let r = 8314.462;
    vrel / (2.0 * r * t / mmol).sqrt()
}

/// `drag.sentman(s, delta, aT, Tw, Talt)` -> (cp, ct): Sentman (1961) flat panel,
/// diffuse re-emission with energy accommodation `aT`.
pub fn sentman(s: f64, delta: f64, a_t: f64, tw: f64, talt: f64) -> (f64, f64) {
    let c = delta.cos();
    let sn = s * c;
    let ti = (2.0 / 3.0) * pw(s, 2.0) * talt;
    let e = 1.0 + erf(sn);
    let p = (-pw(sn, 2.0)).exp();
    let sqpi = PI.sqrt();
    let cp = c / (sqpi * s) * p + (1.0 / (2.0 * pw(s, 2.0)) + pw(c, 2.0)) * e
        + 0.5 * ((2.0 / 3.0) * (1.0 + a_t * (tw / ti - 1.0))).sqrt() * (sqpi * c * e + (1.0 / s) * p);
    let ct = delta.sin() / (sqpi * s) * (p + sqpi * sn * e);
    (cp, ct)
}

/// `drag.cll(s, delta, sig_n, sig_t, Tw, Talt)` -> (cp, ct): Cercignani-Lampis-Lord /
/// Schaaf-Chambre with normal and tangential momentum accommodation.
pub fn cll(s: f64, delta: f64, sig_n: f64, sig_t: f64, tw: f64, talt: f64) -> (f64, f64) {
    let c = delta.cos();
    let sn = s * c;
    let e = 1.0 + erf(sn);
    let p = (-pw(sn, 2.0)).exp();
    let vr = (tw / talt).sqrt();
    let sqpi = PI.sqrt();
    let ip = c / (sqpi * s) * p + (1.0 / (2.0 * pw(s, 2.0)) + pw(c, 2.0)) * e;
    let dp = (vr / (2.0 * s)) * (sqpi * c * e + (1.0 / s) * p);
    let st = delta.sin() / (sqpi * s) * (p + sqpi * sn * e);
    ((2.0 - sig_n) * ip + sig_n * dp, sig_t * st)
}

/// `drag.sesam(nO, T)`: energy accommodation `x/(1+x)`, `x = 7.5e-17 nO T`.
#[inline]
pub fn sesam(n_o: f64, t: f64) -> f64 {
    let x = 7.5e-17 * n_o * t;
    x / (1.0 + x)
}

/// `drag.dria(s, delta, nO, T, Tw)`: Sentman with the SESAM accommodation.
pub fn dria(s: f64, delta: f64, n_o: f64, t: f64, tw: f64) -> (f64, f64) {
    let a_t = sesam(n_o, t);
    sentman(s, delta, a_t, tw, t)
}

/// Panel (free-molecular) GSI models of drag.force / drag.panelCoeffs.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PanelModel {
    /// Sentman with a typed `aT`
    Sentman,
    /// Sentman with `aT` from SESAM (DRIA)
    Dria,
    /// alias of Dria in drag.force
    Sesam,
    /// CLL with `sig_n`, `sig_t`
    Cll,
}

/// The `gsi` struct: wall temperature, energy accommodation (sentman), momentum
/// accommodations (cll). Default = forces.drag's `struct('Tw',300,'aT',0.9,'sig_n',0.9,'sig_t',0.9)`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Gsi {
    /// wall temperature [K] (`Tw`)
    pub tw: f64,
    /// energy accommodation (`aT`, sentman)
    pub a_t: f64,
    /// normal momentum accommodation (`sig_n`, cll)
    pub sig_n: f64,
    /// tangential momentum accommodation (`sig_t`, cll)
    pub sig_t: f64,
}

impl Default for Gsi {
    fn default() -> Self {
        Gsi { tw: 300.0, a_t: 0.9, sig_n: 0.9, sig_t: 0.9 }
    }
}

/// `drag.panelCoeffs(model, s, delta, gsi)` with `gsi.Talt`/`gsi.T` = `talt` and
/// `gsi.nO` = `n_o`.
pub fn panel_coeffs(model: PanelModel, s: f64, delta: f64, gsi: &Gsi, talt: f64, n_o: f64) -> (f64, f64) {
    match model {
        PanelModel::Sentman => sentman(s, delta, gsi.a_t, gsi.tw, talt),
        PanelModel::Dria => dria(s, delta, n_o, talt, gsi.tw),
        PanelModel::Cll => cll(s, delta, gsi.sig_n, gsi.sig_t, gsi.tw, talt),
        PanelModel::Sesam => sentman(s, delta, sesam(n_o, talt), gsi.tw, talt),
    }
}
