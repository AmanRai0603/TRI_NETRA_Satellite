//! Gravity and tides vs the MATLAB POP (Octave reference vectors printed by
//! refgen/gravity_tides.m, gravity_tides_tides.m into tests/data/gravity_tides_*.json).
//!
//! Owner: Agastya. Copyright (c) 2026 Agastya. All rights reserved.
use adcs_pop::gravity::{self, Field, GravityFieldCfg, GravityForceCfg, GravityModel, SphHarm};
use adcs_pop::la::{M3, V3};
use adcs_pop::oceantides::{self, FesTable, OceanTideInputs};
use adcs_pop::solidtides::{self, Dcs5, SolidTideInputs};
use serde_json::Value;
use std::path::{Path, PathBuf};

const TOL_GRAV: f64 = 1e-13;
const TOL_POT: f64 = 1e-13;
const TOL_TIDE: f64 = 1e-12;
const FLOOR_TIDE: f64 = 1e-18;

fn data_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests").join("data")
}
fn load(name: &str) -> Value {
    let t = std::fs::read_to_string(data_dir().join(name)).expect(name);
    let mut p = Json { b: t.as_bytes(), i: 0 };
    p.value()
}

/// Minimal JSON reader with correctly rounded numbers (`str::parse::<f64>`):
/// serde_json without its `float_roundtrip` feature may land 1 ulp off, and the
/// tide finite differences need the Octave inputs bit for bit.
struct Json<'a> {
    b: &'a [u8],
    i: usize,
}
impl Json<'_> {
    fn ws(&mut self) {
        while self.i < self.b.len() && self.b[self.i].is_ascii_whitespace() {
            self.i += 1;
        }
    }
    fn value(&mut self) -> Value {
        self.ws();
        match self.b[self.i] {
            b'{' => {
                self.i += 1;
                let mut m = serde_json::Map::new();
                loop {
                    self.ws();
                    if self.b[self.i] == b'}' {
                        self.i += 1;
                        break;
                    }
                    let Value::String(k) = self.value() else { panic!("key") };
                    self.ws();
                    assert_eq!(self.b[self.i], b':');
                    self.i += 1;
                    let v = self.value();
                    m.insert(k, v);
                    self.ws();
                    if self.b[self.i] == b',' {
                        self.i += 1;
                    }
                }
                Value::Object(m)
            }
            b'[' => {
                self.i += 1;
                let mut a = Vec::new();
                loop {
                    self.ws();
                    if self.b[self.i] == b']' {
                        self.i += 1;
                        break;
                    }
                    a.push(self.value());
                    self.ws();
                    if self.b[self.i] == b',' {
                        self.i += 1;
                    }
                }
                Value::Array(a)
            }
            b'"' => {
                let s = self.i + 1;
                let e = s + self.b[s..].iter().position(|&c| c == b'"').unwrap();
                self.i = e + 1;
                Value::String(String::from_utf8(self.b[s..e].to_vec()).unwrap())
            }
            _ => {
                let s = self.i;
                while self.i < self.b.len() && matches!(self.b[self.i], b'0'..=b'9' | b'-' | b'+' | b'.' | b'e' | b'E') {
                    self.i += 1;
                }
                let x: f64 = std::str::from_utf8(&self.b[s..self.i]).unwrap().parse().unwrap();
                Value::Number(serde_json::Number::from_f64(x).unwrap())
            }
        }
    }
}
fn f(v: &Value) -> f64 {
    v.as_f64().expect("number")
}
fn fv(v: &Value) -> Vec<f64> {
    v.as_array().expect("array").iter().map(f).collect()
}
fn v3(v: &Value) -> V3 {
    let x = fv(v);
    [x[0], x[1], x[2]]
}
fn m3(v: &Value) -> M3 {
    let x = fv(v);
    [[x[0], x[1], x[2]], [x[3], x[4], x[5]], [x[6], x[7], x[8]]]
}
fn rows3(v: &Value) -> Vec<V3> {
    fv(v).chunks(3).map(|c| [c[0], c[1], c[2]]).collect()
}
fn vnorm(a: &V3) -> f64 {
    (a[0] * a[0] + a[1] * a[1] + a[2] * a[2]).sqrt()
}
/// |a-b| / |b| (vector 2-norms), with an absolute floor in the denominator.
fn rel(a: &V3, b: &V3, floor: f64) -> f64 {
    let d = [a[0] - b[0], a[1] - b[1], a[2] - b[2]];
    vnorm(&d) / vnorm(b).max(floor)
}

fn synth_field(degree: usize) -> Field {
    Field::load_gfc(data_dir().join("gravity_tides_synth.gfc"), Some(degree)).unwrap()
}

#[test]
fn fields_and_loader() {
    let g = load("gravity_tides_grav.json");
    let d = &g["default"];
    let fd = Field::default_field();
    assert_eq!(fd.gm, f(&d["mu"]));
    assert_eq!(fd.re, f(&d["re"]));
    assert_eq!(fd.nmax as f64, f(&d["nmax"]));
    assert_eq!(fd.name, d["name"].as_str().unwrap());
    assert_eq!(fd.cbar, fv(&d["cbar"]));
    assert_eq!(fd.sbar, fv(&d["sbar"]));
    assert_eq!(fd.j, fv(&d["j"]));

    let s = &g["synth"];
    let fs = synth_field(70);
    assert_eq!(fs.gm, f(&s["mu"]));
    assert_eq!(fs.re, f(&s["re"]));
    assert_eq!(fs.nmax as f64, f(&s["nmax"]));
    assert_eq!(fs.name, s["name"].as_str().unwrap());
    assert_eq!(fs.j, fv(&s["j"]));
    assert_eq!(fs.c(70, 33), f(&s["c70_33"]));
    let f20 = synth_field(20);
    assert_eq!(f20.nmax as f64, f(&s["nmax20"]));
    assert_eq!(f20.cbar, fv(&s["cbar20"]));
    assert_eq!(f20.sbar, fv(&s["sbar20"]));
    let fall = Field::load_gfc(data_dir().join("gravity_tides_synth.gfc"), None).unwrap();
    assert_eq!(fall.nmax as f64, f(&s["nmax_all"]));
    assert_eq!(fall.c(72, 5), f(&s["c_all_72_5"]));
    assert_eq!(fall.s(72, 71), f(&s["s_all_72_71"]));

    // op.gravLoad
    let dl = gravity::grav_load(&GravityFieldCfg { field: "DEFAULT".into(), degree: Some(20) }, &[]).unwrap();
    assert_eq!(dl.cbar, fd.cbar);
    let sl = gravity::grav_load(&GravityFieldCfg { field: "gravity_tides_synth.gfc".into(), degree: Some(70) }, &[data_dir()]).unwrap();
    assert_eq!(sl.cbar, fs.cbar);
    assert!(gravity::grav_load(&GravityFieldCfg { field: "EGM2008".into(), degree: Some(70) }, &[data_dir()]).is_err());
    assert!(GravityModel::parse("toolbox").is_err());
    assert_eq!(GravityModel::parse("J4").unwrap(), GravityModel::Zonal(4));
}

#[test]
fn spherical_harmonic_potential_zonal() {
    let g = load("gravity_tides_grav.json");
    let pos = rows3(&g["positions"]);
    let fd = Field::default_field();
    let fs = synth_field(70);
    let mut wd = SphHarm::new(&fd);
    let mut ws = SphHarm::new(&fs);
    let mut worst = 0.0f64;
    for c in g["sph"].as_array().unwrap() {
        let (fld, deg, ord) = (c["field"].as_str().unwrap(), f(&c["deg"]) as usize, f(&c["ord"]) as usize);
        let want = rows3(&c["acc"]);
        let (field, w) = if fld == "default" { (&fd, &mut wd) } else { (&fs, &mut ws) };
        let mut e = 0.0f64;
        for (r, a_ref) in pos.iter().zip(&want) {
            let a = w.accel(r, field.gm, field.re, deg.min(field.nmax), ord);
            e = e.max(rel(&a, a_ref, 0.0));
            // the one-shot wrapper must give identical bits
            assert_eq!(a, gravity::accel_ecef(field, r, deg, ord));
        }
        println!("sphericalHarmonic {fld:8} {deg:2}x{ord:2}: max rel err {e:.3e}");
        assert!(e <= TOL_GRAV, "{fld} {deg}x{ord}: {e:e}");
        worst = worst.max(e);
    }
    for c in g["pot"].as_array().unwrap() {
        let (fld, n) = (c["field"].as_str().unwrap(), f(&c["nmax"]) as usize);
        let field = if fld == "default" { &fd } else { &fs };
        let mut e = 0.0f64;
        for (r, u_ref) in pos.iter().zip(fv(&c["u"])) {
            let u = gravity::potential(r, field, n);
            e = e.max(((u - u_ref) / u_ref).abs());
        }
        println!("potential {fld:8} nmax {n:2}: max rel err {e:.3e}");
        assert!(e <= TOL_POT, "potential {fld} {n}: {e:e}");
    }
    for c in g["j2accel"].as_array().unwrap() {
        let nj = f(&c["nj"]) as usize;
        let mut e = 0.0f64;
        for (r, a_ref) in pos.iter().zip(rows3(&c["acc"])) {
            let a = gravity::j2accel(r, fd.gm, fd.re, &fd.j[..nj]);
            e = e.max(rel(&a, &a_ref, 0.0));
        }
        println!("j2accel J2..J{}: max rel err {e:.3e}", nj + 1);
        assert!(e <= TOL_GRAV, "j2accel {nj}: {e:e}");
    }
    let mut e = 0.0f64;
    for (r, a_ref) in pos.iter().zip(rows3(&g["twobody"])) {
        e = e.max(rel(&gravity::two_body(r, fd.gm), &a_ref, 0.0));
    }
    println!("twoBody: max rel err {e:.3e}");
    assert!(e <= TOL_GRAV);
    println!("worst sphericalHarmonic rel err {worst:.3e}");
}

#[test]
fn forces_gravity_eci() {
    let g = load("gravity_tides_grav.json");
    let fd = Field::default_field();
    let fs = synth_field(70);
    let mut e = 0.0f64;
    let mut n = 0;
    for c in g["forces"].as_array().unwrap() {
        let field = if c["field"].as_str().unwrap() == "default" { &fd } else { &fs };
        let ord = f(&c["order"]);
        let cfg = GravityForceCfg {
            model: GravityModel::parse(c["model"].as_str().unwrap()).unwrap(),
            degree: f(&c["degree"]) as usize,
            order: if ord < 0.0 { None } else { Some(ord as usize) },
        };
        let a = gravity::accel_eci(field, &v3(&c["r_eci"]), &m3(&c["c"]), &cfg).unwrap();
        let err = rel(&a, &v3(&c["a"]), 0.0);
        assert!(err <= TOL_GRAV, "{c}: {err:e}");
        e = e.max(err);
        n += 1;
    }
    println!("forces.gravity (ECI, {n} cases): max rel err {e:.3e}");
}

fn dcs5(v: &Value) -> Dcs5 {
    assert_eq!(f(&v["n"]), 5.0);
    let (c, s) = (fv(&v["dc"]), fv(&v["ds"]));
    let mut d = Dcs5::default();
    for i in 0..5 {
        for j in 0..5 {
            d.dc[i][j] = c[5 * i + j];
            d.ds[i][j] = s[5 * i + j];
        }
    }
    d
}
/// max |a - b| / max|b| over both matrices.
fn dcs_err(a: &[f64], b: &[f64]) -> f64 {
    let scale = b.iter().fold(0.0f64, |m, x| m.max(x.abs()));
    a.iter().zip(b).fold(0.0f64, |m, (x, y)| m.max((x - y).abs())) / scale
}
fn flat(d: &Dcs5) -> Vec<f64> {
    d.dc.iter().chain(d.ds.iter()).flat_map(|r| r.iter().copied()).collect()
}

#[test]
fn tides() {
    let t = load("gravity_tides_tides.json");
    let (mu, re) = (f(&t["mu"]), f(&t["re"]));
    let fes = FesTable::fes2004_deg10();
    assert_eq!(fes.n.len(), 1052);
    let mut e_dcs = 0.0f64;
    let mut e_fes = 0.0f64;
    let mut e_acc = [0.0f64; 6];
    for ep in t["epochs"].as_array().unwrap() {
        let c = m3(&ep["c"]);
        let (sun, moon) = (v3(&ep["sun_eci"]), v3(&ep["moon_eci"]));
        let (rs, rm) = (adcs_pop::la::mv(&c, &sun), adcs_pop::la::mv(&c, &moon));
        assert_eq!(rs, v3(&ep["sun_ecef"]), "C*sun must be bit-identical to Octave");
        assert_eq!(rm, v3(&ep["moon_ecef"]));
        let d_iers = solidtides::iers2010(&rm, &rs, Some(mu), Some(re));
        let checks: [(Dcs5, &str); 5] = [
            (d_iers, "iers2010"),
            (solidtides::iers2010(&rm, &rs, None, None), "iers2010_defaults"),
            (solidtides::elastic2(&rm, &rs, None, None, None), "elastic2"),
            (solidtides::elastic2(&rm, &rs, Some(0.3), Some(mu), Some(re)), "elastic2_fld"),
            (solidtides::freq_dependent(f(&ep["gmst_rad"]), &d_iers), "freqdep"),
        ];
        for (d, key) in checks.iter() {
            let err = dcs_err(&flat(d), &flat(&dcs5(&ep[*key])));
            assert!(err <= TOL_TIDE, "{key}: {err:e}");
            e_dcs = e_dcs.max(err);
        }
        let tt = f(&ep["tt_jd"]);
        let beta = oceantides::doodson(tt);
        let bref = fv(&ep["doodson"]);
        for k in 0..6 {
            assert!((beta[k] - bref[k]).abs() <= 1e-13 * bref[k].abs().max(1.0), "doodson {k}");
        }
        let ml = oceantides::main_lines(tt);
        let err = dcs_err(&flat(&ml), &flat(&dcs5(&ep["mainlines"])));
        assert!(err <= TOL_TIDE, "mainLines {err:e}");
        e_dcs = e_dcs.max(err);
        let fm = oceantides::from_model(tt, &fes);
        let fr = &ep["frommodel"];
        assert_eq!(f(&fr["n"]) as usize, fm.nmax + 1);
        let a: Vec<f64> = fm.dc.iter().chain(fm.ds.iter()).copied().collect();
        let b: Vec<f64> = fv(&fr["dc"]).into_iter().chain(fv(&fr["ds"])).collect();
        let err = dcs_err(&a, &b);
        assert!(err <= TOL_TIDE, "fromModel {err:e}");
        e_fes = e_fes.max(err);
        // normLegendre
        let pl = solidtides::norm_legendre::<5>(f(&ep["legendre_x"]));
        let pref = fv(&ep["legendre4"]);
        for i in 0..5 {
            for j in 0..5 {
                assert!((pl[i][j] - pref[5 * i + j]).abs() <= 1e-15 * pref[5 * i + j].abs().max(1.0));
            }
        }
        let (fd2c, fd2s) = fm.deg2();
        let (s2c, s2s) = d_iers.deg2();
        let (m2c, m2s) = ml.deg2();
        for p in ep["pos"].as_array().unwrap() {
            let r_ecef = v3(&p["r_ecef"]);
            assert_eq!(r_ecef, adcs_pop::la::mv(&c, &v3(&p["r_eci"])));
            let got = [
                solidtides::accel_from_deg2(&r_ecef, &s2c, &s2s, Some(mu), Some(re)),
                solidtides::solid_tides_accel(&SolidTideInputs { r_ecef, c_eci2ecef: c, sun_eci: sun, moon_eci: moon, mu, re }),
                solidtides::accel_from_deg2(&r_ecef, &m2c, &m2s, Some(mu), Some(re)),
                oceantides::ocean_tides_accel(&OceanTideInputs { tt_jd: tt, r_ecef, c_eci2ecef: c, mu, re }),
                solidtides::accel_from_deg2(&r_ecef, &fd2c, &fd2s, Some(mu), Some(re)),
                solidtides::accel_from_deg2(&r_ecef, &s2c, &s2s, None, None),
            ];
            let keys = ["solid_ecef", "solid_eci", "ocean_ecef", "ocean_eci", "fes_ecef", "solid_ecef_defaults"];
            for k in 0..6 {
                let err = rel(&got[k], &v3(&p[keys[k]]), FLOOR_TIDE);
                assert!(err <= TOL_TIDE, "{} @ {}: {err:e}", keys[k], ep["utc"]);
                e_acc[k] = e_acc[k].max(err);
            }
        }
    }
    println!("tide dcs (iers2010/elastic2/freqDependent/mainLines): max rel err {e_dcs:.3e}");
    println!("fromModel (FES2004 deg 10): max rel err {e_fes:.3e}");
    for (k, name) in ["accelFromDeg2 solid", "forces.solidtides", "accelFromDeg2 ocean", "forces.oceantides", "accelFromDeg2 FES", "accelFromDeg2 defaults"].iter().enumerate() {
        println!("{name}: max rel err {:.3e}", e_acc[k]);
    }
}

/// Cross-check against the recorded MATLAB run force_data/16U_*/gravity.csv
/// (default field, cfg degree 20 -> the 6x6 zonal field; 'gmst' frame, a pure z
/// rotation that a zonal field is invariant to). CSVs carry 10 significant digits.
#[test]
fn recorded_16u_run() {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../matlab_sils/pop/force_data/16U_dria_dtm2020_srp-boxwing_erp-boxwing");
    let Ok(st) = std::fs::read_to_string(dir.join("state.csv")) else {
        println!("16U recording not present; skipped");
        return;
    };
    let gr = std::fs::read_to_string(dir.join("gravity.csv")).unwrap();
    let parse = |s: &str| -> Vec<Vec<f64>> {
        s.lines().skip(1).filter(|l| !l.trim().is_empty()).map(|l| l.split(',').map(|x| x.trim().parse().unwrap()).collect()).collect()
    };
    let (st, gr) = (parse(&st), parse(&gr));
    assert_eq!(st.len(), gr.len());
    let fd = Field::default_field();
    let mut g = gravity::Gravity::new(fd, GravityForceCfg { model: GravityModel::SphHarm, degree: 20, order: None }).unwrap();
    let mut e = 0.0f64;
    let mut e_mag = 0.0f64;
    for (s, a) in st.iter().zip(&gr) {
        let r = [s[1], s[2], s[3]];
        let acc = g.accel_eci(&r, &adcs_pop::la::I3);
        e = e.max(rel(&acc, &[a[1], a[2], a[3]], 0.0));
        e_mag = e_mag.max((vnorm(&acc) - a[4]).abs() / a[4]);
    }
    println!("16U recorded gravity.csv ({} rows): max rel vector err {e:.3e}, |a| err {e_mag:.3e}", st.len());
    assert!(e < 1e-8, "{e:e}");
}

#[test]
fn speed_report() {
    let fd = Field::default_field();
    let fs = synth_field(70);
    let r = [4021987.416987214, -2474343.167990451, -4722155.958392787];
    // central term only at 70x70: the V/W recursion without the accumulation
    let mut fz = fs.clone();
    fz.cbar.iter_mut().skip(1).for_each(|x| *x = 0.0);
    fz.sbar.iter_mut().for_each(|x| *x = 0.0);
    for (name, field, deg, ord) in [
        ("default 6x6", &fd, 6usize, 6usize),
        ("synth 6x6", &fs, 6, 6),
        ("synth 20x20", &fs, 20, 20),
        ("synth 70x0", &fs, 70, 0),
        ("synth 70x70", &fs, 70, 70),
        ("recursion-only 70x70", &fz, 70, 70),
    ] {
        let mut ws = SphHarm::new(field);
        let n = if deg >= 70 { 20_000 } else { 200_000 };
        let mut acc = 0.0;
        let t0 = std::time::Instant::now();
        for i in 0..n {
            let rr = [r[0] + i as f64 * 1e-3, r[1], r[2]];
            acc += ws.accel(&rr, field.gm, field.re, deg, ord)[0];
        }
        let dt = t0.elapsed().as_nanos() as f64 / n as f64;
        println!("SphHarm::accel {name}: {dt:.0} ns/call (checksum {acc:.3e})");
    }
}
