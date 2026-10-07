//! adcs-pop spk / ephem / thirdbody / srp / erp / relativity against the MATLAB POP.
//! Reference data: `refgen/ephem_forces.m` -> `tests/data/ephem_forces.json` and
//! `refgen/ephem_forces_golden.m` -> `tests/data/ephem_forces_golden.json`.
//! Owner: Agastya. Copyright (c) 2026 Agastya. All rights reserved.

use adcs_pop::ephem::{self, Ephem};
use adcs_pop::la::{norm, M3, V3};
use adcs_pop::spk::{chebval, Kernel};
use adcs_pop::{erp, relativity, srp, thirdbody};
use serde_json::Value;
use std::sync::OnceLock;

fn eph() -> &'static Ephem {
    static E: OnceLock<Ephem> = OnceLock::new();
    E.get_or_init(|| Ephem::open_default().expect("de440s.bsp"))
}

/// serde_json (without its `float_roundtrip` feature) may misround 17-digit
/// decimals by an ulp, which is visible at 1e-12 in ill-conditioned formulas. So
/// every bare number is quoted before parsing and read back with the correctly
/// rounded `str::parse::<f64>` in [`f`].
fn quote_numbers(src: &str) -> String {
    let b = src.as_bytes();
    let mut out = String::with_capacity(src.len() + src.len() / 4);
    let (mut i, mut in_str) = (0, false);
    while i < b.len() {
        let c = b[i] as char;
        if in_str {
            out.push(c);
            if c == '\\' { out.push(b[i + 1] as char); i += 1; } else if c == '"' { in_str = false; }
            i += 1;
        } else if c == '"' {
            in_str = true;
            out.push(c);
            i += 1;
        } else if c == '-' || c.is_ascii_digit() {
            let j0 = i;
            while i < b.len() && matches!(b[i], b'0'..=b'9' | b'-' | b'+' | b'.' | b'e' | b'E') { i += 1; }
            out.push('"');
            out.push_str(&src[j0..i]);
            out.push('"');
        } else {
            out.push(c);
            i += 1;
        }
    }
    out
}

fn load(name: &str) -> Value {
    let p = format!("{}/tests/data/{}", env!("CARGO_MANIFEST_DIR"), name);
    let txt = std::fs::read_to_string(&p).unwrap_or_else(|e| panic!("{p}: {e}"));
    serde_json::from_str(&quote_numbers(&txt)).unwrap()
}
fn data() -> &'static Value {
    static D: OnceLock<Value> = OnceLock::new();
    D.get_or_init(|| load("ephem_forces.json"))
}

fn f(v: &Value) -> f64 {
    match v {
        Value::String(s) => s.parse::<f64>().unwrap_or_else(|e| panic!("{s}: {e}")),
        _ => v.as_f64().unwrap(),
    }
}
fn int(v: &Value) -> i32 { f(v) as i32 }
fn v3(v: &Value) -> V3 { let a = v.as_array().unwrap(); [f(&a[0]), f(&a[1]), f(&a[2])] }
fn m3(v: &Value) -> M3 {
    let a: Vec<f64> = v.as_array().unwrap().iter().map(f).collect();
    [[a[0], a[1], a[2]], [a[3], a[4], a[5]], [a[6], a[7], a[8]]]
}
fn vecf(v: &Value) -> Vec<f64> { v.as_array().unwrap().iter().map(f).collect() }

/// Relative vector error |a-b|/|b| (0 when both are zero).
fn rel(a: &V3, b: &V3) -> f64 {
    let d = norm(&[a[0] - b[0], a[1] - b[1], a[2] - b[2]]);
    if d == 0.0 { return 0.0; }
    d / norm(b).max(1e-300)
}

/// Passes when |a-b| <= floor or |a-b|/|b| <= rtol; returns the relative error
/// (for reporting; 0 for exact-zero vectors such as umbra SRP).
#[track_caller]
fn check(what: &str, got: &V3, want: &V3, rtol: f64, floor: f64) -> f64 {
    let d = norm(&[got[0] - want[0], got[1] - want[1], got[2] - want[2]]);
    let e = rel(got, want);
    assert!(d <= floor || e <= rtol, "{what}: rel err {e:.3e} > {rtol:.1e}\n  got  {got:?}\n  want {want:?}");
    e
}

#[track_caller]
fn check_s(what: &str, got: f64, want: f64, rtol: f64, floor: f64) {
    let d = (got - want).abs();
    assert!(d <= floor || d <= rtol * want.abs(), "{what}: got {got:.17e} want {want:.17e}");
}

const RTOL: f64 = 1e-12;
const AFLOOR: f64 = 1e-18; // m/s^2

// ------------------------------------------------------------------ SPK / DE440

#[test]
fn chebval_matches_recurrence() {
    let c = [0.5, -1.25, 3.0, 0.75, -0.1];
    let x = 0.3;
    let (v, d) = chebval(&c, x, 2.0);
    let t = [1.0, x, 2.0 * x * x - 1.0, 4.0 * x * x * x - 3.0 * x, 8.0 * x.powi(4) - 8.0 * x * x + 1.0];
    let dt = [0.0, 1.0, 4.0 * x, 12.0 * x * x - 3.0, 32.0 * x.powi(3) - 16.0 * x];
    let ve: f64 = c.iter().zip(t.iter()).map(|(a, b)| a * b).sum();
    let de: f64 = c.iter().zip(dt.iter()).map(|(a, b)| a * b).sum::<f64>() / 2.0;
    assert!((v - ve).abs() < 1e-14 && (d - de).abs() < 1e-14);
}

#[test]
fn kernel_segments() {
    let k = eph().kernel();
    let keys: Vec<(i32, i32)> = k.segments().iter().map(|s| (s.center, s.target)).collect();
    for want in [(0, 1), (0, 10), (0, 3), (3, 301), (3, 399), (1, 199), (2, 299)] {
        assert!(keys.contains(&want), "missing segment {want:?}");
    }
    assert_eq!(keys.len(), 14);
    assert!(k.segments().iter().all(|s| s.data_type == 2 && s.frame == 1));
    assert!(Kernel::from_bytes(b"NOT A DAF".to_vec()).is_err());
}

#[test]
fn spk_states_match_matlab() {
    let e = eph();
    let mut worst = (0.0f64, 0.0f64);
    for s in data()["spk"].as_array().unwrap() {
        let (c, t, jd) = (int(&s["center"]), int(&s["target"]), f(&s["jd"]));
        let (p, v) = e.state(c, t, jd).unwrap();
        worst.0 = worst.0.max(check(&format!("pos {c}->{t} @ {jd}"), &p, &v3(&s["pos"]), 1e-14, 0.0));
        worst.1 = worst.1.max(check(&format!("vel {c}->{t} @ {jd}"), &v, &v3(&s["vel"]), 1e-13, 0.0));
    }
    eprintln!("spk worst rel: pos {:.2e}  vel {:.2e}", worst.0, worst.1);
}

#[test]
fn ephem_and_inputs_match_matlab() {
    let e = eph();
    let k = &data()["constants"];
    let c = ephem::constants();
    for (name, got) in [("AU_m", c.au_m), ("c", c.c), ("GM_sun", c.gm_sun), ("GM_earth", c.gm_earth), ("GM_moon", c.gm_moon),
        ("EMRAT", c.emrat), ("TSI", c.tsi), ("P0", c.p0), ("Re_earth", c.re_earth), ("f_earth", c.f_earth),
        ("Rp_earth", c.rp_earth), ("mu_earth", c.mu_earth), ("omega_earth", c.omega_earth), ("N_A", c.n_a), ("Rsun", c.rsun)] {
        assert_eq!(got, f(&k[name]), "constant {name}");
    }
    let mut worst = 0.0f64;
    for s in data()["ephem"].as_array().unwrap() {
        let jd = f(&s["jd"]);
        let (rs, vs) = e.sun(jd);
        let (rm, vm) = e.moon(jd);
        let (re, ve) = e.earth(jd);
        for (n, g, w) in [("sun_r", rs, "sun_r"), ("sun_v", vs, "sun_v"), ("moon_r", rm, "moon_r"), ("moon_v", vm, "moon_v"),
                          ("earth_r", re, "earth_r"), ("earth_v", ve, "earth_v")] {
            worst = worst.max(check(n, &g, &v3(&s[w]), 1e-13, 0.0));
        }
        let i = ephem::inputs(e, jd);
        assert_eq!(i.jd_tdb, jd);
        for (n, g) in [("sun_unit", i.sun_unit), ("sun_eci", i.sun_eci), ("sun_vel", i.sun_vel), ("moon_eci", i.moon_eci),
            ("moon_vel", i.moon_vel), ("tide_sun", i.tide_sun), ("tide_moon", i.tide_moon), ("albedo_sun_unit", i.albedo_sun_unit),
            ("earth_helio_pos", i.earth_helio_pos), ("earth_helio_vel", i.earth_helio_vel)] {
            check(n, &g, &v3(&s[n]), 1e-13, 0.0);
        }
        for (n, g) in [("sun_dist", i.sun_dist), ("flux_scale", i.flux_scale), ("P_srp", i.p_srp), ("GM_sun", i.gm_sun),
                       ("GM_moon", i.gm_moon), ("sun_ra", i.sun_ra), ("sun_dec", i.sun_dec)] {
            check_s(n, g, f(&s[n]), 1e-13, 0.0);
        }
    }
    eprintln!("de440 sun/moon/earth worst rel {worst:.2e}");
}

// ------------------------------------------------------------------ forces

struct Case<'a> {
    c: &'a Value,
    r: V3,
    v: V3,
    e: ephem::EphemInputs,
    sc: srp::Spacecraft,
}

fn cases() -> Vec<Case<'static>> {
    data()["cases"].as_array().unwrap().iter().map(|c| {
        let mut sc = srp::sat16u();
        sc.r_bi = m3(&c["R_bi"]);
        Case { c, r: v3(&c["r"]), v: v3(&c["v"]), e: ephem::inputs(eph(), f(&c["jd"])), sc }
    }).collect()
}

#[test]
fn thirdbody_matches_matlab() {
    use thirdbody::Model::*;
    let mut worst = 0.0f64;
    let mut worst_direct = 0.0f64;
    for k in cases() {
        let (r, e, c) = (&k.r, &k.e, k.c);
        for (key, body, gm, m) in [
            ("tb_battin_sun", e.sun_eci, e.gm_sun, Battin), ("tb_battin_moon", e.moon_eci, e.gm_moon, Battin),
            ("tb_direct_sun", e.sun_eci, e.gm_sun, Direct), ("tb_direct_moon", e.moon_eci, e.gm_moon, Direct),
            ("tb_tidal_sun", e.sun_eci, e.gm_sun, Tidal), ("tb_tidal_moon", e.moon_eci, e.gm_moon, Tidal),
            ("tb_legendre_sun", e.sun_eci, e.gm_sun, Legendre(4)), ("tb_legendre_moon", e.moon_eci, e.gm_moon, Legendre(4)),
            ("tb_legendre8_moon", e.moon_eci, e.gm_moon, Legendre(8)),
        ] {
            // direct loses ~5 digits for the Sun by construction (MATLAB's own
            // cancellation); it holds 1e-12 only because the norm is Octave's
            let e = check(key, &thirdbody::accel(r, &body, gm, m), &v3(&c[key]), RTOL, AFLOOR);
            if key == "tb_direct_sun" { worst_direct = worst_direct.max(e) } else { worst = worst.max(e) }
        }
        let inp = thirdbody::ThirdBodyInput::new(k.r, e, thirdbody::Model::from_name("battin").unwrap());
        check("f_thirdbody", &thirdbody::force(&inp), &v3(&c["f_thirdbody"]), RTOL, AFLOOR);
        let inp = thirdbody::ThirdBodyInput { model: thirdbody::Model::from_name("Legendre").unwrap(), ..inp };
        check("f_thirdbody_legendre", &thirdbody::force(&inp), &v3(&c["f_thirdbody_legendre"]), RTOL, AFLOOR);
    }
    eprintln!("thirdbody worst rel {worst:.2e} (direct/Sun {worst_direct:.2e})");
}

#[test]
fn eclipse_and_srp_match_matlab() {
    use srp::EclipseModel as EM;
    let o = srp::EclipseOpts::default();
    let (mut n_umbra, mut n_pen, mut n_sun) = (0, 0, 0);
    let mut worst = 0.0f64;
    for k in cases() {
        let c = k.c;
        let tag = c["tag"].as_str().unwrap();
        for (key, m) in [("nu_cylindrical", EM::Cylindrical), ("nu_conical", EM::Conical), ("nu_fine", EM::Fine), ("nu_default", EM::default())] {
            // penumbra_* cases were bisected to nu = 0.5 in MATLAB: fraction is steep there
            let tol = 1e-12;
            check_s(&format!("{key} ({tag})"), srp::eclipse(&k.r, &k.e.sun_eci, m, &o), f(&c[key]), tol, 1e-15);
        }
        let nu = f(&c["nu_conical"]);
        if nu == 0.0 { n_umbra += 1 } else if nu == 1.0 { n_sun += 1 } else { n_pen += 1 }
        let ecl = EM::from_name(c["eclipse_model"].as_str().unwrap()).unwrap();
        let tol = RTOL;
        let mut inp = srp::SrpInput::new(k.r, &k.e, &k.sc, srp::SrpModel::from_name("cannonball").unwrap(), ecl);
        worst = worst.max(check("f_srp_cannonball", &srp::force(&inp), &v3(&c["f_srp_cannonball"]), tol, AFLOOR));
        inp.cr = Some(1.7);
        check("f_srp_cannonball_cr17", &srp::force(&inp), &v3(&c["f_srp_cannonball_cr17"]), tol, AFLOOR);
        inp.cr = None;
        inp.model = srp::SrpModel::Boxwing;
        worst = worst.max(check("f_srp_boxwing", &srp::force(&inp), &v3(&c["f_srp_boxwing"]), tol, AFLOOR));
        let d = [k.e.sun_eci[0] - k.r[0], k.e.sun_eci[1] - k.r[1], k.e.sun_eci[2] - k.r[2]];
        let nd = norm(&d);
        let shat = [d[0] / nd, d[1] / nd, d[2] / nd];
        let (a, fo) = srp::boxwing(k.e.p_srp, 0.37, &shat, &k.sc.r_bi, &k.sc.facets, k.sc.mass);
        worst = worst.max(check("srp_boxwing_a", &a, &v3(&c["srp_boxwing_a"]), RTOL, AFLOOR));
        check("srp_boxwing_F", &fo, &v3(&c["srp_boxwing_F"]), RTOL, 1e-16);
    }
    eprintln!("eclipse cases: sunlit {n_sun}, penumbra {n_pen}, umbra {n_umbra}; srp worst rel {worst:.2e}");
    assert!(n_umbra >= 9 && n_pen >= 9 && n_sun >= 9);
}

#[test]
fn erp_matches_matlab() {
    let mut worst = 0.0f64;
    for k in cases() {
        let (c, r, s) = (k.c, &k.r, &k.e.sun_eci);
        let doy = f(&c["doy"]);
        let craom = 1.3 * k.sc.aref / k.sc.mass;
        let (a, comp) = erp::knocke(r, s, craom, doy, erp::NRINGS, erp::NSEG);
        worst = worst.max(check("erp_knocke", &a, &v3(&c["erp_knocke"]), RTOL, AFLOOR));
        check("erp_knocke_sw", &comp.sw, &v3(&c["erp_knocke_sw"]), RTOL, AFLOOR);
        check("erp_knocke_lw", &comp.lw, &v3(&c["erp_knocke_lw"]), RTOL, AFLOOR);
        check("erp_simple", &erp::simple(r, s, craom, doy).0, &v3(&c["erp_simple"]), RTOL, AFLOOR);
        check("erp_ceres", &erp::ceres(r, s, craom, None, erp::NRINGS, erp::NSEG).0, &v3(&c["erp_ceres"]), RTOL, AFLOOR);
        let (a, comp) = erp::boxwing(r, s, &k.sc.r_bi, &k.sc.facets, k.sc.mass, doy, erp::NRINGS, erp::NSEG);
        worst = worst.max(check("erp_boxwing", &a, &v3(&c["erp_boxwing"]), RTOL, AFLOOR));
        check("erp_boxwing_sw", &comp.sw, &v3(&c["erp_boxwing_sw"]), RTOL, AFLOOR);
        check("erp_boxwing_lw", &comp.lw, &v3(&c["erp_boxwing_lw"]), RTOL, AFLOOR);
        let mut inp = erp::ErpInput::new(k.r, &k.e, doy, &k.sc, erp::Model::from_name("boxwing").unwrap());
        inp.nrings = Some(8);
        inp.nseg = Some(16);
        check("f_erp_boxwing_8x16", &erp::force(&inp), &v3(&c["f_erp_boxwing_8x16"]), RTOL, AFLOOR);
        let inp = erp::ErpInput { model: erp::Model::Knocke, cr: Some(1.1), ..inp };
        check("f_erp_knocke_cr11", &erp::force(&inp), &v3(&c["f_erp_knocke_cr11"]), RTOL, AFLOOR);
        // erp.accel dispatch
        let a = erp::accel(r, s, craom, erp::Model::Simple, &k.sc.r_bi, &k.sc, doy, 16, 48).0;
        check("erp_accel_simple", &a, &v3(&c["erp_simple"]), RTOL, AFLOOR);
    }
    eprintln!("erp worst rel {worst:.2e}");
}

#[test]
fn ceres_grid_is_used() {
    let r = [6.9e6, 1.0e5, 2.0e5];
    let s = [1.4e11, 2.0e10, 1.0e10];
    let g = |_lat: f64, _lon: f64| (0.0, 0.0);
    let (a, _) = erp::ceres(&r, &s, 0.01, Some(&g), 16, 48);
    assert_eq!(a, [0.0; 3]);
}

#[test]
fn relativity_matches_matlab() {
    use relativity::Term;
    let mu = f(&data()["mu_field"]);
    let mut worst = 0.0f64;
    for k in cases() {
        let c = k.c;
        let (a, parts) = relativity::total(&k.r, &k.v, Some((&k.e.earth_helio_pos, &k.e.earth_helio_vel)), &relativity::ALL_TERMS, mu).unwrap();
        worst = worst.max(check("rel_total", &a, &v3(&c["rel_total"]), RTOL, AFLOOR));
        check("rel_schwarzschild", &parts.schwarzschild.unwrap(), &v3(&c["rel_schwarzschild"]), RTOL, AFLOOR);
        check("rel_lensethirring", &parts.lensethirring.unwrap(), &v3(&c["rel_lensethirring"]), RTOL, AFLOOR);
        check("rel_desitter", &parts.desitter.unwrap(), &v3(&c["rel_desitter"]), RTOL, AFLOOR);
        let inp = relativity::RelativityInput::new(k.r, k.v, Some(&k.e), &relativity::FORCE_DEFAULT_TERMS, mu);
        check("f_relativity", &relativity::force(&inp).unwrap(), &v3(&c["f_relativity"]), RTOL, AFLOOR);
        let mue = ephem::constants().mu_earth;
        check("rel_lt_default", &relativity::lense_thirring(&k.r, &k.v, mue, &relativity::j_earth(), 1.0), &v3(&c["rel_lt_default"]), RTOL, AFLOOR);
        check("rel_sch_default", &relativity::schwarzschild(&k.r, &k.v, mue, 1.0, 1.0), &v3(&c["rel_sch_default"]), RTOL, AFLOOR);
        let terms = [Term::from_name("DeSitter").unwrap()];
        assert!(relativity::total(&k.r, &k.v, None, &terms, mu).is_err());
    }
    eprintln!("relativity worst rel {worst:.2e}");
}

#[test]
fn secular_matches_matlab() {
    use thirdbody::secular as s;
    let d = &data()["secular"];
    let k = ephem::constants();
    let e = ephem::inputs(eph(), f(&data()["ephem"][4]["jd"]));
    let phis = s::phi_quad(k.re_earth + 550e3, k.gm_sun, norm(&e.sun_eci), Some(0.0167), None);
    let phim = s::phi_quad(k.re_earth + 550e3, k.gm_moon, norm(&e.moon_eci), None, None);
    check_s("phiS", phis, f(&d["phiS"]), 1e-12, 0.0);
    check_s("phiM", phim, f(&d["phiM"]), 1e-12, 0.0);
    let (j, ev) = s::elem2vectors(7e6, 0.3, 1.1, 0.4, 0.9);
    check("j", &j, &v3(&d["j"]), 1e-15, 0.0);
    check("e", &ev, &v3(&d["e"]), 1e-15, 0.0);
    let (ecc, inc, om, w) = s::vectors2elem(&j, &ev);
    for (n, g) in [("ecc", ecc), ("inc", inc), ("Om", om), ("w", w)] { check_s(n, g, f(&d[n]), 1e-14, 0.0); }
    let ns = [0.0, -0.3977771559, 0.9174820621];
    let nn = norm(&ns);
    let ns = [ns[0] / nn, ns[1] / nn, ns[2] / nn];
    let (dj, de) = s::kozai_rates(&j, &ev, &ns, phis);
    check("dj", &dj, &v3(&d["dj"]), 1e-12, 0.0);
    check("de", &de, &v3(&d["de"]), 1e-12, 0.0);
    let p = [s::Perturber { nhat: ns, phi_q: phis * 1e3 }, s::Perturber { nhat: [0.1, -0.35, 0.93], phi_q: phim * 1e3 }];
    let out = s::propagate(k.re_earth + 550e3, 0.01, 63f64.to_radians(), 0.2, 0.5, &p, 86400.0 * 30.0, 200);
    for (n, g) in [("prop_t", &out.t), ("prop_e", &out.e), ("prop_inc", &out.inc), ("prop_Om", &out.om), ("prop_w", &out.w), ("prop_kozai", &out.kozai)] {
        let w = vecf(&d[n]);
        assert_eq!(g.len(), w.len());
        for (a, b) in g.iter().zip(w.iter()) { check_s(n, *a, *b, 1e-10, 1e-13); }
    }
}

// ------------------------------------------------------------------ golden POP dataset

/// force_data/16U_dria_dtm2020_srp-boxwing_erp-boxwing: the Rust forces from the CSV
/// state with the reconstructed inputs (TDB JD, doy, ram R_bi, field mu) against
/// (a) MATLAB op.accel re-evaluated at the same state (tight) and (b) the golden
/// CSV printed by TEMPLATE_16U (10 significant digits).
#[test]
fn golden_16u_dataset() {
    let d = load("ephem_forces_golden.json");
    let mu = f(&d["mu_field"]);
    let mut sc = srp::sat16u();
    let terms = [relativity::Term::Schwarzschild];
    let mut w_m = [0.0f64; 4];
    let mut w_g = [0.0f64; 4];
    for row in d["rows"].as_array().unwrap() {
        let (r, v) = (v3(&row["r"]), v3(&row["v"]));
        let e = ephem::inputs(eph(), f(&row["jd_tdb"]));
        sc.r_bi = m3(&row["R_bi"]);
        let tb = thirdbody::force(&thirdbody::ThirdBodyInput::new(r, &e, thirdbody::Model::Battin));
        let sr = srp::force(&srp::SrpInput::new(r, &e, &sc, srp::SrpModel::Boxwing, srp::EclipseModel::Conical));
        let mut ei = erp::ErpInput::new(r, &e, f(&row["doy"]), &sc, erp::Model::Boxwing);
        ei.nrings = Some(8);
        ei.nseg = Some(16);
        let er = erp::force(&ei);
        let rl = relativity::force(&relativity::RelativityInput::new(r, v, Some(&e), &terms, mu)).unwrap();
        for (i, (name, got)) in [("thirdbody", tb), ("srp", sr), ("erp", er), ("relativity", rl)].iter().enumerate() {
            let m = v3(&row[format!("m_{name}").as_str()]);
            let g = v3(&row[format!("g_{name}").as_str()]);
            w_m[i] = w_m[i].max(check(&format!("golden {name} vs MATLAB t={}", row["t"]), got, &m, RTOL, AFLOOR));
            // CSV carries 10 significant digits; srp rows in the penumbra amplify the
            // state's 10-digit rounding through the eclipse fraction
            let e = rel(got, &g);
            w_g[i] = w_g[i].max(e);
            assert!(e < if *name == "srp" { 1e-5 } else { 1e-8 }, "golden {name} vs CSV rel {e:.2e} at t={}", row["t"]);
        }
    }
    eprintln!("golden 16U vs MATLAB re-eval  (thirdbody, srp, erp, relativity): {:.1e} {:.1e} {:.1e} {:.1e}", w_m[0], w_m[1], w_m[2], w_m[3]);
    eprintln!("golden 16U vs CSV (10 digits) (thirdbody, srp, erp, relativity): {:.1e} {:.1e} {:.1e} {:.1e}", w_g[0], w_g[1], w_g[2], w_g[3]);
}

#[test]
fn hot_path_speed() {
    let e = eph();
    let t0 = std::time::Instant::now();
    let n = 200_000;
    let mut acc = 0.0;
    for i in 0..n {
        let x = ephem::inputs(e, 2461406.75 + i as f64 * 1e-4);
        acc += x.sun_eci[0] + x.moon_eci[1];
    }
    let dt = t0.elapsed().as_secs_f64() / n as f64;
    eprintln!("ephem::inputs {:.2} us/call ({acc:.1})", dt * 1e6);
    let x = ephem::inputs(e, 2461406.75);
    let sc = srp::sat16u();
    let r = [6.9e6, 1.0e5, 2.0e5];
    let t0 = std::time::Instant::now();
    let n = 2000;
    for i in 0..n {
        let r = [r[0], r[1] + i as f64, r[2]];
        let mut ei = erp::ErpInput::new(r, &x, 1.0, &sc, erp::Model::Boxwing);
        ei.nrings = Some(8);
        ei.nseg = Some(16);
        acc += erp::force(&ei)[0];
    }
    eprintln!("erp boxwing 8x16 x 8 facets {:.2} us/call ({acc:.1e})", t0.elapsed().as_secs_f64() / n as f64 * 1e6);
}
