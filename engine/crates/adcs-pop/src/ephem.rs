//! JPL DE440 ephemeris API and the per-step ephemeris bundle -- port of
//! `matlab_sils/pop/03_frames_time/ephemeris/+de440/*` (`open`, `state`, `sun`,
//! `moon`, `earth`, `constants`) and `ephemeris/ephemInputs.m`: env's method env_de440 over env's slice of DE440's
//! records (env_de440_slice: the Sun, the Earth-Moon barycentre, the Earth and the Moon over the runs' span, read into
//! the design by tools/readers.py), generated from the design into `gen::de440` (tools/engine_build.py). The DAF/SPK
//! reader ([`crate::spk`]) stays code: it loads a kernel, and serves the generated evaluation a record the design's
//! slice does not hold (an epoch outside the runs' span, another body).
//!
//! Frame ICRF (== GCRF/J2000 ECI to < 1 mas). Time: TDB Julian date (TT accepted,
//! < 1.7 ms). Units exactly as MATLAB: [`Ephem::state`] and [`Ephem::earth`] in
//! km, km/s; [`Ephem::sun`], [`Ephem::moon`] and [`EphemInputs`] in SI.

use crate::la::V3;
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

/// `de440.constants()`: the single source of constants for the whole POP (the design's, `gen::de440`).
pub fn constants() -> Constants {
    let k = crate::gen::de440::de440_constants();
    Constants {
        au_m: k.au_m, c: k.c, gm_sun: k.gm_sun, gm_earth: k.gm_earth, gm_moon: k.gm_moon, emrat: k.emrat, tsi: k.tsi, p0: k.p0,
        re_earth: k.re_earth, f_earth: k.f_earth, rp_earth: k.rp_earth, mu_earth: k.mu_earth, omega_earth: k.omega_earth,
        n_a: k.n_a, rsun: k.rsun,
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

/// The DE440 ephemeris -- the `eph` struct returned by `de440.open` (with `eph.const`): the design's slice of the
/// Sun, Earth-Moon barycentre, Earth and Moon records, and, when one is open, a kernel for what the slice does not
/// hold (its Sun/Moon/Earth segments pre-resolved).
#[derive(Debug, Clone)]
pub struct Ephem {
    kernel: Option<Kernel>,
    /// `eph.const` (`de440.constants()`).
    pub constants: Constants,
    /// the kernel's segments SSB->Sun, SSB->EMB, EMB->Earth, EMB->Moon (the design's DeSegment order)
    segs: Option<[usize; 4]>,
}

/// Julian date TDB -> seconds past J2000 TDB, as `de440.state` computes it.
pub use crate::gen::de440::jd_to_et;

use crate::gen::de440 as de;

impl Ephem {
    /// `de440.open()`: load [`default_kernel_path`].
    pub fn open_default() -> Result<Ephem, SpkError> { Ephem::open(default_kernel_path()) }

    /// `de440.open(bspPath)`.
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Ephem, SpkError> {
        Ephem::from_kernel(Kernel::open(path)?)
    }

    /// The design's slice alone (no kernel): the runs' span, 1 Dec 2026 to 1 Feb 2028 TDB. An epoch outside it is
    /// refused (a panic naming the slice) -- open a kernel for those.
    pub fn design() -> Ephem { Ephem { kernel: None, constants: constants(), segs: None } }

    /// Whether the design's slice holds every record [`Ephem::inputs`] needs at a TDB Julian date.
    pub fn in_design(jd_tdb: f64) -> bool {
        let et = jd_to_et(jd_tdb);
        (0..4).all(|s| de::de440_record(s, et).0)
    }

    /// Wrap an already parsed kernel; fails if it lacks 0->10, 0->3, 3->399 or 3->301.
    pub fn from_kernel(kernel: Kernel) -> Result<Ephem, SpkError> {
        let need = |c: i32, t: i32| {
            kernel.segment(c, t).ok_or_else(|| SpkError::Format(format!("segment {c}->{t} not in kernel")))
        };
        let segs = [need(naif::SSB, naif::SUN)?, need(naif::SSB, naif::EMB)?, need(naif::EMB, naif::EARTH)?, need(naif::EMB, naif::MOON)?];
        Ok(Ephem { kernel: Some(kernel), constants: constants(), segs: Some(segs) })
    }

    /// The underlying SPK kernel (panics for [`Ephem::design`], which has none).
    pub fn kernel(&self) -> &Kernel { self.kernel.as_ref().expect("this ephemeris is the design's slice: no kernel is open") }

    /// `de440.state(center, target, jdTDB, eph)`: km, km/s, ICRF. `None` when the
    /// segment is not in the kernel (MATLAB asserts), or, with no kernel, not one of the design's four.
    pub fn state(&self, center: i32, target: i32, jd_tdb: f64) -> Option<(V3, V3)> {
        let et = jd_to_et(jd_tdb);
        if let Some(k) = &self.kernel {
            return k.state(center, target, et);
        }
        let s = [(naif::SSB, naif::SUN), (naif::SSB, naif::EMB), (naif::EMB, naif::EARTH), (naif::EMB, naif::MOON)].iter().position(|&p| p == (center, target))?;
        Some(self.seg(s as i64, et))
    }

    /// One of the four segments at `et` [km, km/s]: from the design's slice, else from the kernel's record.
    fn seg(&self, s: i64, et: f64) -> (V3, V3) {
        let (ok, p, v) = de::de440_segment(s, et);
        if ok {
            return (p, v);
        }
        match (&self.kernel, self.segs) {
            (Some(k), Some(i)) => k.state_seg(i[s as usize], et),
            _ => panic!("DE440: et {et} s is outside the design's slice (1 Dec 2026 to 1 Feb 2028 TDB) and no kernel is open"),
        }
    }

    /// `de440.earth(jdTDB)`: geocentre w.r.t. the SSB [km, km/s].
    pub fn earth(&self, jd_tdb: f64) -> (V3, V3) {
        let et = jd_to_et(jd_tdb);
        let (er, ev) = self.seg(de::DESEGMENT_EMB_EARTH, et);
        let (br, bv) = self.seg(de::DESEGMENT_SSB_EMB, et);
        let z = [0.0; 3];
        let g = de::geocentric(z, z, br, bv, er, ev, z, z);
        (g.4, g.5)
    }

    /// `de440.sun(jdTDB)`: geocentric Sun (Earth->Sun) [m, m/s].
    pub fn sun(&self, jd_tdb: f64) -> (V3, V3) {
        let et = jd_to_et(jd_tdb);
        let (er, ev) = self.seg(de::DESEGMENT_EMB_EARTH, et);
        let (br, bv) = self.seg(de::DESEGMENT_SSB_EMB, et);
        let (sr, sv) = self.seg(de::DESEGMENT_SSB_SUN, et);
        let z = [0.0; 3];
        let g = de::geocentric(sr, sv, br, bv, er, ev, z, z);
        (g.0, g.1)
    }

    /// `de440.moon(jdTDB)`: geocentric Moon [m, m/s].
    pub fn moon(&self, jd_tdb: f64) -> (V3, V3) {
        let et = jd_to_et(jd_tdb);
        let (er, ev) = self.seg(de::DESEGMENT_EMB_EARTH, et);
        let (mr, mv) = self.seg(de::DESEGMENT_EMB_MOON, et);
        let z = [0.0; 3];
        let g = de::geocentric(z, z, z, z, er, ev, mr, mv);
        (g.2, g.3)
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
/// stack (the design's, `gen::de440::ephem_inputs`). The EMB->Earth record is evaluated once and shared by Sun and
/// Moon (MATLAB evaluates it twice with bit-identical results).
pub fn inputs(eph: &Ephem, jd_tdb: f64) -> EphemInputs {
    let et = jd_to_et(jd_tdb);
    let (er, ev) = eph.seg(de::DESEGMENT_EMB_EARTH, et);
    let (br, bv) = eph.seg(de::DESEGMENT_SSB_EMB, et);
    let (sr, sv) = eph.seg(de::DESEGMENT_SSB_SUN, et);
    let (mr, mv) = eph.seg(de::DESEGMENT_EMB_MOON, et);
    let (rs, vs, rm, vm, _re, _ve) = de::geocentric(sr, sv, br, bv, er, ev, mr, mv);
    let e = de::ephem_inputs(jd_tdb, rs, vs, rm, vm);
    EphemInputs {
        jd_tdb: e.jd_tdb,
        sun_unit: e.sun_unit,
        sun_dist: e.sun_dist,
        flux_scale: e.flux_scale,
        p_srp: e.p_srp,
        sun_eci: e.sun_eci,
        sun_vel: e.sun_vel,
        moon_eci: e.moon_eci,
        moon_vel: e.moon_vel,
        gm_sun: e.gm_sun,
        gm_moon: e.gm_moon,
        tide_sun: e.sun_eci,
        tide_moon: e.moon_eci,
        albedo_sun_unit: e.sun_unit,
        earth_helio_pos: e.earth_helio_pos,
        earth_helio_vel: e.earth_helio_vel,
        sun_ra: e.sun_ra,
        sun_dec: e.sun_dec,
        constants: eph.constants,
    }
}
