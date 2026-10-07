//! Gravity-field containers and loaders: `grav.defaultField`, `grav.loadGFC`,
//! `op.gravLoad` and the local-file part of `data.gravity`. The default field and the zonals are the design's
//! (`gen::gravity`); the ICGEM `.gfc` reader is code.
//!
//! Owner: Agastya. Copyright (c) 2026 Agastya. All rights reserved.
use std::path::{Path, PathBuf};

/// A spherical-harmonic gravity field, as the MATLAB `fld` struct.
///
/// Port of the struct returned by `grav.defaultField` / `grav.loadGFC` and completed
/// by `op.gravLoad` (which adds `.J`). Coefficients are fully (4 pi) normalised and
/// stored row-major: `cbar[n * (nmax + 1) + m]` is MATLAB `Cbar(n+1, m+1)`.
#[derive(Clone, Debug)]
pub struct Field {
    /// `fld.mu`, GM of the field [m^3/s^2].
    pub gm: f64,
    /// `fld.Re`, reference radius [m].
    pub re: f64,
    /// `fld.nmax`, maximum degree held (the matrices are (nmax+1)x(nmax+1)).
    pub nmax: usize,
    /// `fld.Cbar`, normalised cosine coefficients, row-major (nmax+1)^2.
    pub cbar: Vec<f64>,
    /// `fld.Sbar`, normalised sine coefficients, row-major (nmax+1)^2.
    pub sbar: Vec<f64>,
    /// `fld.name`.
    pub name: String,
    /// `fld.J` from `op.gravLoad`: unnormalised zonals `[J2 .. J_min(nmax,6)]`,
    /// `J(n-1) = -Cbar(n+1,1) * sqrt(2n+1)`.
    pub j: Vec<f64>,
}

impl Field {
    /// Row stride of `cbar` / `sbar` (= nmax + 1).
    #[inline]
    pub fn stride(&self) -> usize {
        self.nmax + 1
    }
    /// MATLAB `Cbar(n+1, m+1)`.
    #[inline]
    pub fn c(&self, n: usize, m: usize) -> f64 {
        self.cbar[n * (self.nmax + 1) + m]
    }
    /// MATLAB `Sbar(n+1, m+1)`.
    #[inline]
    pub fn s(&self, n: usize, m: usize) -> f64 {
        self.sbar[n * (self.nmax + 1) + m]
    }

    /// Port of `grav.defaultField` (zonal J2..J6, EGM2008 GM and radius), with the
    /// `.J` vector attached exactly as `op.gravLoad` derives it from `Cbar`: the design's field
    /// (`gen::gravity::default_field` over env_gravity_default_field).
    pub fn default_field() -> Field {
        let (gm, re, c) = crate::gen::gravity::default_field();
        let nmax = 6;
        let mut f = Field { gm, re, nmax, cbar: c.to_vec(), sbar: vec![0.0; 49], name: "zonal J2-J6 (EGM-consistent)".into(), j: Vec::new() };
        f.expose_j();
        f
    }

    /// Port of `grav.loadGFC(fname, maxdeg)` (ICGEM `.gfc` parser; `maxdeg = None`
    /// is MATLAB's `Inf`), with `.J` attached as `op.gravLoad` does.
    ///
    /// Same semantics as the MATLAB: header keys `earth_gravity_constant`, `radius`,
    /// `max_degree`, `modelname` are read until `end_of_head`, which allocates
    /// `N = min(maxdeg, max_degree)` (360 when both are unbounded); `gfc` rows set
    /// `Cbar/Sbar`, `gfct` rows are skipped (the matrix still grows to their degree),
    /// rows with `n > maxdeg` are ignored, Fortran `D` exponents are accepted in the
    /// coefficients, and `nmax` is the size of the matrix, not the largest row seen.
    /// A file without `end_of_head` (MATLAB: empty field, nmax = -1) is an error here.
    pub fn load_gfc(path: impl AsRef<Path>, maxdeg: Option<usize>) -> Result<Field, crate::PopError> {
        let path = path.as_ref();
        let text = std::fs::read_to_string(path).map_err(|e| crate::PopError::Data(format!("grav:loadGFC cannot open \"{}\": {e}", path.display())))?;
        Self::parse_gfc(&text, maxdeg)
    }

    /// The parser of [`Field::load_gfc`] on in-memory text (`grav.loadGFC`).
    pub fn parse_gfc(text: &str, maxdeg: Option<usize>) -> Result<Field, crate::PopError> {
        let str2double = |s: &str| s.parse::<f64>().unwrap_or(f64::NAN);
        let str2num_safe = |s: &str| str2double(&s.replace(['D', 'd'], "E"));
        let (mut mu, mut re, mut name) = (f64::NAN, f64::NAN, String::from("(gfc)"));
        let mut nmax_hdr = f64::INFINITY;
        let mut in_header = true;
        // square matrices of size `sz`, row-major
        let mut sz = 0usize;
        let mut c: Vec<f64> = Vec::new();
        let mut s: Vec<f64> = Vec::new();
        let grow = |c: &mut Vec<f64>, s: &mut Vec<f64>, sz: &mut usize, new: usize| {
            if new <= *sz {
                return;
            }
            let mut c2 = vec![0.0; new * new];
            let mut s2 = vec![0.0; new * new];
            for i in 0..*sz {
                for j in 0..*sz {
                    c2[i * new + j] = c[i * *sz + j];
                    s2[i * new + j] = s[i * *sz + j];
                }
            }
            *c = c2;
            *s = s2;
            *sz = new;
        };
        let maxdeg_f = maxdeg.map(|d| d as f64).unwrap_or(f64::INFINITY);
        for ln in text.lines() {
            let t = ln.trim();
            if t.is_empty() {
                continue;
            }
            let tok: Vec<&str> = t.split_whitespace().collect();
            let key = tok[0].to_lowercase();
            if in_header {
                match key.as_str() {
                    "earth_gravity_constant" => mu = tok.get(1).map_or(f64::NAN, |v| str2double(v)),
                    "radius" => re = tok.get(1).map_or(f64::NAN, |v| str2double(v)),
                    "max_degree" => nmax_hdr = tok.get(1).map_or(f64::NAN, |v| str2double(v)),
                    "modelname" => {
                        if tok.len() >= 2 {
                            name = tok[1].to_string();
                        }
                    }
                    "end_of_head" => {
                        in_header = false;
                        let mut n = maxdeg_f.min(nmax_hdr);
                        if n.is_infinite() {
                            n = 360.0;
                        }
                        if n.is_nan() || n < 0.0 {
                            return Err(crate::PopError::Data(format!("grav:loadGFC bad max_degree {n}")));
                        }
                        sz = 0;
                        c.clear();
                        s.clear();
                        grow(&mut c, &mut s, &mut sz, n as usize + 1);
                    }
                    _ => {}
                }
                continue;
            }
            if key == "gfc" || key == "gfct" {
                if tok.len() < 5 {
                    return Err(crate::PopError::Data(format!("grav:loadGFC short coefficient line: {t}")));
                }
                let n = str2double(tok[1]);
                let m = str2double(tok[2]);
                if n > maxdeg_f {
                    continue;
                }
                if !(n >= 0.0 && m >= 0.0 && n.fract() == 0.0 && m.fract() == 0.0) {
                    return Err(crate::PopError::Data(format!("grav:loadGFC bad degree/order: {t}")));
                }
                let (n, m) = (n as usize, m as usize);
                if n + 1 > sz || m + 1 > sz {
                    let new = (n + 1).max(m + 1);
                    grow(&mut c, &mut s, &mut sz, new);
                }
                let cv = str2num_safe(tok[3]);
                let sv = str2num_safe(tok[4]);
                if key == "gfc" {
                    c[n * sz + m] = cv;
                    s[n * sz + m] = sv;
                }
            }
        }
        if sz == 0 {
            return Err(crate::PopError::Data("grav:loadGFC no end_of_head / no coefficients (MATLAB would return an empty field)".into()));
        }
        let mut f = Field { gm: mu, re, nmax: sz - 1, cbar: c, sbar: s, name, j: Vec::new() };
        f.expose_j();
        Ok(f)
    }

    /// The `.J` step of `op.gravLoad`: `nz = min(nmax,6)`, `J(n-1) = -Cbar(n+1,1)*sqrt(2n+1)` (the design's,
    /// `gen::gravity::zonal_j`).
    pub fn expose_j(&mut self) {
        let nz = self.nmax.min(6);
        self.j = (2..=nz).map(|n| crate::gen::gravity::zonal_j(self.c(n, 0), n as i64)).collect();
    }
}

/// `cfg.gravityField` as read by `op.gravLoad`.
#[derive(Clone, Debug)]
pub struct GravityFieldCfg {
    /// `.field`: `"default"`, a `.gfc` path, or a known ICGEM model name
    /// (`EGM2008`, `EIGEN-6C4`, `GOCO06s`) resolved to a local cached file.
    pub field: String,
    /// `.degree`: maximum degree kept from the file (MATLAB default 60, with a warning).
    pub degree: Option<usize>,
}

impl Default for GravityFieldCfg {
    /// `config.defaultConfig`: `struct('field','default','degree',6)`.
    fn default() -> Self {
        GravityFieldCfg { field: "default".into(), degree: Some(6) }
    }
}

/// The model-name registry of `data.gravity` (file names only: no downloads here).
pub fn registry_file(model: &str) -> Option<&'static str> {
    let m = model.replace(' ', "").to_uppercase();
    match m.as_str() {
        "EGM2008" => Some("EGM2008.gfc"),
        "EIGEN-6C4" => Some("EIGEN-6C4.gfc"),
        "GOCO06S" => Some("GOCO06s.gfc"),
        _ => None,
    }
}

/// Port of `op.gravLoad` with the offline part of `data.gravity`.
///
/// `"default"` (any case) -> [`Field::default_field`]; an existing file path ->
/// [`Field::load_gfc`]; otherwise the name is looked up in each of `search_dirs`
/// (MATLAB: `exist(src,'file')` also finds files on the path, e.g. `gravity_data/`),
/// and a known model name is resolved to its cached file name
/// (`<data.root>/gravity/EGM2008.gfc` ...) in the same directories. Nothing is
/// downloaded: a missing file is an error. When `degree` is `None` the MATLAB
/// default 60 is used and its warning is printed to stderr.
pub fn grav_load(cfg: &GravityFieldCfg, search_dirs: &[PathBuf]) -> Result<Field, crate::PopError> {
    let deg = match cfg.degree {
        Some(d) => d,
        None => {
            eprintln!("warning(op:gravLoad:degreeDefault): no cfg.gravityField.degree given -- defaulting to 60");
            60
        }
    };
    let src = cfg.field.as_str();
    if src.eq_ignore_ascii_case("default") {
        return Ok(Field::default_field());
    }
    let p = Path::new(src);
    if p.is_file() {
        return Field::load_gfc(p, Some(deg));
    }
    for d in search_dirs {
        let q = d.join(src);
        if q.is_file() {
            return Field::load_gfc(q, Some(deg));
        }
    }
    if let Some(fname) = registry_file(src) {
        for d in search_dirs {
            for q in [d.join(fname), d.join("gravity").join(fname)] {
                if q.is_file() {
                    return Field::load_gfc(q, Some(deg));
                }
            }
        }
        return Err(crate::PopError::Data(format!("data:gravity: {fname} not found locally (searched {search_dirs:?}); this port never downloads")));
    }
    Err(crate::PopError::Unsupported(format!("op:gravLoad: \"{src}\" is not 'default', an existing .gfc file, or a known model name")))
}
