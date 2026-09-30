//! Onboard time, frames and environment models (fsw/pseudocode/02), twin of adcs_env.c.
use crate::igrf13;
use crate::m::*;
use crate::math::*;

/// ECI -> ECEF rotation about z by GMST.
pub fn gmst_rot(jd: f64) -> M3 {
    let t = (jd - 2451545.0)/36525.0;
    let mut g = fmod(67310.54841 + (876600.0*3600.0 + 8640184.812866)*t + 0.093104*t*t - 6.2e-6*t*t*t, 86400.0);
    if g < 0.0 { g += 86400.0; }
    g = g/240.0*D2R;
    [[cos(g), sin(g), 0.0], [-sin(g), cos(g), 0.0], [0.0, 0.0, 1.0]]
}
/// J2000 -> mean of date (IAU-76 precession): P = R3(-z) R2(theta) R3(-zeta).
pub fn prec_rot(jd: f64) -> M3 {
    let t = (jd - 2451545.0)/36525.0;
    let as_ = PI/(180.0*3600.0);
    let ze = (2306.2181*t + 0.30188*t*t + 0.017998*t*t*t)*as_;
    let z = (2306.2181*t + 1.09468*t*t + 0.018203*t*t*t)*as_;
    let th = (2004.3109*t - 0.42665*t*t - 0.041833*t*t*t)*as_;
    let (cz, sz, cc, sc, ct, st) = (cos(ze), sin(ze), cos(z), sin(z), cos(th), sin(th));
    [[cc*ct*cz - sc*sz, -cc*ct*sz - sc*cz, -cc*st],
     [sc*ct*cz + cc*sz, -sc*ct*sz + cc*cz, -sc*st],
     [st*cz, -st*sz, ct]]
}
/// J2000 -> ECEF: precession to the mean equator of date, then GMST (nutation and polar motion omitted).
pub fn eci2ecef(jd: f64) -> M3 { mat3_mul(&gmst_rot(jd), &prec_rot(jd)) }
pub fn decyear(jd: f64) -> f64 { 2000.0 + (jd - 2451544.5)/365.25 }
fn mod360(x: f64) -> f64 { let x = fmod(x, 360.0); if x < 0.0 { x + 360.0 } else { x } }

/// Unit Sun direction, J2000 (mean-of-date model precessed to J2000).
pub fn sun_model(jd: f64) -> V3 {
    let t = (jd - 2451545.0)/36525.0;
    let l = mod360(280.460 + 36000.771*t);
    let m = mod360(357.5291092 + 35999.05034*t)*D2R;
    let lam = (l + 1.914666471*sin(m) + 0.019994643*sin(2.0*m))*D2R;
    let eps = (23.439291 - 0.0130042*t)*D2R;
    let as_ = PI/(180.0*3600.0);
    let zeta = (2306.2181*t + 0.30188*t*t)*as_;
    let z = (2306.2181*t + 1.09468*t*t)*as_;
    let th = (2004.3109*t - 0.42665*t*t)*as_;
    let sm = [cos(lam), cos(eps)*sin(lam), sin(eps)*sin(lam)];
    let a = [cos(z)*sm[0] + sin(z)*sm[1], -sin(z)*sm[0] + cos(z)*sm[1], sm[2]];
    let b = [cos(-th)*a[0] - sin(-th)*a[2], a[1], sin(-th)*a[0] + cos(-th)*a[2]];
    let s = [cos(zeta)*b[0] + sin(zeta)*b[1], -sin(zeta)*b[0] + cos(zeta)*b[1], b[2]];
    unit(&s)
}

/// WGS-84 geodetic latitude, longitude [rad] and height [m] of an ECEF position.
pub fn geodetic(r: &V3) -> (f64, f64, f64) {
    let a = 6378137.0;
    let f = 1.0/298.257223563;
    let e2 = f*(2.0 - f);
    let p = sqrt(r[0]*r[0] + r[1]*r[1]);
    let lon = atan2(r[1], r[0]);
    let mut la = atan2(r[2], p*(1.0 - e2));
    let mut hh = 0.0;
    for _ in 0..3 {
        let sl = sin(la);
        let n = a/sqrt(1.0 - e2*sl*sl);
        hh = p/cos(la) - n;
        la = atan2(r[2], p*(1.0 - e2*n/(n + hh)));
    }
    (la, lon, hh)
}

/// Gauss coefficients interpolated to a decimal year [nT] (linear, last interval extrapolated).
pub fn igrf_gh(dy: f64) -> [f64; 195] {
    let mut i = 0;
    while i + 1 < igrf13::EPOCHS && igrf13::YEAR[i + 1] <= dy { i += 1; }
    if i >= igrf13::EPOCHS - 1 { i = igrf13::EPOCHS - 2; }
    let f = (dy - igrf13::YEAR[i])/(igrf13::YEAR[i + 1] - igrf13::YEAR[i]);
    let mut gh = [0.0; 195];
    for k in 0..igrf13::NCOEF { gh[k] = igrf13::GH[i][k] + (igrf13::GH[i + 1][k] - igrf13::GH[i][k])*f; }
    gh
}

/// Field in north-east-down [nT] at geodetic lat, lon [rad], altitude [km].
pub fn igrf_ned(gh: &[f64; 195], lat: f64, lon: f64, alt_km: f64, nmax: i32) -> V3 {
    let re = 6371.2;
    let a = 6378.137;
    let f = 1.0/298.257223563;
    let b = a*(1.0 - f);
    let nmax = if nmax > 13 { 13 } else { nmax } as usize;
    let mut ct = cos(PI/2.0 - lat);
    let mut st = sin(PI/2.0 - lat);
    let rho = sqrt((a*st)*(a*st) + (b*ct)*(b*ct));
    let r = sqrt(alt_km*alt_km + 2.0*alt_km*rho + (a*a*a*a*st*st + b*b*b*b*ct*ct)/(rho*rho));
    let cd = (alt_km + rho)/r;
    let sd = (a*a - b*b)/rho*ct*st/r;
    let oc = ct;
    let mut p = [0.0f64; 120];
    let mut dp = [0.0f64; 120];
    let mut cphi = [0.0f64; 14];
    let mut sphi = [0.0f64; 14];
    let (mut br, mut bt, mut bp) = (0.0, 0.0, 0.0);
    ct = ct*cd - st*sd;
    st = st*cd + oc*sd;
    for k in 1..=nmax { cphi[k] = cos(k as f64*lon); sphi[k] = sin(k as f64*lon); }
    let pmax = (nmax + 1)*(nmax + 2)/2;
    p[0] = 1.0; p[2] = st; dp[2] = ct;
    let mut a_r = (re/r)*(re/r);
    let (mut m, mut n, mut ci) = (1usize, 0usize, 0usize);
    for pi in 2..=pmax {
        let ix = pi - 1;
        if n < m { m = 0; n += 1; a_r *= re/r; }
        let (nf, mf) = (n as f64, m as f64);
        if m < n && pi != 3 {
            let l1 = pi - n - 1;
            let l2 = pi - 2*n;
            let k1 = (2.0*nf - 1.0)/sqrt(nf*nf - mf*mf);
            let k2 = sqrt(((nf - 1.0)*(nf - 1.0) - mf*mf)/(nf*nf - mf*mf));
            p[ix] = k1*ct*p[l1] - k2*p[l2];
            dp[ix] = k1*(ct*dp[l1] - st*p[l1]) - k2*dp[l2];
        } else if pi != 3 {
            let ln = pi - n - 2;
            let kk = sqrt(1.0 - 1.0/(2.0*mf));
            p[ix] = kk*st*p[ln];
            dp[ix] = kk*(st*dp[ln] + ct*p[ln]);
        }
        if m == 0 {
            let c = a_r*gh[ci];
            br += (nf + 1.0)*c*p[ix];
            bt -= c*dp[ix];
            ci += 1;
        } else {
            let gc = gh[ci]*cphi[m] + gh[ci + 1]*sphi[m];
            let gs = -gh[ci]*sphi[m] + gh[ci + 1]*cphi[m];
            let c = a_r*gc;
            br += (nf + 1.0)*c*p[ix];
            bt -= c*dp[ix];
            if st == 0.0 { bp -= ct*a_r*gs*dp[ix]; } else { bp -= 1.0/st*a_r*mf*gs*p[ix]; }
            ci += 2;
        }
        m += 1;
    }
    let (bx, by, bz) = (-bt, bp, -br);
    [bx*cd + bz*sd, by, bz*cd - bx*sd]
}

/// Field in ECI [T] at r_eci [m].
pub fn field_eci(r_eci: &V3, jd: f64, gh: &[f64; 195], nmax: i32) -> V3 {
    let c = eci2ecef(jd);
    let re = mat3_vec(&c, r_eci);
    let (lat, lon, h) = geodetic(&re);
    let mut bn = igrf_ned(gh, lat, lon, h/1000.0, nmax);
    for x in bn.iter_mut() { *x *= 1e-9; }
    let (sl, cl, so, co) = (sin(lat), cos(lat), sin(lon), cos(lon));
    let be = [-sl*co*bn[0] - so*bn[1] - cl*co*bn[2],
              -sl*so*bn[0] + co*bn[1] - cl*so*bn[2],
              cl*bn[0] - sl*bn[2]];
    mat3t_vec(&c, &be)
}
