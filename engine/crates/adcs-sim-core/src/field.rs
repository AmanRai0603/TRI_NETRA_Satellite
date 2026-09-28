//! Truth geomagnetic field: IGRF-13 to degree 13 (asils.env.igrf_ned / field).
//! The coefficient table is the generated fsw-rs/src/igrf13.rs (data, not code:
//! the same numbers the MATLAB twin exports to matlab_sils/data/igrf13.json).
use crate::la::*;
use crate::pm::*;

#[allow(clippy::all)]
#[path = "../../../../fsw-rs/src/igrf13.rs"]
mod tab;
pub use tab::{EPOCHS, NCOEF};

pub type Gh = [f64; 195];

/// Gauss coefficients at a decimal year [nT] (linear between epochs, last interval extrapolated).
pub fn gh(dy: f64) -> Gh {
    let mut i = 0;
    while i + 1 < EPOCHS && tab::YEAR[i + 1] <= dy { i += 1; }
    if i >= EPOCHS - 1 { i = EPOCHS - 2; }
    let f = (dy - tab::YEAR[i])/(tab::YEAR[i + 1] - tab::YEAR[i]);
    let mut g = [0.0; 195];
    for k in 0..NCOEF { g[k] = tab::GH[i][k] + (tab::GH[i + 1][k] - tab::GH[i][k])*f; }
    g
}

/// WGS-84 geodetic latitude, longitude [rad], height [m].
pub fn geodetic(r: &V3) -> (f64, f64, f64) {
    let a = 6378137.0;
    let f = 1.0/298.257223563;
    let e2 = f*(2.0 - f);
    let p = sqrt(r[0]*r[0] + r[1]*r[1]);
    let lon = atan2(r[1], r[0]);
    let mut la = atan2(r[2], p*(1.0 - e2));
    let mut h = 0.0;
    for _ in 0..5 {
        let sl = sin(la);
        let n = a/sqrt(1.0 - e2*sl*sl);
        h = p/cos(la) - n;
        la = atan2(r[2], p*(1.0 - e2*n/(n + h)));
    }
    (la, lon, h)
}

/// Field north-east-down [nT] (Schmidt semi-normalised recursion, geodetic input).
pub fn ned(g: &Gh, lat: f64, lon: f64, alt_km: f64, nmax: usize) -> V3 {
    let re = 6371.2;
    let a = 6378.137;
    let f = 1.0/298.257223563;
    let b = a*(1.0 - f);
    let nmax = if nmax > 13 { 13 } else { nmax };
    let mut ct = cos(PI/2.0 - lat);
    let mut st = sin(PI/2.0 - lat);
    let rho = sqrt((a*st)*(a*st) + (b*ct)*(b*ct));
    let r = sqrt(alt_km*alt_km + 2.0*alt_km*rho + (a*a*a*a*st*st + b*b*b*b*ct*ct)/(rho*rho));
    let cd = (alt_km + rho)/r;
    let sd = (a*a - b*b)/rho*ct*st/r;
    let oc = ct;
    ct = ct*cd - st*sd;
    st = st*cd + oc*sd;
    let mut p = [0.0f64; 120];
    let mut dp = [0.0f64; 120];
    let mut cphi = [0.0f64; 14];
    let mut sphi = [0.0f64; 14];
    for k in 1..=nmax { cphi[k] = cos(k as f64*lon); sphi[k] = sin(k as f64*lon); }
    let pmax = (nmax + 1)*(nmax + 2)/2;
    p[0] = 1.0; p[2] = st; dp[2] = ct;
    let (mut br, mut bt, mut bp) = (0.0, 0.0, 0.0);
    let mut a_r = (re/r)*(re/r);
    let (mut m, mut n, mut ci) = (1usize, 0usize, 0usize);
    for pi in 2..=pmax {
        let ix = pi - 1;
        if n < m { m = 0; n += 1; a_r *= re/r; }
        let (nf, mf) = (n as f64, m as f64);
        if m < n && pi != 3 {
            let (l1, l2) = (pi - n - 1, pi - 2*n);
            let k1 = (2.0*nf - 1.0)/sqrt(nf*nf - mf*mf);
            let k2 = sqrt(((nf - 1.0)*(nf - 1.0) - mf*mf)/(nf*nf - mf*mf));
            p[ix] = k1*ct*p[l1] - k2*p[l2];
            dp[ix] = k1*(ct*dp[l1] - st*p[l1]) - k2*dp[l2];
        } else if pi != 3 {
            let ln_ = pi - n - 2;
            let kk = sqrt(1.0 - 1.0/(2.0*mf));
            p[ix] = kk*st*p[ln_];
            dp[ix] = kk*(st*dp[ln_] + ct*p[ln_]);
        }
        if m == 0 {
            let c = a_r*g[ci];
            br += (nf + 1.0)*c*p[ix]; bt -= c*dp[ix]; ci += 1;
        } else {
            let gc = g[ci]*cphi[m] + g[ci + 1]*sphi[m];
            let gs = -g[ci]*sphi[m] + g[ci + 1]*cphi[m];
            let c = a_r*gc;
            br += (nf + 1.0)*c*p[ix]; bt -= c*dp[ix];
            if st == 0.0 { bp -= ct*a_r*gs*dp[ix]; } else { bp -= 1.0/st*a_r*mf*gs*p[ix]; }
            ci += 2;
        }
        m += 1;
    }
    let (bx, by, bz) = (-bt, bp, -br);
    [bx*cd + bz*sd, by, bz*cd - bx*sd]
}

/// Field in ECI [T] at an ECEF position, given the ECI->ECEF matrix.
pub fn eci(r_ecef: &V3, c_eci2ecef: &M3, g: &Gh, nmax: usize) -> V3 {
    let (lat, lon, h) = geodetic(r_ecef);
    let bn = scale(&ned(g, lat, lon, h/1000.0, nmax), 1e-9);
    let (sl, cl, so, co) = (sin(lat), cos(lat), sin(lon), cos(lon));
    let be = [-sl*co*bn[0] - so*bn[1] - cl*co*bn[2], -sl*so*bn[0] + co*bn[1] - cl*so*bn[2], cl*bn[0] - sl*bn[2]];
    mtv(c_eci2ecef, &be)
}
