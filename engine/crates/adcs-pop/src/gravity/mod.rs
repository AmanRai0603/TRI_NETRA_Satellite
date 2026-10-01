//! Central-body gravity: ports of `matlab_sils/pop/01_core/+grav/*`
//! (`defaultField`, `loadGFC`, `sphericalHarmonic`, `potential`, `j2accel`,
//! `twoBody`), `01_core/+op/gravLoad.m` and `02_forces/+forces/gravity.m`.
//!
//! Frames follow the MATLAB: the field is evaluated in ECEF (`r_ecef = C * r_eci`,
//! `C = ctx.C` from `frames.eci2ecef`) and rotated back with `ctx.Ct = C'`.
//! Arithmetic is kept in the MATLAB evaluation order, and `norm` is Octave's
//! scaled 2-norm ([`octave_norm`]), so results agree with Octave to the last bits.
//!
//! Owner: Agastya. Copyright (c) 2026 Agastya. All rights reserved.
mod field;
mod potential;
mod sphharm;
mod zonal;

pub use field::{grav_load, registry_file, Field, GravityFieldCfg};
pub use potential::{norm_legendre_pot, potential, potential_coeffs};
pub use sphharm::{accel_ecef, denorm_factor, SphHarm};
pub use zonal::{j2accel, two_body, J2Accel};

use crate::la::{mtv, mv, M3, V3};

/// MATLAB/Octave `norm(v)` of a 3-vector: the scaled accumulator of Octave's
/// `vector_norm` (`norm_accumulator_2`), which can differ from
/// `sqrt(x^2+y^2+z^2)` in the last bit. Used wherever the MATLAB calls `norm`.
pub fn octave_norm(v: &V3) -> f64 {
    let mut scl = 0.0f64;
    let mut sum = 1.0f64;
    for &x in v {
        let t = x.abs();
        if scl == t {
            sum += 1.0;
        } else if scl < t {
            let q = scl / t;
            sum *= q * q;
            sum += 1.0;
            scl = t;
        } else if t != 0.0 {
            let q = t / scl;
            sum += q * q;
        }
    }
    scl * sum.sqrt()
}

/// `cfg.forces.gravity.model` of `forces.gravity`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GravityModel {
    /// `'twobody'`: `grav.twoBody(r_eci, mu)` (no rotation).
    TwoBody,
    /// `'j2'..'j6'`: `grav.j2accel` with `fld.J(1:n-1)`; holds n (2..=6).
    Zonal(usize),
    /// `'sphharm'`: `grav.sphericalHarmonic` to `min(degree, nmax)` x `min(order, nmax)`.
    SphHarm,
}

impl GravityModel {
    /// Parse the MATLAB model string (case-insensitive). `'toolbox'` (the MATLAB
    /// Aerospace Toolbox `gravitysphericalharmonic`) is not portable and is refused.
    pub fn parse(s: &str) -> Result<GravityModel, crate::PopError> {
        let l = s.to_lowercase();
        match l.as_str() {
            "twobody" => Ok(GravityModel::TwoBody),
            "j2" | "j3" | "j4" | "j5" | "j6" => Ok(GravityModel::Zonal((l.as_bytes()[1] - b'0') as usize)),
            "sphharm" => Ok(GravityModel::SphHarm),
            "toolbox" => Err(crate::PopError::Unsupported("forces:gravity 'toolbox' needs the MATLAB Aerospace Toolbox; not ported (use 'sphharm')".into())),
            _ => Err(crate::PopError::Unsupported(format!("forces:gravity unknown gravity model \"{s}\""))),
        }
    }
}

/// `cfg.forces.gravity` (`.model .degree .order`) as read by `forces.gravity`.
#[derive(Clone, Copy, Debug)]
pub struct GravityForceCfg {
    /// `.model`.
    pub model: GravityModel,
    /// `.degree` (clamped to the field's nmax for `'sphharm'`).
    pub degree: usize,
    /// `.order`; `None` means "= degree" (MATLAB `getf(g,'order',nmax)`).
    pub order: Option<usize>,
}

impl Default for GravityForceCfg {
    /// `config.defaultConfig`: `struct('model','sphharm','degree',6,'order',6)`.
    fn default() -> Self {
        GravityForceCfg { model: GravityModel::SphHarm, degree: 6, order: Some(6) }
    }
}

/// Port of `forces.gravity(ctx)` with its field (`ctx.grav`) and preallocated
/// workspaces, so the per-step evaluation allocates nothing.
#[derive(Clone, Debug)]
pub struct Gravity {
    /// `ctx.grav` (from `op.gravLoad`).
    pub field: Field,
    /// `ctx.cfg.forces.gravity`.
    pub cfg: GravityForceCfg,
    sh: Option<SphHarm>,
    zonal: Option<J2Accel>,
    nmax: usize,
    mmax: usize,
}

impl Gravity {
    /// Build the force model; errors where the MATLAB would error at run time
    /// (`'jN'` with a field holding fewer than N-1 zonals).
    pub fn new(field: Field, cfg: GravityForceCfg) -> Result<Gravity, crate::PopError> {
        let (mut sh, mut zonal, mut nmax, mut mmax) = (None, None, 0, 0);
        match cfg.model {
            GravityModel::TwoBody => {}
            GravityModel::Zonal(nz) => {
                if field.j.len() < nz - 1 {
                    return Err(crate::PopError::Unsupported(format!("forces:gravity 'j{nz}' needs J2..J{nz} but the field has {} zonals", field.j.len())));
                }
                zonal = Some(J2Accel::new(&field.j[..nz - 1]));
            }
            GravityModel::SphHarm => {
                nmax = cfg.degree.min(field.nmax);
                mmax = cfg.order.unwrap_or(nmax).min(nmax);
                sh = Some(SphHarm::new(&field));
            }
        }
        Ok(Gravity { field, cfg, sh, zonal, nmax, mmax })
    }

    /// Gravity acceleration in ECEF for the configured model (for `'twobody'`
    /// the point mass, which is frame-independent).
    pub fn accel_ecef(&mut self, r_ecef: &V3) -> V3 {
        let (mu, re) = (self.field.gm, self.field.re);
        match self.cfg.model {
            GravityModel::TwoBody => two_body(r_ecef, mu),
            GravityModel::Zonal(_) => self.zonal.as_mut().unwrap().accel(r_ecef, mu, re),
            GravityModel::SphHarm => self.sh.as_mut().unwrap().accel(r_ecef, mu, re, self.nmax, self.mmax),
        }
    }

    /// `forces.gravity(ctx)`: ECI acceleration [m/s^2] from `r_eci` and
    /// `c_eci2ecef = ctx.C` (`r_ecef = C*r_eci`, result `= C' * a_ecef`).
    pub fn accel_eci(&mut self, r_eci: &V3, c_eci2ecef: &M3) -> V3 {
        if self.cfg.model == GravityModel::TwoBody {
            return two_body(r_eci, self.field.gm);
        }
        let r_ecef = mv(c_eci2ecef, r_eci);
        let a = self.accel_ecef(&r_ecef);
        mtv(c_eci2ecef, &a)
    }
}

/// One-shot `forces.gravity` (allocates the workspaces; use [`Gravity`] in loops).
pub fn accel_eci(f: &Field, r_eci: &V3, c_eci2ecef: &M3, cfg: &GravityForceCfg) -> Result<V3, crate::PopError> {
    Ok(Gravity::new(f.clone(), *cfg)?.accel_eci(r_eci, c_eci2ecef))
}
