//! `+atmos/exponential.m`: the Vallado piecewise-exponential density (altitude only,
//! no space weather), with its coarse temperature and mean molar mass: env's method env_exponential_atmosphere over
//! env's one Vallado table (env.atmosphere), generated from the design into `gen::expatmos` (tools/engine_build.py).

/// Avogadro constant [1/mol] (`de440.constants().N_A`; CODATA 2018, exact): the design's.
pub use crate::gen::expatmos::N_AVOGADRO as N_A;

/// Result of [`exponential`]: `rho` [kg/m^3], `t` [K], `mmol` [kg/kmol], `n_o` [m^-3]
/// (total number density treated as atomic O for the GSI models).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ExpAtm {
    /// density [kg/m^3]
    pub rho: f64,
    /// temperature [K]
    pub t: f64,
    /// mean molar mass [kg/kmol]
    pub mmol: f64,
    /// number density treated as atomic O [m^-3]
    pub n_o: f64,
}

/// `meanMolarMass(h)`: crude N2/O2 -> O -> H transition [kg/kmol].
pub use crate::gen::expatmos::mean_molar_mass;

/// `atmos.exponential(alt_km)`.
pub fn exponential(alt_km: f64) -> ExpAtm {
    let (rho, t, mmol, n_o) = crate::gen::expatmos::exponential_atm(alt_km);
    ExpAtm { rho, t, mmol, n_o }
}
