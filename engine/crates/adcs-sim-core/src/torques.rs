//! Environment torques from the orbit state: env's methods, generated from the design (tools/engine_build.py): the box's
//! faces and the aerodynamic torque (`gen::facets`, l3_dist_row_02), the gravity gradient (`gen::gravgrad`,
//! l3_dist_row_01), the radiation of the Sun and the Earth's albedo and infrared (`gen::radiation`, `gen::albedo`,
//! `gen::earthir`, l3_dist_row_03 to 05) and the residual dipole (`gen::dipoletorque`, l3_dist_row_06). What is left
//! here is the step's call of them, by the names the engine has always used: the attitude matrix (the toolbox's) and
//! which parts the case switches on.
use crate::gen::{albedo, dipoletorque, earthir, facets, gravgrad, radiation as rad};
use crate::la::*;

/// The faces of the box: outward normals, areas, arms from the centre of mass, and the surface's constants.
pub use crate::gen::facets::Facets;

impl Facets {
    /// A box of edges box_m [m] with the centre of mass offset cm [m].
    pub fn boxed(box_m: &V3, cm: &V3, sigma_n: f64, sigma_t: f64, vb_ratio: f64, refl: f64, spec_frac: f64) -> Facets {
        facets::facets_box(*box_m, *cm, sigma_n, sigma_t, vb_ratio, refl, spec_frac)
    }
}

/// The Earth's mean Bond albedo and its mean emitted flux [W/m^2], as the engine flies them (env's l3_dist_row_04 and
/// l3_dist_row_05).
pub const EARTH_ALBEDO: f64 = albedo::EARTH_ALBEDO;
pub const EARTH_IR_W_M2: f64 = earthir::EARTH_IR_FLUX;

/// The pressures [N/m^2] of the Earth's albedo and infrared on a plate facing the Earth's centre at r; `p_sun` is the
/// Sun's pressure at the satellite.
pub fn earth_pressure(r: &V3, sun_rel: &V3, p_sun: f64) -> (f64, f64) {
    (albedo::albedo_pressure(*r, *sun_rel, p_sun), earthir::earth_ir_pressure(*r))
}

/// The torque of light of pressure p arriving from body direction `sb` (unit, towards the source) on the facets.
pub fn radiation(g: &Facets, sb: &V3, p: f64) -> V3 { rad::radiation_torque(*g, *sb, p) }

/// [gg, aero, radiation (Sun, albedo, Earth IR), mag] torques, body frame.
pub fn torques(q: &Q, r: &V3, v_rel: &V3, b_eci: &V3, sun_rel: &V3, nu: f64, p_srp: f64, rho: f64, i: &M3, g: &Facets, m_res: &V3, mu: f64, on: [bool; 4]) -> [V3; 4] {
    let rm = dcm(q);
    let mut out = [[0.0; 3]; 4];
    if on[0] { out[0] = gravgrad::gravity_gradient_torque(rm, *r, *i, mu); }
    if on[1] { out[1] = facets::aero_torque(rm, *v_rel, rho, *g); }
    if on[2] { out[2] = rad::light_torque(rm, *r, *sun_rel, nu, p_srp, *g); }
    if on[3] { out[3] = dipoletorque::residual_dipole_torque(rm, *m_res, *b_eci); }
    out
}
