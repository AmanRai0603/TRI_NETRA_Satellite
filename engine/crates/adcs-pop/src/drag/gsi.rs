//! Gas-surface interaction panel coefficients: `+drag/sentman.m`, `cll.m`, `dria.m`,
//! `sesam.m`, `panelCoeffs.m`, `speedRatio.m`, `species.m`. env's method env_gas_surface,
//! generated from the design into `gen::gsi` (tools/engine_build.py); what is left here is the
//! crate's names, the species by name and the panel model as an enum.
use crate::gen::gsi as g;

/// `drag.species(name)`: molar mass [kg/kmol]; `None` for an unknown species.
pub fn species(name: &str) -> Option<f64> {
    let sp = match name {
        "O" => g::SPECIES_O,
        "O2" => g::SPECIES_O2,
        "N2" => g::SPECIES_N2,
        "He" => g::SPECIES_HE,
        "H" => g::SPECIES_H,
        "N" => g::SPECIES_N,
        "Ar" => g::SPECIES_AR,
        _ => return None,
    };
    Some(g::molar_mass(sp))
}

/// Molar masses of the DTM species order H, He, O, N2, O2, N.
pub fn m_dtm() -> [f64; 6] {
    g::dtm_molar_masses()
}

/// `drag.speedRatio(Vrel, T, Mmol)`: `s = Vrel / sqrt(2 R T / Mmol)`, R = 8314.462.
#[inline]
pub fn speed_ratio(vrel: f64, t: f64, mmol: f64) -> f64 {
    g::speed_ratio(vrel, t, mmol)
}

/// `drag.sentman(s, delta, aT, Tw, Talt)` -> (cp, ct): Sentman (1961) flat panel,
/// diffuse re-emission with energy accommodation `aT`.
pub fn sentman(s: f64, delta: f64, a_t: f64, tw: f64, talt: f64) -> (f64, f64) {
    g::sentman(s, delta, a_t, tw, talt)
}

/// `drag.cll(s, delta, sig_n, sig_t, Tw, Talt)` -> (cp, ct): Cercignani-Lampis-Lord /
/// Schaaf-Chambre with normal and tangential momentum accommodation.
pub fn cll(s: f64, delta: f64, sig_n: f64, sig_t: f64, tw: f64, talt: f64) -> (f64, f64) {
    g::cll(s, delta, sig_n, sig_t, tw, talt)
}

/// `drag.sesam(nO, T)`: energy accommodation `x/(1+x)`, `x = 7.5e-17 nO T`.
#[inline]
pub fn sesam(n_o: f64, t: f64) -> f64 {
    g::sesam(n_o, t)
}

/// `drag.dria(s, delta, nO, T, Tw)`: Sentman with the SESAM accommodation.
pub fn dria(s: f64, delta: f64, n_o: f64, t: f64, tw: f64) -> (f64, f64) {
    g::dria(s, delta, n_o, t, tw)
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

impl PanelModel {
    /// The design's choice PanelModel (`gen::gsi::PANELMODEL_*`).
    pub fn choice(self) -> i64 {
        match self {
            PanelModel::Sentman => g::PANELMODEL_SENTMAN,
            PanelModel::Dria => g::PANELMODEL_DRIA,
            PanelModel::Sesam => g::PANELMODEL_SESAM,
            PanelModel::Cll => g::PANELMODEL_CLL,
        }
    }
}

/// The `gsi` struct: wall temperature, energy accommodation (sentman), momentum
/// accommodations (cll). Default = forces.drag's (the design's `gen::gsi::gsi_default`).
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
        Gsi::from(g::gsi_default())
    }
}

impl From<g::Gsi> for Gsi {
    fn from(x: g::Gsi) -> Self {
        Gsi { tw: x.tw, a_t: x.a_t, sig_n: x.sig_n, sig_t: x.sig_t }
    }
}

impl Gsi {
    /// The design's record.
    pub fn rec(&self) -> g::Gsi {
        g::Gsi { tw: self.tw, a_t: self.a_t, sig_n: self.sig_n, sig_t: self.sig_t }
    }
}

/// `drag.panelCoeffs(model, s, delta, gsi)` with `gsi.Talt`/`gsi.T` = `talt` and
/// `gsi.nO` = `n_o`.
pub fn panel_coeffs(model: PanelModel, s: f64, delta: f64, gsi: &Gsi, talt: f64, n_o: f64) -> (f64, f64) {
    g::panel_coeffs(model.choice(), s, delta, gsi.rec(), talt, n_o)
}
