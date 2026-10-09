//! Atmosphere / space weather / drag against the MATLAB POP (Octave reference vectors
//! printed by refgen/atmos_drag.m into tests/data/atmos_drag.json).
use adcs_pop::atmos::dtm2020::{self, KpIn};
use adcs_pop::atmos::dtm2020_research::{self, Ap60In};
use adcs_pop::atmos::jb2008::{self, JbIndices};
use adcs_pop::atmos::{self, exponential, AtmosModel, AtmosOut, Composition, Geo};
use adcs_pop::drag::geom::{self, Facet};
use adcs_pop::drag::gsi::{self, Gsi, PanelModel};
use adcs_pop::drag::{self, DragConfig, DragInput, DragModel, PanelOpts, Spacecraft, SwSources};
use adcs_pop::la::{M3, V3};
use adcs_pop::spaceweather::{self, ManualIndices, ResearchSw, SwOpts, SwTable};
use json::Value;
use std::sync::OnceLock;

const TOL: f64 = 1e-12;

/// A minimal JSON reader whose numbers go through `str::parse::<f64>` (correctly
/// rounded). serde_json's default float parser is only "best effort" and can be one
/// ulp off for 17-digit numbers -- enough to flip the sign of a 1e-17 dot product.
mod json {
    #[derive(Debug, Clone)]
    pub enum Value {
        Null,
        Bool(bool),
        Num(f64),
        Str(String),
        Arr(Vec<Value>),
        Obj(Vec<(String, Value)>),
    }
    static NULL: Value = Value::Null;
    impl Value {
        pub fn as_f64(&self) -> Option<f64> {
            if let Value::Num(x) = self { Some(*x) } else { None }
        }
        pub fn as_bool(&self) -> Option<bool> {
            if let Value::Bool(x) = self { Some(*x) } else { None }
        }
        pub fn as_str(&self) -> Option<&str> {
            if let Value::Str(x) = self { Some(x) } else { None }
        }
        pub fn as_array(&self) -> Option<&Vec<Value>> {
            if let Value::Arr(x) = self { Some(x) } else { None }
        }
        pub fn is_null(&self) -> bool {
            matches!(self, Value::Null)
        }
        pub fn get(&self, k: &str) -> Option<&Value> {
            if let Value::Obj(o) = self { o.iter().find(|(n, _)| n == k).map(|(_, v)| v) } else { None }
        }
    }
    impl std::ops::Index<&str> for Value {
        type Output = Value;
        fn index(&self, k: &str) -> &Value {
            self.get(k).unwrap_or(&NULL)
        }
    }
    impl std::ops::Index<usize> for Value {
        type Output = Value;
        fn index(&self, i: usize) -> &Value {
            self.as_array().and_then(|a| a.get(i)).unwrap_or(&NULL)
        }
    }
    impl std::fmt::Display for Value {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{self:?}")
        }
    }
    pub fn parse(s: &str) -> Value {
        let b = s.as_bytes();
        let mut i = 0;
        val(b, &mut i)
    }
    fn ws(b: &[u8], i: &mut usize) {
        while *i < b.len() && (b[*i] as char).is_ascii_whitespace() {
            *i += 1;
        }
    }
    fn val(b: &[u8], i: &mut usize) -> Value {
        ws(b, i);
        match b[*i] {
            b'{' => {
                *i += 1;
                let mut o = Vec::new();
                loop {
                    ws(b, i);
                    if b[*i] == b'}' {
                        *i += 1;
                        break;
                    }
                    let k = match val(b, i) {
                        Value::Str(s) => s,
                        _ => panic!("json key"),
                    };
                    ws(b, i);
                    assert_eq!(b[*i], b':');
                    *i += 1;
                    let v = val(b, i);
                    o.push((k, v));
                    ws(b, i);
                    if b[*i] == b',' {
                        *i += 1;
                    }
                }
                Value::Obj(o)
            }
            b'[' => {
                *i += 1;
                let mut a = Vec::new();
                loop {
                    ws(b, i);
                    if b[*i] == b']' {
                        *i += 1;
                        break;
                    }
                    a.push(val(b, i));
                    ws(b, i);
                    if b[*i] == b',' {
                        *i += 1;
                    }
                }
                Value::Arr(a)
            }
            b'"' => {
                *i += 1;
                let st = *i;
                while b[*i] != b'"' {
                    *i += 1;
                }
                let s = String::from_utf8(b[st..*i].to_vec()).unwrap();
                *i += 1;
                Value::Str(s)
            }
            b't' => {
                *i += 4;
                Value::Bool(true)
            }
            b'f' => {
                *i += 5;
                Value::Bool(false)
            }
            b'n' => {
                *i += 4;
                Value::Null
            }
            _ => {
                let st = *i;
                while *i < b.len() && matches!(b[*i], b'0'..=b'9' | b'-' | b'+' | b'.' | b'e' | b'E') {
                    *i += 1;
                }
                let t = std::str::from_utf8(&b[st..*i]).unwrap();
                Value::Num(t.parse::<f64>().unwrap_or_else(|_| panic!("json number {t}")))
            }
        }
    }
}

fn data() -> &'static Value {
    static D: OnceLock<Value> = OnceLock::new();
    D.get_or_init(|| {
        let p = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/data/atmos_drag.json");
        json::parse(&std::fs::read_to_string(p).expect("read atmos_drag.json"))
    })
}

fn f(v: &Value) -> f64 {
    v.as_f64().unwrap_or(f64::NAN)
}
fn vecn(v: &Value) -> Vec<f64> {
    v.as_array().expect("array").iter().map(f).collect()
}
fn v3(v: &Value) -> V3 {
    let a = vecn(v);
    [a[0], a[1], a[2]]
}
fn m3(v: &Value) -> M3 {
    let r = v.as_array().expect("matrix");
    [v3(&r[0]), v3(&r[1]), v3(&r[2])]
}
fn utc6(v: &Value) -> [f64; 6] {
    let a = vecn(v);
    [a[0], a[1], a[2], a[3], a[4], a[5]]
}

/// Worst relative error tracker; zero references demand an (almost) zero result.
struct Err {
    name: &'static str,
    max: f64,
    n: usize,
}
impl Err {
    fn new(name: &'static str) -> Self {
        Err { name, max: 0.0, n: 0 }
    }
    fn chk(&mut self, got: f64, want: f64, ctx: &str) {
        self.n += 1;
        let e = if want == 0.0 {
            got.abs()
        } else if want.is_nan() {
            if got.is_nan() { 0.0 } else { f64::INFINITY }
        } else {
            ((got - want) / want).abs()
        };
        if e > self.max {
            self.max = e;
        }
        assert!(e <= TOL, "{}: {} got {:.17e} want {:.17e} (rel {:.3e})", self.name, ctx, got, want, e);
    }
    fn chk3(&mut self, got: &[f64], want: &[f64], ctx: &str) {
        // vectors: relative to the vector's magnitude
        let scale = want.iter().fold(0.0f64, |m, x| m.max(x.abs()));
        for (g, w) in got.iter().zip(want) {
            self.n += 1;
            let e = if scale == 0.0 { g.abs() } else { (g - w).abs() / scale };
            if e > self.max {
                self.max = e;
            }
            assert!(e <= TOL, "{}: {} got {:?} want {:?} (rel {:.3e})", self.name, ctx, got, want, e);
        }
    }
    fn report(&self) {
        println!("{:<28} {:>7} values  max rel err {:.3e}", self.name, self.n, self.max);
    }
}

fn kp_in(v: &Value) -> KpIn {
    let k = vecn(v);
    if k.len() == 4 { KpIn::Akp([k[0], k[1], k[2], k[3]]) } else { KpIn::Scalar(k[0]) }
}

#[test]
fn dtm2020_operational() {
    let st = dtm2020::oper_coeffs();
    let mut e = Err::new("dtm2020 oper (dtm3)");
    for c in data()["dtm_oper"].as_array().unwrap() {
        let o = dtm2020::oper_density(f(&c["alt"]), f(&c["lat"]), f(&c["lon"]), f(&c["lst"]), f(&c["doy"]), f(&c["f"]), f(&c["fbar"]), kp_in(&c["kp"]), st)
            .unwrap();
        e.chk(o.rho_kgm3, f(&c["rho"]), "rho");
        e.chk(o.t_k, f(&c["T"]), "T");
        e.chk(o.tinf_k, f(&c["Tinf"]), "Tinf");
        e.chk(o.mbar_amu, f(&c["mbar"]), "mbar");
        for (i, w) in vecn(&c["n"]).iter().enumerate() {
            e.chk(o.n_cm3[i], *w, "n");
        }
    }
    e.report();
    assert!(dtm2020::oper_density(120.0, 0.0, 0.0, 0.0, 1.0, 100.0, 100.0, KpIn::Scalar(2.0), st).is_err());
}

fn manual(v: &Value) -> ManualIndices {
    let g = |k: &str| v.get(k).map(f);
    ManualIndices { f107: g("F107").unwrap(), f107a: g("F107a"), kp: g("Kp").map(KpIn::Scalar), ap: g("ap"), ap3: g("ap3") }
}

fn geo(v: &Value) -> Geo {
    Geo { alt_km: f(&v["alt_km"]), lat_deg: f(&v["lat_deg"]), lon_deg: f(&v["lon_deg"]), lst_h: f(&v["lst_h"]), doy: f(&v["doy"]), utc: utc6(&v["utc"]) }
}

fn species(a: &AtmosOut) -> [f64; 6] {
    match a.comp {
        Composition::Species(n) => n,
        _ => panic!("expected species"),
    }
}

#[test]
fn dtm2020_provider_manual() {
    let mut e = Err::new("atmos.dtm2020 + manual sw");
    for c in data()["dtm_oper_atm"].as_array().unwrap() {
        let sw = spaceweather::from_manual(&manual(&c["manual"])).unwrap();
        let g = geo(&c["geo"]);
        let a = atmos::density(AtmosModel::Dtm2020, &g, &atmos::AtmosDrivers::Sw(&sw)).unwrap();
        e.chk(a.rho, f(&c["rho"]), "rho");
        e.chk(a.t, f(&c["T"]), "T");
        for (i, w) in vecn(&c["n"]).iter().enumerate() {
            e.chk(species(&a)[i], *w, "n [m^-3]");
        }
    }
    e.report();
}

#[test]
fn dtm2020_research_model() {
    let st = dtm2020_research::research_coeffs();
    let mut e = Err::new("dtm2020 research (dtm5)");
    for c in data()["dtm_res"].as_array().unwrap() {
        let a = vecn(&c["kp"]);
        let ap = if a.len() == 10 {
            let mut x = [0.0; 10];
            x.copy_from_slice(&a);
            Ap60In::Array(x)
        } else {
            Ap60In::Scalar(a[0])
        };
        let o = dtm2020_research::density(f(&c["alt"]), f(&c["lat"]), f(&c["lon"]), f(&c["lst"]), f(&c["doy"]), f(&c["f"]), f(&c["fbar"]), ap, st).unwrap();
        e.chk(o.rho_kgm3, f(&c["rho"]), "rho");
        e.chk(o.t_k, f(&c["T"]), "T");
        e.chk(o.tinf_k, f(&c["Tinf"]), "Tinf");
        e.chk(o.mbar_amu, f(&c["mbar"]), "mbar");
        for (i, w) in vecn(&c["n"]).iter().enumerate() {
            e.chk(o.n_cm3[i], *w, "n");
        }
    }
    e.report();
    let mut e = Err::new("atmos.dtm2020_research");
    for c in data()["dtm_res_atm"].as_array().unwrap() {
        let sw = ResearchSw { f30: f(&c["F30"]), f30_bar: f(&c["F30_bar"]), ap60: f(&c["ap60"]), f30_is_derived: c["f30_is_derived"].as_bool().unwrap() };
        let a = atmos::density(AtmosModel::Dtm2020Research, &geo(&c["geo"]), &atmos::AtmosDrivers::Research(&sw)).unwrap();
        e.chk(a.rho, f(&c["rho"]), "rho");
        e.chk(a.t, f(&c["T"]), "T");
        for (i, w) in vecn(&c["n"]).iter().enumerate() {
            e.chk(species(&a)[i], *w, "n");
        }
    }
    e.report();
    let mut e = Err::new("bint_oe / geogm");
    let b = &data()["bint_oe"];
    for (ap, kp) in vecn(&b["ap"]).iter().zip(vecn(&b["kp"])) {
        e.chk(dtm2020_research::bint_oe(*ap), kp, "bint_oe");
    }
    let g = &data()["geogm"];
    let (la, lo, gl, gg) = (vecn(&g["lat"]), vecn(&g["lon"]), vecn(&g["gmlat"]), vecn(&g["gmlon"]));
    for i in 0..la.len() {
        let (a, b) = dtm2020_research::geogm(la[i], lo[i]).unwrap();
        e.chk(a, gl[i], "gmlat");
        e.chk(b, gg[i], "gmlon");
    }
    e.report();
}

#[test]
fn jb2008_model() {
    let idx = JbIndices::bundled();
    let mut e = Err::new("jb2008 (wrapper + core)");
    let mut ec = Err::new("jb2008 core only");
    for c in data()["jb2008"].as_array().unwrap() {
        let (rho, out, inp) = jb2008::jb2008_density(&utc6(&c["utc"]), f(&c["lon"]), f(&c["lat"]), f(&c["alt"]), &idx);
        let core = &c["core"];
        e.chk(inp.mjd, f(&core["mjd"]), "mjd");
        for (g, w) in inp.sun.iter().zip(vecn(&core["sun"])) {
            e.chk(*g, w, "sun");
        }
        for (g, w) in inp.sat.iter().zip(vecn(&core["sat"])) {
            e.chk(*g, w, "sat");
        }
        for (g, w) in inp.ind.iter().zip(vecn(&core["ind"])) {
            e.chk(*g, w, "indices");
        }
        e.chk(rho, f(&c["rho"]), "rho");
        for (g, w) in out.temp.iter().zip(vecn(&core["temp"])) {
            e.chk(*g, w, "temp");
        }
        // the core alone, on the exact MATLAB inputs
        let s = vecn(&core["sun"]);
        let t = vecn(&core["sat"]);
        let x = vecn(&core["ind"]);
        let o = jb2008::jb2008_core(f(&core["mjd"]), [s[0], s[1]], [t[0], t[1], t[2]], x[0], x[1], x[2], x[3], x[4], x[5], x[6], x[7], x[8]);
        ec.chk(o.rho, f(&core["rho"]), "rho");
        ec.chk(o.temp[0], vecn(&core["temp"])[0], "Tinf");
        ec.chk(o.temp[1], vecn(&core["temp"])[1], "T");
    }
    e.report();
    ec.report();
    // atmos.jb2008 adapter: Mmol 16, nO = rho*N_A*1000/16, T placeholder 1000 K
    let c = &data()["jb2008"][5];
    let g = Geo { alt_km: f(&c["alt"]), lat_deg: f(&c["lat"]), lon_deg: f(&c["lon"]), lst_h: 0.0, doy: 1.0, utc: utc6(&c["utc"]) };
    let a = atmos::density(AtmosModel::Jb2008, &g, &atmos::AtmosDrivers::Jb(&idx, None)).unwrap();
    assert_eq!(a.t, 1000.0);
    match a.comp {
        Composition::Mean { mmol, n_o } => {
            assert_eq!(mmol, 16.0);
            assert!(((n_o - a.rho * f(&data()["jb_nO_factor"])) / n_o).abs() < 1e-15);
        }
        _ => panic!(),
    }
}

#[test]
fn exponential_model() {
    let x = &data()["exponential"];
    let mut e = Err::new("atmos.exponential");
    let (alt, rho, t, mm, no) = (vecn(&x["alt"]), vecn(&x["rho"]), vecn(&x["T"]), vecn(&x["Mmol"]), vecn(&x["nO"]));
    for i in 0..alt.len() {
        let a = exponential::exponential(alt[i]);
        e.chk(a.rho, rho[i], "rho");
        e.chk(a.t, t[i], "T");
        e.chk(a.mmol, mm[i], "Mmol");
        e.chk(a.n_o, no[i], "nO");
    }
    e.report();
}

#[test]
fn spaceweather_manual_and_table() {
    let mut e = Err::new("atmos.spaceweather manual");
    for c in data()["sw_manual"].as_array().unwrap() {
        let sw = spaceweather::from_manual(&manual(&c["manual"])).unwrap();
        e.chk(sw.f107, f(&c["F107"]), "F107");
        e.chk(sw.f107a, f(&c["F107a"]), "F107a");
        e.chk(sw.kp_scalar(), f(&c["Kp"]), "Kp");
        e.chk(sw.ap, f(&c["ap"]), "ap");
        e.chk(sw.ap3, f(&c["ap3"]), "ap3");
        e.chk(sw.f107_today, f(&c["F107_today"]), "F107_today");
        for (g, w) in sw.aph.iter().zip(vecn(&c["aph"])) {
            e.chk(*g, w, "aph");
        }
    }
    e.report();
    // the SILS setting: DTM2020 gets F10.7 = F10.7a = 130 and akp = [2 0 2 0]
    let s = spaceweather::from_manual(&ManualIndices::SILS).unwrap();
    assert_eq!((s.f107, s.f107a, s.kp, s.ap), (130.0, 130.0, KpIn::Scalar(2.0), 7.0));
    assert!(spaceweather::from_manual(&ManualIndices { f107: 100.0, f107a: None, kp: None, ap: None, ap3: None }).is_err());

    let d = &data()["sw_table_def"];
    let apf = vecn(&d["ap"]);
    let t = SwTable {
        mjd: vecn(&d["mjd"]),
        f107obs: vecn(&d["f107obs"]),
        f107c81: vecn(&d["f107c81"]),
        kp: vecn(&d["kp"]),
        ap_daily: vecn(&d["apDaily"]),
        ap: apf.chunks(8).map(|c| [c[0], c[1], c[2], c[3], c[4], c[5], c[6], c[7]]).collect(),
        source: "synthetic".into(),
    };
    let mut e = Err::new("atmos.spaceweather table");
    for c in data()["sw_table"].as_array().unwrap() {
        let o = SwOpts { lag_f107: c["lag"].as_bool().unwrap(), aph_history: c["history"].as_bool().unwrap() };
        let sw = spaceweather::from_table(&t, &utc6(&c["utc"]), o).unwrap();
        e.chk(sw.f107, f(&c["F107"]), "F107");
        e.chk(sw.f107a, f(&c["F107a"]), "F107a");
        e.chk(sw.kp_scalar(), f(&c["Kp"]), "Kp");
        e.chk(sw.ap, f(&c["ap"]), "ap");
        e.chk(sw.ap3, f(&c["ap3"]), "ap3");
        e.chk(sw.f107_today, f(&c["F107_today"]), "F107_today");
        for (g, w) in sw.aph.iter().zip(vecn(&c["aph"])) {
            e.chk(*g, w, "aph");
        }
    }
    e.report();
}

#[test]
fn gsi_coefficients() {
    let g = &data()["gsi"];
    let k = |n: &str| vecn(&g[n]);
    let (s, dl, at, tw, ta, sn, st, no) = (k("s"), k("delta"), k("aT"), k("Tw"), k("Talt"), k("sig_n"), k("sig_t"), k("nO"));
    let mut e = Err::new("sentman/cll/dria/sesam");
    for i in 0..s.len() {
        let (cp, ct) = gsi::sentman(s[i], dl[i], at[i], tw[i], ta[i]);
        e.chk(cp, k("sentman_cp")[i], "sentman cp");
        e.chk(ct, k("sentman_ct")[i], "sentman ct");
        let (cp, ct) = gsi::cll(s[i], dl[i], sn[i], st[i], tw[i], ta[i]);
        e.chk(cp, k("cll_cp")[i], "cll cp");
        e.chk(ct, k("cll_ct")[i], "cll ct");
        let (cp, ct) = gsi::dria(s[i], dl[i], no[i], ta[i], tw[i]);
        e.chk(cp, k("dria_cp")[i], "dria cp");
        e.chk(ct, k("dria_ct")[i], "dria ct");
        e.chk(gsi::sesam(no[i], ta[i]), k("sesam")[i], "sesam");
        e.chk(gsi::speed_ratio(7600.0 * s[i] / 5.0, ta[i], 15.9994), k("speed_ratio")[i], "speedRatio");
    }
    e.report();
}

fn omega(v: &Value) -> V3 {
    let a = vecn(v);
    if a.len() == 1 { [0.0, 0.0, a[0]] } else { [a[0], a[1], a[2]] }
}

#[test]
fn cannonball_and_attitude() {
    let mut e = Err::new("drag.cannonball");
    for c in data()["cannonball"].as_array().unwrap() {
        let o = drag::cannonball(&v3(&c["r"]), &v3(&c["v"]), f(&c["rho"]), f(&c["Cd"]), f(&c["A"]), f(&c["mass"]), &v3(&c["wind"]), &omega(&c["omega"]));
        e.chk3(&o.a, &vecn(&c["a"]), "a");
        e.chk3(&o.f, &vecn(&c["F"]), "F");
        e.chk(o.drag, f(&c["drag"]), "drag");
        e.chk(o.vrel, f(&c["Vrel"]), "Vrel");
    }
    e.report();
    let mut e = Err::new("dgeom/sgeom attitude");
    let flat = |m: &M3| m.iter().flatten().copied().collect::<Vec<f64>>();
    for c in data()["attitude"].as_array().unwrap() {
        let vr = v3(&c["vrel"]);
        e.chk3(&flat(&geom::ram_attitude(&vr)), &flat(&m3(&c["ram"])), "ram");
        e.chk3(&flat(&geom::attitude_from_aoa(&vr, f(&c["aoa"]), f(&c["ss"]))), &flat(&m3(&c["aoa_R"])), "aoa");
        e.chk3(&flat(&geom::attitude_from_aoa(&vr, f(&c["aoa"]), 0.0)), &flat(&m3(&c["aoa_R0"])), "aoa0");
        e.chk3(&flat(&geom::r_lvlh(&v3(&c["r"]), &v3(&c["v"]))), &flat(&m3(&c["lvlh"])), "lvlh");
        e.chk3(&geom::array_normal(&v3(&c["axis"]), &v3(&c["sun_b"])), &vecn(&c["array_n"]), "arrayNormal");
    }
    e.report();
}

fn geometry(name: &str) -> Vec<Facet> {
    match name {
        "sat16u" => geom::sat16u().1,
        "box_array" => {
            let mut b = geom::build_box(0.34, 0.20, 0.20);
            geom::add_array(&mut b, [0.0, 0.0, 1.0], 0.12);
            b
        }
        "box_0.3_0.1_0.1" => geom::build_box(0.3, 0.1, 0.1),
        "plate" => vec![Facet::plain([1.0, 0.0, 0.0], 0.04)],
        _ => panic!("geometry {name}"),
    }
}

fn panel_model(s: &str) -> PanelModel {
    match DragModel::from_name(s).unwrap() {
        DragModel::Panel(p) => p,
        _ => panic!(),
    }
}

#[test]
fn panel_force_models() {
    let mut e = Err::new("drag.force (panel)");
    for c in data()["panel"].as_array().unwrap() {
        let a = &c["atm"];
        let atm = if a["species"].as_bool().unwrap() {
            let n = vecn(&a["n"]);
            AtmosOut { rho: f64::NAN, t: f(&a["T"]), comp: Composition::Species([n[0], n[1], n[2], n[3], n[4], n[5]]) }
        } else {
            AtmosOut { rho: f(&a["rho"]), t: f(&a["T"]), comp: Composition::Mean { mmol: f(&a["Mmol"]), n_o: f(&a["nO"]) } }
        };
        let facets = geometry(c["geom"].as_str().unwrap());
        let gsi = Gsi { tw: f(&c["Tw"]), a_t: f(&c["aT"]), sig_n: f(&c["sig_n"]), sig_t: f(&c["sig_t"]) };
        let opts = PanelOpts { mass: f(&c["mass"]), aref: f(&c["Aref"]), r_bi: m3(&c["R_bi"]), wind: [0.0; 3], omega: omega(&c["omega"]), sun_eci: Some(v3(&c["sun"])) };
        let o = drag::panel_force(&v3(&c["r"]), &v3(&c["v"]), &atm, &facets, panel_model(c["model"].as_str().unwrap()), &gsi, &opts).unwrap();
        e.chk3(&o.a, &vecn(&c["a"]), "a");
        e.chk3(&o.f, &vecn(&c["F"]), "F");
        e.chk3(&o.fbody, &vecn(&c["Fbody"]), "Fbody");
        let fm = vecn(&c["F"]).iter().fold(0.0f64, |m, x| m.max(x.abs()));
        e.chk3(&[o.drag, o.lift, o.side], &[f(&c["drag"]), f(&c["lift"]), f(&c["side"])].map(|x| x), "D/L/S");
        let _ = fm;
        e.chk(o.cd, f(&c["Cd"]), "Cd");
        if o.a_proj > 0.0 {
            e.chk(o.cd_a, f(&c["Cd_A"]), "Cd_A");
        }
        e.chk(o.a_proj, f(&c["A_proj"]), "A_proj");
        e.chk(o.alpha, f(&c["alpha"]), "alpha");
        e.chk(o.beta, f(&c["beta"]), "beta");
        e.chk(o.qd, f(&c["qd"]), "qd");
        e.chk(o.s_o, f(&c["s_O"]), "s_O");
    }
    e.report();
}

#[test]
fn forces_drag_chain() {
    let mut e = Err::new("forces.drag (end to end)");
    let mut e_sils = Err::new("  of which SILS path");
    for c in data()["chain"].as_array().unwrap() {
        let geom_name = c["geom"].as_str().unwrap();
        let facets = if geom_name == "none" { None } else { Some(geometry(geom_name)) };
        let g = c["gsi"].as_array().map(|a| Gsi { tw: f(&a[0]), a_t: f(&a[1]), sig_n: f(&a[2]), sig_t: f(&a[3]) }).unwrap_or_default();
        let cfg = DragConfig {
            model: DragModel::from_name(c["model"].as_str().unwrap()).unwrap(),
            atmos: AtmosModel::from_name(c["atmos"].as_str().unwrap()).unwrap(),
            cd: c["cfg_Cd"].as_f64(),
            corotate: c["corotate"].as_bool().unwrap(),
            gsi: g,
        };
        let man = manual(&c["manual"]);
        let inp = DragInput {
            r_eci: v3(&c["r"]),
            v_eci: v3(&c["v"]),
            lat_rad: f(&c["lat"]),
            lon_rad: f(&c["lon"]),
            alt_m: f(&c["alt"]),
            utc: utc6(&c["utc"]),
            doy: f(&c["doy"]),
            omega_eci: if c["omega"].is_null() { None } else { Some(v3(&c["omega"])) },
            sun_eci: Some(v3(&c["sun"])),
            sc: Spacecraft { mass: f(&c["mass"]), aref: f(&c["Aref"]), cd: Some(f(&c["sc_Cd"])), r_bi: m3(&c["R_bi"]), facets: facets.as_deref() },
            cfg: &cfg,
            sw: SwSources { manual: Some(&man), ..Default::default() },
        };
        let (a, info) = drag::accel(&inp).unwrap();
        e.chk3(&a, &vecn(&c["a"]), &format!("a kind {}", c["kind"]));
        e.chk(info.rho, f(&c["rho"]), "rho");
        e.chk(info.atm.t, f(&c["T"]), "T");
        e.chk(info.geo.lst_h, f(&c["lst"]), "lst");
        e.chk3(&info.v_rel, &vecn(&c["v_rel"]), "v_rel");
        if f(&c["kind"]) == 0.0 {
            e_sils.chk3(&a, &vecn(&c["a"]), "a");
            e_sils.chk(info.rho, f(&c["rho"]), "rho");
        }
    }
    e.report();
    e_sils.report();
    // cost of the SILS path (cannonball + DTM2020 + manual indices), allocation-free
    let c = &data()["chain"][0];
    let man = ManualIndices::SILS;
    let cfg = DragConfig::sils();
    let mut inp = DragInput {
        r_eci: v3(&c["r"]),
        v_eci: v3(&c["v"]),
        lat_rad: f(&c["lat"]),
        lon_rad: f(&c["lon"]),
        alt_m: f(&c["alt"]),
        utc: utc6(&c["utc"]),
        doy: f(&c["doy"]),
        omega_eci: Some(v3(&c["omega"])),
        sun_eci: None,
        sc: Spacecraft { mass: f(&c["mass"]), aref: f(&c["Aref"]), cd: Some(f(&c["sc_Cd"])), r_bi: m3(&c["R_bi"]), facets: None },
        cfg: &cfg,
        sw: SwSources { manual: Some(&man), ..Default::default() },
    };
    let (a0, _) = drag::accel(&inp).unwrap();
    assert_eq!(a0.map(f64::to_bits), v3(&c["a"]).map(f64::to_bits));
    let n = 20000;
    let t0 = std::time::Instant::now();
    let mut acc = 0.0;
    for k in 0..n {
        inp.alt_m = 3.0e5 + k as f64;
        acc += drag::accel(&inp).unwrap().1.rho;
    }
    println!("SILS drag (DTM2020 + cannonball): {:.2} us per call ({acc:.3e})", t0.elapsed().as_secs_f64() * 1e6 / n as f64);
}

#[test]
fn jb_parse_and_sources() {
    let idx = JbIndices::bundled();
    // 1997-001 .. 2026-137 (release 8_1_0)
    assert_eq!(idx.sol_t.len(), 10729);
    assert_eq!(idx.dtc_t.len(), 24 * 10729);
    assert!(idx.sol_t.windows(2).all(|w| w[1] > w[0]) && idx.dtc_t.windows(2).all(|w| w[1] > w[0]));
    // 1997-001: F10 72.4 F81 78.0 ... (first data line of SOLFSMY.TXT)
    assert_eq!(idx.sol[0], [72.4, 78.0, 74.0, 79.2, 65.4, 73.8, 61.9, 70.7]);
    // GFZ JSON + OMNI2 + merge
    let js = r#"{"datetime":["2020-01-01T00:00:00Z","2020-01-01T03:00:00Z","2020-01-02T00:00:00Z"],"Kp":[1.333,2.0,-1]}"#;
    let kp = spaceweather::parse_gfz_json(js, "Kp").unwrap();
    assert!(kp.v[2].is_nan());
    let ap = spaceweather::parse_gfz_json(r#"{"datetime":["2020-01-01T00:00:00Z","2020-01-01T03:00:00Z","2020-01-02T00:00:00Z"],"ap":[5,7,9]}"#, "ap").unwrap();
    let t = spaceweather::merge_to_table(&[], &[kp, ap]);
    assert_eq!(t.mjd, vec![58849.0, 58850.0]);
    assert!((t.kp[0] - (1.333 + 2.0) / 2.0).abs() < 1e-15);
    assert_eq!(t.ap_daily[0], 6.0);
    assert_eq!(spaceweather::drivers_for("dtm2020_res"), Some(spaceweather::DriverSet::F30Ap60));
    let d = spaceweather::parse_45day("  12 Aug 2026   150   12\nfoo\n");
    assert_eq!(d.len(), 1);
    assert_eq!((d[0].1, d[0].2), (150.0, 12.0));
}

/// force_data/16U_dria_dtm2020_srp-boxwing_erp-boxwing (TEMPLATE_16U: DRIA panels on the
/// sgeom.sat16u box, ram attitude, DTM2020 with manual F10.7 = 90, Kp = 2, ap = 8).
/// (1) the port against the CURRENT MATLAB chain on the exact per-row inputs (1e-12);
/// (2) the port against the golden CSVs, which were written by an older forces.drag
///     whose panel branch co-rotated the atmosphere with the scalar z-axis rate
///     [0,0,omega_earth] (the bug its "OMEGA." comment describes) while op.accel's ram
///     attitude already used W.omega_eci -- reproduced with exactly that mix.
#[test]
fn golden_16u_dria_dtm2020() {
    let p = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/data/atmos_drag_golden.json");
    let g = json::parse(&std::fs::read_to_string(p).expect("read atmos_drag_golden.json"));
    let man = manual(&g["manual"]);
    let (_, facets) = geom::sat16u();
    let cfg = DragConfig { model: DragModel::Panel(PanelModel::Dria), atmos: AtmosModel::Dtm2020, cd: None, corotate: true, gsi: Gsi::default() };
    let mut e = Err::new("16U golden: port vs MATLAB");
    let mut worst = [0.0f64; 7]; // a, rho, T, nO, Cd, A_proj, |a| vs golden CSV
    for row in g["rows"].as_array().unwrap() {
        let mk = |omega: V3, r_bi: M3| DragInput {
            r_eci: v3(&row["r"]),
            v_eci: v3(&row["v"]),
            lat_rad: f(&row["lat"]),
            lon_rad: f(&row["lon"]),
            alt_m: f(&row["alt"]),
            utc: utc6(&row["utc"]),
            doy: f(&row["doy"]),
            omega_eci: Some(omega),
            sun_eci: Some(v3(&row["sun"])),
            sc: Spacecraft { mass: f(&g["mass"]), aref: f(&g["Aref"]), cd: Some(2.2), r_bi, facets: Some(&facets) },
            cfg: &cfg,
            sw: SwSources { manual: Some(&man), ..Default::default() },
        };
        // (1) current MATLAB
        let (a, info) = drag::accel(&mk(v3(&row["omega"]), m3(&row["R_bi"]))).unwrap();
        e.chk3(&a, &vecn(&row["a"]), "a");
        e.chk(info.rho, f(&row["rho"]), "rho");
        e.chk(info.atm.t, f(&row["T"]), "T");
        e.chk(info.atm.n_o(), f(&row["nO"]), "nO");
        let drag::DragOut::Panel(o) = info.out else { panic!() };
        e.chk(o.cd, f(&row["Cd"]), "Cd");
        e.chk(o.a_proj, f(&row["A_proj"]), "A_proj");
        // (2) golden CSV: z-axis rate in the co-rotation, ram attitude from the true rate.
        // (A ram attitude built from the z-axis rate too agrees only to ~3e-2: the +-y bus
        // faces lie exactly along the flow, and whether their shear counts hangs on the
        // sign of a ~1e-17 dot product -- a knife edge of the MATLAB model, reproduced
        // bit for bit in (1).)
        let omz = [0.0, 0.0, drag::omega_earth()];
        let (a, info) = drag::accel(&mk(omz, m3(&row["R_bi"]))).unwrap();
        let gd = &row["golden"];
        let ga = vecn(&gd["a"]);
        let gn = ga.iter().map(|x| x * x).sum::<f64>().sqrt();
        let da = (0..3).map(|i| (a[i] - ga[i]).abs()).fold(0.0, f64::max) / gn;
        let drag::DragOut::Panel(o) = info.out else { panic!() };
        let rel = |x: f64, y: f64| ((x - y) / y).abs();
        let an = adcs_pop::la::norm(&a);
        let d = [da, rel(info.rho, f(&gd["rho"])), rel(info.atm.t, f(&gd["T"])), rel(info.atm.n_o(), f(&gd["nO"])), rel(o.cd, f(&gd["Cd"])), rel(o.a_proj, f(&gd["A_proj"])), rel(an, gn)];
        for k in 0..7 {
            worst[k] = worst[k].max(d[k]);
        }
    }
    e.report();
    println!(
        "16U golden CSV (10 digits) vs port, max rel diff: a {:.2e} |a| {:.2e} rho {:.2e} T {:.2e} nO {:.2e} Cd {:.2e} A_proj {:.2e}",
        worst[0], worst[6], worst[1], worst[2], worst[3], worst[4], worst[5]
    );
    // the CSV state and values are rounded to 10 digits: agreement ~1e-8
    for (k, lim) in [(0, 1e-7), (1, 1e-7), (2, 1e-7), (3, 1e-7), (4, 1e-7), (5, 1e-7), (6, 1e-7)] {
        assert!(worst[k] < lim, "golden atmosphere/geometry mismatch {k}: {:e}", worst[k]);
    }
}

