//! The force model of POP: op.buildWorld + op.accel (matlab_sils/pop/01_core/+op) and
//! op.coe2rv. One [`World`] holds everything buildWorld resolves once (epoch, frame
//! build, true Earth rate, gravity field, DE440 kernel, spacecraft, space weather);
//! [`World::accel`] evaluates the total acceleration at (t, r, v) summing the forces in
//! op.accel's order: gravity, thirdbody, drag, srp, erp, relativity, solidtides,
//! oceantides, empirical. The force set, the sum, the empirical acceleration, the elements to a state and the
//! sun-synchronous start are env's method env_force_model, generated from the design into `gen::forcemodel`
//! (tools/engine_build.py); the world's set-up, the context and the in-loop RK4 and interpolation are the code's.
use crate::drag::{self, DragConfig, DragInfo, SwSources};
use crate::ephem::{Ephem, EphemInputs};
use crate::frames::{self, Build, FrameOpt};
use crate::gravity::{Field, Gravity, GravityForceCfg, GravityModel};
use crate::la::*;
use crate::spaceweather::ManualIndices;
use crate::gen::forcemodel as fm;
use crate::{erp, geodetic, oceantides, relativity, solidtides, srp, thirdbody, time};

/// cfg.forces (each `None` = `.on = false`).
#[derive(Clone, Debug)]
pub struct Forces {
    pub gravity: GravityForceCfg,
    pub thirdbody: Option<thirdbody::Model>,
    pub drag: Option<DragConfig>,
    /// (model, eclipse, cfg Cr)
    pub srp: Option<(srp::SrpModel, srp::EclipseModel, Option<f64>)>,
    /// (model name: knocke | simple | ceres | boxwing, CrAoM, Cr, nrings, nseg)
    pub erp: Option<(String, Option<f64>, Option<f64>, Option<usize>, Option<usize>)>,
    pub relativity: Option<Vec<relativity::Term>>,
    pub solidtides: bool,
    pub oceantides: bool,
    /// radial / along-track / cross-track constant acceleration [m/s^2]
    pub empirical: Option<V3>,
}

impl Forces {
    /// config.defaultConfig's force set (drag on the exponential atmosphere): the design's (env_force_model's
    /// `default_forces`).
    pub fn pop_default() -> Forces {
        Forces::from_set(&fm::default_forces())
    }
    /// The in-loop set of asils.orbit.init: degree/order-6 field, Battin Sun+Moon,
    /// cannonball DTM2020 co-rotating drag, cannonball SRP with the conical shadow (env_force_model's `sils_forces`).
    pub fn sils(cr: f64) -> Forces {
        Forces::from_set(&fm::sils_forces(cr))
    }
    /// The forces of the design's force set (`gen::forcemodel::ForceSet`).
    pub fn from_set(s: &fm::ForceSet) -> Forces {
        use crate::gen::{densitymodel as dm, drag as gd, gravity as gg, gsi as gi, srp as gs, thirdbody as tb};
        let gravity = GravityForceCfg {
            model: match s.grav_model { gg::GRAVITYMODEL_TWOBODY => GravityModel::TwoBody, gg::GRAVITYMODEL_ZONAL => GravityModel::Zonal(s.grav_degree as usize), _ => GravityModel::SphHarm },
            degree: s.grav_degree as usize,
            order: s.has_grav_order.then_some(s.grav_order as usize),
        };
        let thirdbody = s.tb_on.then(|| match s.tb_model {
            tb::THIRDBODYMODEL_DIRECT => thirdbody::Model::Direct,
            tb::THIRDBODYMODEL_TIDAL => thirdbody::Model::Tidal,
            tb::THIRDBODYMODEL_LEGENDRE => thirdbody::Model::Legendre(s.tb_nmax as usize),
            _ => thirdbody::Model::Battin,
        });
        let panel = match s.drag_panel {
            gi::PANELMODEL_DRIA => drag::gsi::PanelModel::Dria,
            gi::PANELMODEL_SESAM => drag::gsi::PanelModel::Sesam,
            gi::PANELMODEL_CLL => drag::gsi::PanelModel::Cll,
            _ => drag::gsi::PanelModel::Sentman,
        };
        let atmos = match s.drag_atmos {
            dm::ATMOSMODEL_EXPONENTIAL => crate::atmos::AtmosModel::Exponential,
            dm::ATMOSMODEL_NRLMSISE => crate::atmos::AtmosModel::Nrlmsise,
            dm::ATMOSMODEL_JB2008 => crate::atmos::AtmosModel::Jb2008,
            dm::ATMOSMODEL_DTM2020_RESEARCH => crate::atmos::AtmosModel::Dtm2020Research,
            _ => crate::atmos::AtmosModel::Dtm2020,
        };
        let drag = s.drag_on.then(|| DragConfig {
            model: if s.drag_model == gd::DRAGMODEL_PANEL { drag::DragModel::Panel(panel) } else { drag::DragModel::Cannonball },
            atmos,
            cd: s.has_drag_cd.then_some(s.drag_cd),
            corotate: s.drag_corotate,
            gsi: drag::gsi::Gsi::from(s.drag_gsi),
        });
        let srp = s.srp_on.then(|| {
            let m = if s.srp_model == gs::SRPMODEL_BOXWING { srp::SrpModel::Boxwing } else { srp::SrpModel::Cannonball };
            let e = match s.srp_eclipse { gs::ECLIPSEMODEL_CYLINDRICAL => srp::EclipseModel::Cylindrical, gs::ECLIPSEMODEL_FINE => srp::EclipseModel::Fine, _ => srp::EclipseModel::Conical };
            (m, e, s.has_srp_cr.then_some(s.srp_cr))
        });
        assert!(!s.erp_on && !s.relativity_on, "a force set with Earth radiation or relativity names their models in code (Forces' fields)");
        Forces { gravity, thirdbody, drag, srp, erp: None, relativity: None, solidtides: s.solidtides, oceantides: s.oceantides,
                 empirical: s.has_empirical.then_some(s.empirical) }
    }
    fn need_ephem(&self) -> bool {
        self.thirdbody.is_some() || self.srp.is_some() || self.erp.is_some() || self.relativity.is_some() || self.solidtides || self.oceantides
    }
}

/// The spacecraft as the forces see it (cfg.spacecraft).
#[derive(Clone, Debug)]
pub struct Sc {
    pub mass: f64, pub aref: f64, pub cd: Option<f64>, pub cr: Option<f64>, pub r_bi: M3,
    pub srp_facets: Vec<srp::Facet>,
    pub drag_facets: Option<Vec<drag::geom::Facet>>,
}

/// op.buildWorld's product.
pub struct World {
    pub epoch: [f64; 6],
    pub build: Build,
    pub frame: FrameOpt,
    pub omega_eci: V3,
    pub grav: Gravity,
    pub eph: Option<Ephem>,
    pub sc: Sc,
    pub sw_manual: Option<ManualIndices>,
    pub forces: Forces,
}

/// What a run records of the data's validity at an instant (S12.4a, P9; DR-env-002): `eop_flag` 1 when the EOP table is
/// extrapolated there (builds A/B/C), `leap_status` 1 when the leap-second table has expired.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Validity { pub eop_flag: i32, pub leap_status: i32 }

/// What op.accel builds per evaluation (ctx) and returns alongside a.
#[derive(Clone, Debug)]
pub struct Ctx {
    pub t: f64, pub utc: [f64; 6], pub times: time::Times, pub e: Option<EphemInputs>,
    pub c: M3, pub r_eci: V3, pub v_eci: V3, pub r_ecef: V3, pub v_ecef: V3,
}

/// Per-force accelerations (op.accel's `parts`), zero when off.
#[derive(Clone, Copy, Debug, Default)]
pub struct Parts { pub gravity: V3, pub thirdbody: V3, pub drag: V3, pub srp: V3, pub erp: V3, pub relativity: V3, pub solidtides: V3, pub oceantides: V3, pub empirical: V3 }

impl World {
    /// op.buildWorld: the frame build ('gmst' with dUT1 = 0 when no EOP), the true Earth
    /// rate, the gravity field, the DE440 kernel (when a force needs it).
    pub fn new(epoch: [f64; 6], build: Build, frame: FrameOpt, field: Field, forces: Forces, sc: Sc, sw_manual: Option<ManualIndices>, kernel: Option<std::path::PathBuf>) -> Result<World, crate::PopError> {
        let omega_eci = frames::earth_rate_eci(epoch, build, &frame)?;
        let grav = Gravity::new(field, forces.gravity)?;
        let eph = if forces.need_ephem() {
            Some(match kernel { Some(p) => Ephem::open(p), None => Ephem::open_default() }.map_err(|e| crate::PopError::Data(format!("DE440 kernel: {e:?}")))?)
        } else { None };
        if forces.drag.is_some() || forces.srp.is_some() {
            if !(sc.mass > 0.0) { return Err(crate::PopError::Unsupported("spacecraft.mass must be a positive scalar for drag/SRP".into())); }
            if !(sc.aref > 0.0) { return Err(crate::PopError::Unsupported("spacecraft.Aref must be a positive scalar for drag/SRP".into())); }
        }
        Ok(World { epoch, build, frame, omega_eci, grav, eph, sc, sw_manual, forces })
    }

    /// The validity of the data at t seconds after the epoch: the EOP table's extrapolation flag and the leap-second table's
    /// status (both nondecreasing in t, so a run records the end of its span). A run warns and records it; it never refuses.
    pub fn validity(&self, t: f64) -> Validity {
        let utc = time::addsec(self.epoch, t);
        let times = time::convert_utc(utc[0], utc[1], utc[2], utc[3], utc[4], utc[5], self.frame.dut1);
        let mut v = Validity { eop_flag: 0, leap_status: times.leap_status as i32 };
        if self.build != Build::Gmst {
            let fo = FrameOpt { refuse_beyond_validity: false, ..self.frame.clone() };
            if let Ok((_, _, info)) = frames::try_eci2ecef(utc, self.build, &fo) { v.eop_flag = info.eop_flag; v.leap_status |= info.leap_status; }
        }
        v
    }

    /// The context of op.accel at t (time scales, ephemeris inputs, ECI->ECEF).
    /// The Earth frame (GCRF to ITRF on build A) at t seconds from the epoch, as `context` takes it: what a receiver's fix at that
    /// instant is made in (S12.5, T5/E3).
    pub fn earth_frame(&self, t: f64) -> M3 {
        let utc = time::addsec(self.epoch, t);
        if self.build == Build::Gmst {
            let times = time::convert_utc(utc[0], utc[1], utc[2], utc[3], utc[4], utc[5], self.frame.dut1);
            let mut fo = self.frame.clone();
            fo.gmst_rad = Some(times.gmst_rad);
            frames::eci2ecef(utc, self.build, &fo).0
        } else { frames::eci2ecef(utc, self.build, &self.frame).0 }
    }

    pub fn context(&self, t: f64, r: &V3, v: &V3) -> Ctx {
        let utc = time::addsec(self.epoch, t);
        let times = time::convert_utc(utc[0], utc[1], utc[2], utc[3], utc[4], utc[5], self.frame.dut1);
        let e = if self.forces.need_ephem() { self.eph.as_ref().map(|k| k.inputs(times.tdb_jd)) } else { None };
        // the gmst build takes the time scales' GMST; the CIO builds take their EOP from the world's own set (no copy of it)
        let (c, _ct) = if self.build == Build::Gmst {
            let mut fo = self.frame.clone();
            fo.gmst_rad = Some(times.gmst_rad);
            frames::eci2ecef(utc, self.build, &fo)
        } else { frames::eci2ecef(utc, self.build, &self.frame) };
        let r_ecef = mv(&c, r);
        let v_ecef = mv(&c, &sub(v, &cross(&self.omega_eci, r)));
        Ctx { t, utc, times, e, c, r_eci: *r, v_eci: *v, r_ecef, v_ecef }
    }

    /// op.accel: total acceleration [m/s^2] at (t, r, v), the parts, the context and the
    /// drag information (asils reads info.drag.atm.rho).
    pub fn accel(&mut self, t: f64, r: &V3, v: &V3) -> Result<(V3, Parts, Ctx, Option<DragInfo>), crate::PopError> {
        let ctx = self.context(t, r, v);
        let mut p = Parts { gravity: self.grav.accel_eci(r, &ctx.c), ..Default::default() };
        let mut info = None;
        let f = &self.forces;
        if let (Some(model), Some(e)) = (f.thirdbody, ctx.e.as_ref()) {
            p.thirdbody = thirdbody::force(&thirdbody::ThirdBodyInput::new(*r, e, model));
        }
        if let Some(dc) = f.drag.as_ref() {
            let (lat, lon, alt) = geodetic::geodetic(&ctx.r_ecef);
            let dsc = drag::Spacecraft { mass: self.sc.mass, aref: self.sc.aref, cd: self.sc.cd, r_bi: self.sc.r_bi, facets: self.sc.drag_facets.as_deref() };
            let inp = drag::DragInput {
                r_eci: *r, v_eci: *v, lat_rad: lat, lon_rad: lon, alt_m: alt, utc: ctx.utc, doy: ctx.times.doy,
                omega_eci: Some(self.omega_eci), sun_eci: ctx.e.as_ref().map(|e| e.sun_eci), sc: dsc, cfg: dc,
                sw: SwSources { manual: self.sw_manual.as_ref(), ..Default::default() },
            };
            let (a, i) = drag::accel(&inp).map_err(|e| crate::PopError::Run(format!("drag at t = {t}: {e:?}")))?;
            p.drag = a;
            info = Some(i);
        }
        if let (Some((model, ecl, cr)), Some(e)) = (f.srp, ctx.e.as_ref()) {
            let sc = srp_sc_take(&mut self.sc);
            let mut inp = srp::SrpInput::new(*r, e, &sc, model, ecl);
            inp.cr = cr;
            p.srp = srp::force(&inp);
            srp_sc_return(&mut self.sc, sc);
        }
        if let (Some((name, cr_aom, cr, nrings, nseg)), Some(e)) = (f.erp.as_ref(), ctx.e.as_ref()) {
            let model = erp::Model::from_name(name).ok_or_else(|| crate::PopError::Unsupported(format!("unknown erp model {name}")))?;
            let sc = srp_sc_take(&mut self.sc);
            let mut inp = erp::ErpInput::new(*r, e, ctx.times.doy, &sc, model);
            inp.cr_aom = *cr_aom; inp.cr = *cr; inp.nrings = *nrings; inp.nseg = *nseg;
            p.erp = erp::force(&inp);
            srp_sc_return(&mut self.sc, sc);
        }
        if let Some(terms) = f.relativity.as_ref() {
            let inp = relativity::RelativityInput::new(*r, *v, ctx.e.as_ref(), terms, self.grav.field.gm);
            p.relativity = relativity::force(&inp).map_err(|e| crate::PopError::Run(e.to_string()))?;
        }
        if f.solidtides {
            if let Some(e) = ctx.e.as_ref() {
                p.solidtides = solidtides::solid_tides_accel(&solidtides::SolidTideInputs {
                    r_ecef: ctx.r_ecef, c_eci2ecef: ctx.c, sun_eci: e.sun_eci, moon_eci: e.moon_eci, mu: self.grav.field.gm, re: self.grav.field.re });
            }
        }
        if f.oceantides {
            p.oceantides = oceantides::ocean_tides_accel(&oceantides::OceanTideInputs {
                tt_jd: ctx.times.tt_jd, r_ecef: ctx.r_ecef, c_eci2ecef: ctx.c, mu: self.grav.field.gm, re: self.grav.field.re });
        }
        if let Some(acc) = f.empirical {
            p.empirical = fm::empirical_accel(*r, *v, acc);
        }
        let a = fm::force_sum(p.gravity, p.thirdbody, p.drag, p.srp, p.erp, p.relativity, p.solidtides, p.oceantides, p.empirical);
        Ok((a, p, ctx, info))
    }
}

/// The spacecraft as SRP and ERP take it. The facets are moved out of the world's `Sc` for the
/// call (no clone per evaluation) and put back with [`srp_sc_return`] before the next use.
fn srp_sc_take(sc: &mut Sc) -> srp::Spacecraft {
    srp::Spacecraft { mass: sc.mass, aref: sc.aref, cd: sc.cd, cr: sc.cr, r_bi: sc.r_bi, facets: std::mem::take(&mut sc.srp_facets) }
}
fn srp_sc_return(sc: &mut Sc, s: srp::Spacecraft) { sc.srp_facets = s.facets; }

/// op.coe2rv: classical elements -> ECI position/velocity (closed orbits).
pub fn coe2rv(a: f64, e: f64, inc: f64, raan: f64, argp: f64, nu: f64, mu: f64) -> (V3, V3) {
    fm::pop_coe2rv(a, e, inc, raan, argp, nu, mu)
}

/// The in-loop orbit of the SILS (asils.orbit.{init,node,advance,state,context}): RK4
/// nodes every `h` seconds on the full POP force model, cubic Hermite in between, and the
/// environment context (Sun, Moon, SRP pressure, density, ECI->ECEF) at the nodes.
pub struct InLoop {
    pub world: World,
    pub h: f64,
    pub omega_e: f64,
    n0: Node,
    n1: Node,
}

/// The node record of asils.orbit.node.
#[derive(Clone, Debug)]
pub struct Node { pub t: f64, pub r: V3, pub v: V3, pub a: V3, pub c: M3, pub sun_eci: V3, pub moon_eci: V3, pub p_srp: f64, pub rho: f64, pub v_earth: V3, pub parts: Parts }

/// asils.orbit.context at t.
#[derive(Clone, Copy, Debug)]
/// `v_earth`: the geocentre's velocity w.r.t. the solar-system barycentre [m/s] (the star tracker's aberration, S12.5b); with no
/// ephemeris the Sun is the fixed stand-in of env_force_model's no_ephem_env and the Earth does not move relative to it: zero.
pub struct EnvCtx { pub sun_eci: V3, pub moon_eci: V3, pub p_srp: f64, pub rho: f64, pub c: M3, pub v_earth: V3 }

impl InLoop {
    fn node(w: &mut World, t: f64, r: V3, v: V3) -> Result<Node, crate::PopError> {
        let (a, parts, ctx, info) = w.accel(t, &r, &v)?;
        let (sun_eci, p_srp, moon_eci) = match ctx.e.as_ref() {
            Some(e) => (e.sun_eci, e.p_srp, e.moon_eci),
            None => fm::no_ephem_env(),
        };
        let v_earth = ctx.e.as_ref().map(|e| e.earth_ssb_vel).unwrap_or([0.0; 3]);
        Ok(Node { t, r, v, a, c: ctx.c, sun_eci, moon_eci, p_srp, rho: info.map(|i| i.rho).unwrap_or(0.0), v_earth, parts })
    }

    fn advance(w: &mut World, n: &Node, h: f64) -> Result<Node, crate::PopError> {
        let t = n.t;
        let y: [f64; 6] = [n.r[0], n.r[1], n.r[2], n.v[0], n.v[1], n.v[2]];
        let k1 = [n.v[0], n.v[1], n.v[2], n.a[0], n.a[1], n.a[2]];
        let mut f = |tt: f64, yy: &[f64; 6]| -> Result<[f64; 6], crate::PopError> {
            let (a, ..) = w.accel(tt, &[yy[0], yy[1], yy[2]], &[yy[3], yy[4], yy[5]])?;
            Ok([yy[3], yy[4], yy[5], a[0], a[1], a[2]])
        };
        let mut y2 = y; for i in 0..6 { y2[i] = y[i] + 0.5*h*k1[i]; }
        let k2 = f(t + 0.5*h, &y2)?;
        let mut y3 = y; for i in 0..6 { y3[i] = y[i] + 0.5*h*k2[i]; }
        let k3 = f(t + 0.5*h, &y3)?;
        let mut y4 = y; for i in 0..6 { y4[i] = y[i] + h*k3[i]; }
        let k4 = f(t + h, &y4)?;
        let mut y1 = y;
        for i in 0..6 { y1[i] = y[i] + h/6.0*(k1[i] + 2.0*k2[i] + 2.0*k3[i] + k4[i]); }
        InLoop::node(w, t + h, [y1[0], y1[1], y1[2]], [y1[3], y1[4], y1[5]])
    }

    /// asils.orbit.init after the world is built: node 0 at (r0, v0), node 1 one step on.
    pub fn new(mut world: World, h: f64, r0: V3, v0: V3) -> Result<InLoop, crate::PopError> {
        let n0 = InLoop::node(&mut world, 0.0, r0, v0)?;
        let n1 = InLoop::advance(&mut world, &n0, h)?;
        let omega_e = crate::ephem::constants().omega_earth;
        Ok(InLoop { world, h, omega_e, n0, n1 })
    }

    /// asils.orbit.state: Hermite position/velocity at t (t non-decreasing).
    pub fn state(&mut self, t: f64) -> Result<(V3, V3), crate::PopError> {
        while t > self.n1.t + 1e-9 {
            let n2 = InLoop::advance(&mut self.world, &self.n1, self.h)?;
            self.n0 = std::mem::replace(&mut self.n1, n2);
        }
        let (a, b) = (&self.n0, &self.n1);
        let h = b.t - a.t;
        let s = (t - a.t)/h;
        let (s2, s3) = (s*s, s*s*s);
        let (h00, h10, h01, h11) = (2.0*s3 - 3.0*s2 + 1.0, s3 - 2.0*s2 + s, -2.0*s3 + 3.0*s2, s3 - s2);
        let mut r = [0.0; 3];
        let mut v = [0.0; 3];
        for i in 0..3 {
            r[i] = h00*a.r[i] + h10*h*a.v[i] + h01*b.r[i] + h11*h*b.v[i];
            v[i] = h00*a.v[i] + h10*h*a.a[i] + h01*b.v[i] + h11*h*b.a[i];
        }
        Ok((r, v))
    }

    /// asils.orbit.context: Sun/Moon/P_srp linear, density log-linear, ECI->ECEF turned on from node 0 towards node 1
    /// ([`frac_rot`]).
    pub fn context(&self, t: f64) -> EnvCtx {
        let (a, b) = (&self.n0, &self.n1);
        let s = (t - a.t)/(b.t - a.t);
        let lerp = |x: &V3, y: &V3| [x[0] + s*(y[0] - x[0]), x[1] + s*(y[1] - x[1]), x[2] + s*(y[2] - x[2])];
        let rho = if a.rho > 0.0 && b.rho > 0.0 { (a.rho.ln() + s*(b.rho.ln() - a.rho.ln())).exp() } else { a.rho + s*(b.rho - a.rho) };
        EnvCtx { sun_eci: lerp(&a.sun_eci, &b.sun_eci), moon_eci: lerp(&a.moon_eci, &b.moon_eci), p_srp: a.p_srp + s*(b.p_srp - a.p_srp), rho, c: frac_rot(&a.c, &b.c, s), v_earth: lerp(&a.v_earth, &b.v_earth) }
    }

    /// The last node's per-force accelerations (diagnostics).
    pub fn parts(&self) -> Parts { self.n1.parts }
    /// Set the attitude the box-wing / panel models read (asils: O.W.sc.R_bi = dcm(q)').
    pub fn set_attitude(&mut self, r_bi: M3) { self.world.sc.r_bi = r_bi; }
}

/// The Earth frame at the fraction s of the step from node 0 to node 1, from the two nodes' frames: (C1 C0^T)^s C0, the
/// rotation between them taken to the power s about its own axis (Rodrigues), so the frame is exact at both nodes. Turning
/// node 0's frame about the z axis of the Earth-fixed frame by the Earth rate times the time (what this did until S12.4b)
/// is exact only for a frame whose pole is the Earth-fixed z axis: on build A the Earth turns about the CIP, which is the
/// pole of the polar motion (about 1.5e-6 rad off), and a step of 10 s then left 1.3e-9 rad.
pub fn frac_rot(c0: &M3, c1: &M3, s: f64) -> M3 {
    let d = mm(c1, &transpose(c0));
    let v = [d[2][1] - d[1][2], d[0][2] - d[2][0], d[1][0] - d[0][1]];      // 2 sin(theta) n
    let sv = (v[0]*v[0] + v[1]*v[1] + v[2]*v[2]).sqrt();
    if sv == 0.0 { return *c0; }
    let th = (0.5*sv).atan2(0.5*(d[0][0] + d[1][1] + d[2][2] - 1.0));
    let n = [v[0]/sv, v[1]/sv, v[2]/sv];
    let (sn, cs) = (s*th).sin_cos();
    let k = [[0.0, -n[2], n[1]], [n[2], 0.0, -n[0]], [-n[1], n[0], 0.0]];
    let mut r = [[0.0; 3]; 3];
    for i in 0..3 { for j in 0..3 { r[i][j] = (if i == j { cs } else { 0.0 }) + (1.0 - cs)*n[i]*n[j] + sn*k[i][j]; } }
    mm(&r, c0)
}

/// The orbit geometry of asils.orbit.init: a = Re + alt, RAAN from the LTAN and the DE440
/// Sun right ascension at the epoch, argument of latitude u0 -> (r0, v0, raan).
pub fn sso_initial(w: &World, alt_km: f64, ecc: f64, inc_deg: f64, ltan_h: f64, argp_deg: f64, u0_deg: f64) -> Result<(V3, V3, f64), crate::PopError> {
    let e = w.epoch;
    let t = time::convert_utc(e[0], e[1], e[2], e[3], e[4], e[5], 0.0);
    let eph = w.eph.as_ref().ok_or_else(|| crate::PopError::Data("the SSO geometry needs the DE440 kernel".into()))?;
    let ei = eph.inputs(t.tdb_jd);
    Ok(fm::sso_initial(ei.sun_unit, alt_km, ecc, inc_deg, ltan_h, argp_deg, u0_deg))
}
