//! JPL DE440 ephemeris API and the per-step ephemeris bundle -- port of
//! `matlab_sils/pop/03_frames_time/ephemeris/+de440/*` (`open`, `state`, `sun`,
//! `moon`, `earth`, `constants`) and `ephemeris/ephemInputs.m`.
//!
//! Frame ICRF (== GCRF/J2000 ECI to < 1 mas). Time: TDB Julian date (TT accepted,
//! < 1.7 ms). Units exactly as MATLAB: [`Ephem::state`] and [`Ephem::earth`] in
//! km, km/s; [`Ephem::sun`], [`Ephem::moon`] and [`EphemInputs`] in SI.

use crate::la::{norm, V3};
use crate::spk::{Kernel, SpkError};
use std::path::{Path, PathBuf};

/// Physical constants (SI) consistent with DE440 / IAU 2009 -- `de440.constants()`.
/// Field names are the MATLAB ones in snake case (`AU_m` -> `au_m`, `P0` -> `p0`, ...).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Constants {
    /// `AU_m` astronomical unit [m] (IAU 2012).
    pub au_m: f64,
    /// `c` speed of light [m/s].
    pub c: f64,
    /// `GM_sun` [m^3/s^2] (DE440).
    pub gm_sun: f64,
    /// `GM_earth` [m^3/s^2] (DE440, ephemeris-consistent).
    pub gm_earth: f64,
    /// `GM_moon` [m^3/s^2] (DE440).
    pub gm_moon: f64,
    /// `EMRAT` Earth/Moon mass ratio (DE440).
    pub emrat: f64,
    /// `TSI` mean total solar irradiance at 1 AU [W/m^2].
    pub tsi: f64,
    /// `P0 = TSI/c` solar radiation pressure at 1 AU [N/m^2].
    pub p0: f64,
    /// `Re_earth` WGS84 equatorial radius [m].
    pub re_earth: f64,
    /// `f_earth` WGS84 flattening.
    pub f_earth: f64,
    /// `Rp_earth = Re*(1-f)` polar radius [m].
    pub rp_earth: f64,
    /// `mu_earth` WGS84 GM [m^3/s^2] (propagation).
    pub mu_earth: f64,
    /// `omega_earth` nominal Earth rotation rate [rad/s].
    pub omega_earth: f64,
    /// `N_A` Avogadro constant [1/mol].
    pub n_a: f64,
    /// `Rsun` IAU 2015 nominal solar radius [m].
    pub rsun: f64,
}

/// `de440.constants()`: the single source of constants for the whole POP.
pub fn constants() -> Constants {
    let tsi = 1361.0;
    let c = 299792458.0;
    let re = 6378137.0;
    let f = 1.0 / 298.257223563;
    Constants {
        au_m: 149597870700.0,
        c,
        gm_sun: 1.32712440041279419e20,
        gm_earth: 3.98600435507e14,
        gm_moon: 4.902800118e12,
        emrat: 81.3005682214972154,
        tsi,
        p0: tsi / c,
        re_earth: re,
        f_earth: f,
        rp_earth: re * (1.0 - f),
        mu_earth: 3.986004418e14,
        omega_earth: 7.2921150e-5,
        n_a: 6.02214076e23,
        rsun: 6.957e8,
    }
}

/// NAIF ids used by the POP (`de440.state` help): SSB, EMB, Sun, Moon, Earth.
pub mod naif {
    /// Solar System Barycentre.
    pub const SSB: i32 = 0;
    /// Earth-Moon barycentre.
    pub const EMB: i32 = 3;
    /// Sun.
    pub const SUN: i32 = 10;
    /// Moon.
    pub const MOON: i32 = 301;
    /// Earth.
    pub const EARTH: i32 = 399;
}

/// Default kernel path: `matlab_sils/pop/03_frames_time/ephemeris/data/de440s.bsp`
/// in this repository (`de440.open()` with no argument).
pub fn default_kernel_path() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../../matlab_sils/pop/03_frames_time/ephemeris/data/de440s.bsp")
}

/// A loaded DE kernel with the Sun/Moon/Earth segments pre-resolved -- the
/// `eph` struct returned by `de440.open` (with `eph.const`).
#[derive(Debug, Clone)]
pub struct Ephem {
    kernel: Kernel,
    /// `eph.const` (`de440.constants()`).
    pub constants: Constants,
    seg_ssb_sun: usize,
    seg_ssb_emb: usize,
    seg_emb_earth: usize,
    seg_emb_moon: usize,
}

/// Julian date TDB -> seconds past J2000 TDB, as `de440.state` computes it.
#[inline]
pub fn jd_to_et(jd_tdb: f64) -> f64 { (jd_tdb - 2451545.0) * 86400.0 }

impl Ephem {
    /// `de440.open()`: load [`default_kernel_path`].
    pub fn open_default() -> Result<Ephem, SpkError> { Ephem::open(default_kernel_path()) }

    /// `de440.open(bspPath)`.
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Ephem, SpkError> {
        Ephem::from_kernel(Kernel::open(path)?)
    }

    /// Wrap an already parsed kernel; fails if it lacks 0->10, 0->3, 3->399 or 3->301.
    pub fn from_kernel(kernel: Kernel) -> Result<Ephem, SpkError> {
        let need = |c: i32, t: i32| {
            kernel.segment(c, t).ok_or_else(|| SpkError::Format(format!("segment {c}->{t} not in kernel")))
        };
        let seg_ssb_sun = need(naif::SSB, naif::SUN)?;
        let seg_ssb_emb = need(naif::SSB, naif::EMB)?;
        let seg_emb_earth = need(naif::EMB, naif::EARTH)?;
        let seg_emb_moon = need(naif::EMB, naif::MOON)?;
        Ok(Ephem { kernel, constants: constants(), seg_ssb_sun, seg_ssb_emb, seg_emb_earth, seg_emb_moon })
    }

    /// The underlying SPK kernel.
    pub fn kernel(&self) -> &Kernel { &self.kernel }

    /// `de440.state(center, target, jdTDB, eph)`: km, km/s, ICRF. `None` when the
    /// segment is not in the kernel (MATLAB asserts).
    pub fn state(&self, center: i32, target: i32, jd_tdb: f64) -> Option<(V3, V3)> {
        self.kernel.state(center, target, jd_to_et(jd_tdb))
    }

    #[inline]
    fn emb_earth(&self, et: f64) -> (V3, V3) { self.kernel.state_seg(self.seg_emb_earth, et) }

    fn earth_et(&self, et: f64, ree: &(V3, V3)) -> (V3, V3) {
        let (re, ve) = self.kernel.state_seg(self.seg_ssb_emb, et);
        let (r, v) = (&ree.0, &ree.1);
        ([re[0] + r[0], re[1] + r[1], re[2] + r[2]], [ve[0] + v[0], ve[1] + v[1], ve[2] + v[2]])
    }

    fn sun_et(&self, et: f64, ree: &(V3, V3)) -> (V3, V3) {
        let (rs, vs) = self.kernel.state_seg(self.seg_ssb_sun, et);
        let (re, ve) = self.earth_et(et, ree);
        let mut r = [0.0; 3];
        let mut v = [0.0; 3];
        for i in 0..3 {
            r[i] = (rs[i] - re[i]) * 1000.0;
            v[i] = (vs[i] - ve[i]) * 1000.0;
        }
        (r, v)
    }

    fn moon_et(&self, et: f64, ree: &(V3, V3)) -> (V3, V3) {
        let (rm, vm) = self.kernel.state_seg(self.seg_emb_moon, et);
        let mut r = [0.0; 3];
        let mut v = [0.0; 3];
        for i in 0..3 {
            r[i] = (rm[i] - ree.0[i]) * 1000.0;
            v[i] = (vm[i] - ree.1[i]) * 1000.0;
        }
        (r, v)
    }

    /// `de440.earth(jdTDB)`: geocentre w.r.t. the SSB [km, km/s].
    pub fn earth(&self, jd_tdb: f64) -> (V3, V3) {
        let et = jd_to_et(jd_tdb);
        let ree = self.emb_earth(et);
        self.earth_et(et, &ree)
    }

    /// `de440.sun(jdTDB)`: geocentric Sun (Earth->Sun) [m, m/s].
    pub fn sun(&self, jd_tdb: f64) -> (V3, V3) {
        let et = jd_to_et(jd_tdb);
        let ree = self.emb_earth(et);
        self.sun_et(et, &ree)
    }

    /// `de440.moon(jdTDB)`: geocentric Moon [m, m/s].
    pub fn moon(&self, jd_tdb: f64) -> (V3, V3) {
        let et = jd_to_et(jd_tdb);
        let ree = self.emb_earth(et);
        self.moon_et(et, &ree)
    }

    /// `ephemInputs(jdTDB, eph)`; see [`inputs`].
    pub fn inputs(&self, jd_tdb: f64) -> EphemInputs { inputs(self, jd_tdb) }
}

/// Every ephemeris-derived quantity the force/torque stack needs -- the struct `E`
/// returned by `ephemInputs.m` (one field per MATLAB field; SI, ICRF).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct EphemInputs {
    /// `E.jd_tdb` [TDB Julian date].
    pub jd_tdb: f64,
    /// `E.sun_unit` Earth->Sun unit vector.
    pub sun_unit: V3,
    /// `E.sun_dist` Earth-Sun distance [m].
    pub sun_dist: f64,
    /// `E.flux_scale = (AU/d)^2`.
    pub flux_scale: f64,
    /// `E.P_srp = P0*flux_scale` [N/m^2].
    pub p_srp: f64,
    /// `E.sun_eci` geocentric Sun position [m].
    pub sun_eci: V3,
    /// `E.sun_vel` geocentric Sun velocity [m/s].
    pub sun_vel: V3,
    /// `E.moon_eci` geocentric Moon position [m].
    pub moon_eci: V3,
    /// `E.moon_vel` geocentric Moon velocity [m/s].
    pub moon_vel: V3,
    /// `E.GM_sun` [m^3/s^2].
    pub gm_sun: f64,
    /// `E.GM_moon` [m^3/s^2].
    pub gm_moon: f64,
    /// `E.tide_sun` (= sun_eci) [m].
    pub tide_sun: V3,
    /// `E.tide_moon` (= moon_eci) [m].
    pub tide_moon: V3,
    /// `E.albedo_sun_unit` (= sun_unit).
    pub albedo_sun_unit: V3,
    /// `E.earth_helio_pos = -sun_eci` [m].
    pub earth_helio_pos: V3,
    /// `E.earth_helio_vel = -sun_vel` [m/s].
    pub earth_helio_vel: V3,
    /// `E.sun_ra = atan2(y, x)` [rad].
    pub sun_ra: f64,
    /// `E.sun_dec = asin(z/d)` [rad].
    pub sun_dec: f64,
    /// `E.const` (`eph.const`).
    pub constants: Constants,
}

/// `ephemInputs(jdTDB, eph)`: the single ephemeris interface point of the force
/// stack. The EMB->Earth record is evaluated once and shared by Sun and Moon
/// (MATLAB evaluates it twice with bit-identical results).
pub fn inputs(eph: &Ephem, jd_tdb: f64) -> EphemInputs {
    let c = eph.constants;
    let et = jd_to_et(jd_tdb);
    let ree = eph.emb_earth(et);
    let (rs, vs) = eph.sun_et(et, &ree);
    let (rm, vm) = eph.moon_et(et, &ree);
    let d = norm(&rs);
    let sun_unit = [rs[0] / d, rs[1] / d, rs[2] / d];
    let q = c.au_m / d;
    let flux_scale = q * q;
    EphemInputs {
        jd_tdb,
        sun_unit,
        sun_dist: d,
        flux_scale,
        p_srp: c.p0 * flux_scale,
        sun_eci: rs,
        sun_vel: vs,
        moon_eci: rm,
        moon_vel: vm,
        gm_sun: c.gm_sun,
        gm_moon: c.gm_moon,
        tide_sun: rs,
        tide_moon: rm,
        albedo_sun_unit: sun_unit,
        earth_helio_pos: [-rs[0], -rs[1], -rs[2]],
        earth_helio_vel: [-vs[0], -vs[1], -vs[2]],
        sun_ra: rs[1].atan2(rs[0]),
        sun_dec: (rs[2] / d).asin(),
        constants: c,
    }
}
