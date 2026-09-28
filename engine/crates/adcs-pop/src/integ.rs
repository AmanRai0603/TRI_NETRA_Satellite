//! Integrators of POP (matlab_sils/pop/01_core/+integ): rk4, rk45 (Dormand-Prince 5(4)),
//! rk78 (Dormand-Prince 8(7) / Prince-Dormand 13-stage), rk6luther, nystrom4,
//! gaussJackson8 (8th-order summed Stormer-Cowell with Kahan sums), hermite dense output
//! and the run() dispatcher. The ode45/78/89/113 wrappers of integ.odesuite call MATLAB's
//! built-in solvers and have no counterpart here.
//! State y = [r; v] (6); f(t, y) -> dy/dt.

pub type Y = [f64; 6];

/// Nodes of an integration: times, states and derivatives (for Hermite output).
#[derive(Clone, Debug, Default)]
pub struct Sol { pub t: Vec<f64>, pub y: Vec<Y>, pub d: Vec<Y> }

#[derive(Clone, Copy, Debug)]
pub struct Opts { pub h: f64, pub rtol: f64, pub atol: f64, pub hmax: Option<f64>, pub hmin: f64, pub h0: Option<f64>, pub facmin: f64, pub facmax: f64, pub maxsteps: usize }
impl Default for Opts {
    fn default() -> Self { Opts { h: 30.0, rtol: 1e-9, atol: 1e-12, hmax: None, hmin: 1e-6, h0: None, facmin: 0.2, facmax: 5.0, maxsteps: 2_000_000 } }
}

#[derive(Debug, Clone, PartialEq)]
pub enum IntegError { NonFinite(f64), MaxSteps(f64), Method(String) }

#[inline] fn axpy(y: &Y, h: f64, k: &Y) -> Y { let mut o = *y; for i in 0..6 { o[i] += h*k[i]; } o }
fn finite(y: &Y) -> bool { y.iter().all(|x| x.is_finite()) }

/// integ.rk4 (lands exactly on tf).
pub fn rk4<F: FnMut(f64, &Y) -> Y>(mut f: F, t0: f64, tf: f64, y0: Y, h: f64) -> Sol {
    let n = (((tf - t0)/h).round() as usize).max(1);
    let h = (tf - t0)/n as f64;
    let mut s = Sol::default();
    let mut y = y0;
    s.t.push(t0); s.y.push(y); s.d.push(f(t0, &y));
    for k in 0..n {
        let t = t0 + k as f64*h;
        let k1 = f(t, &y);
        let k2 = f(t + h/2.0, &axpy(&y, h/2.0, &k1));
        let k3 = f(t + h/2.0, &axpy(&y, h/2.0, &k2));
        let k4 = f(t + h, &axpy(&y, h, &k3));
        for i in 0..6 { y[i] += h/6.0*(k1[i] + 2.0*k2[i] + 2.0*k3[i] + k4[i]); }
        let tn = t0 + (k + 1) as f64*h;
        s.t.push(tn); s.y.push(y); s.d.push(f(tn, &y));
    }
    s
}

fn adopt(o: &Opts, t0: f64, tf: f64) -> Opts {
    let span = (tf - t0).abs();
    let mut r = *o;
    let hmax = o.hmax.unwrap_or(span/2.0);
    r.hmax = Some(hmax);
    r.h0 = Some(o.h0.unwrap_or((span/100.0).min(10.0)).min(hmax));
    r
}

fn rms_err(y: &Y, yn: &Y, e: &Y, o: &Opts) -> f64 {
    let mut s = 0.0;
    for i in 0..6 { let sc = o.atol + o.rtol*y[i].abs().max(yn[i].abs()); s += (e[i]/sc)*(e[i]/sc); }
    (s/6.0).sqrt()
}

/// integ.rk45: Dormand-Prince 5(4), FSAL.
pub fn rk45<F: FnMut(f64, &Y) -> Y>(mut f: F, t0: f64, tf: f64, y0: Y, opts: &Opts) -> Result<Sol, IntegError> {
    let o = adopt(opts, t0, tf);
    let (mut t, mut y, mut h) = (t0, y0, o.h0.unwrap());
    let mut s = Sol::default();
    s.t.push(t); s.y.push(y); s.d.push(f(t, &y));
    let mut k1 = f(t, &y);
    if !finite(&k1) { return Err(IntegError::NonFinite(t)); }
    let (mut steps, mut iter) = (0usize, 0usize);
    while t < tf - 1e-9*tf.abs().max(1.0) {
        iter += 1;
        if iter > o.maxsteps { return Err(IntegError::MaxSteps(t)); }
        if t + h > tf { h = tf - t; }
        let k2 = f(t + h/5.0, &axpy(&y, h/5.0, &k1));
        let mut z = y; for i in 0..6 { z[i] += h*(3.0/40.0*k1[i] + 9.0/40.0*k2[i]); } let k3 = f(t + 3.0*h/10.0, &z);
        let mut z = y; for i in 0..6 { z[i] += h*(44.0/45.0*k1[i] - 56.0/15.0*k2[i] + 32.0/9.0*k3[i]); } let k4 = f(t + 4.0*h/5.0, &z);
        let mut z = y; for i in 0..6 { z[i] += h*(19372.0/6561.0*k1[i] - 25360.0/2187.0*k2[i] + 64448.0/6561.0*k3[i] - 212.0/729.0*k4[i]); } let k5 = f(t + 8.0*h/9.0, &z);
        let mut z = y; for i in 0..6 { z[i] += h*(9017.0/3168.0*k1[i] - 355.0/33.0*k2[i] + 46732.0/5247.0*k3[i] + 49.0/176.0*k4[i] - 5103.0/18656.0*k5[i]); } let k6 = f(t + h, &z);
        let mut yn = y; for i in 0..6 { yn[i] += h*(35.0/384.0*k1[i] + 500.0/1113.0*k3[i] + 125.0/192.0*k4[i] - 2187.0/6784.0*k5[i] + 11.0/84.0*k6[i]); }
        let k7 = f(t + h, &yn);
        let mut ye = [0.0; 6];
        for i in 0..6 { ye[i] = h*(71.0/57600.0*k1[i] - 71.0/16695.0*k3[i] + 71.0/1920.0*k4[i] - 17253.0/339200.0*k5[i] + 22.0/525.0*k6[i] - 1.0/40.0*k7[i]); }
        if !finite(&yn) { return Err(IntegError::NonFinite(t)); }
        let err = rms_err(&y, &yn, &ye, &o);
        if err <= 1.0 {
            t += h; y = yn; k1 = k7;
            s.t.push(t); s.y.push(y); s.d.push(f(t, &y));
            steps += 1;
        }
        let fac = if err == 0.0 { o.facmax } else { o.facmax.min(o.facmin.max(0.9*err.powf(-1.0/5.0))) };
        h = o.hmax.unwrap().min(o.hmin.max(h*fac));
        if h <= o.hmin*1.0000001 && t < tf { h = o.hmin; }
        if steps > o.maxsteps { break; }
    }
    Ok(s)
}

const C78: [f64; 12] = [1.0/18.0, 1.0/12.0, 1.0/8.0, 5.0/16.0, 3.0/8.0, 59.0/400.0, 93.0/200.0, 5490023248.0/9719169821.0, 13.0/20.0, 1201146811.0/1299019798.0, 1.0, 1.0];
/// Rows = stage j+1's coefficients on F(:,1..13) (the transpose of MATLAB's a').
const A78: [[f64; 13]; 12] = [
    [1.0/18.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0],
    [1.0/48.0, 1.0/16.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0],
    [1.0/32.0, 0.0, 3.0/32.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0],
    [5.0/16.0, 0.0, -75.0/64.0, 75.0/64.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0],
    [3.0/80.0, 0.0, 0.0, 3.0/16.0, 3.0/20.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0],
    [29443841.0/614563906.0, 0.0, 0.0, 77736538.0/692538347.0, -28693883.0/1125000000.0, 23124283.0/1800000000.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0],
    [16016141.0/946692911.0, 0.0, 0.0, 61564180.0/158732637.0, 22789713.0/633445777.0, 545815736.0/2771057229.0, -180193667.0/1043307555.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0],
    [39632708.0/573591083.0, 0.0, 0.0, -433636366.0/683701615.0, -421739975.0/2616292301.0, 100302831.0/723423059.0, 790204164.0/839813087.0, 800635310.0/3783071287.0, 0.0, 0.0, 0.0, 0.0, 0.0],
    [246121993.0/1340847787.0, 0.0, 0.0, -37695042795.0/15268766246.0, -309121744.0/1061227803.0, -12992083.0/490766935.0, 6005943493.0/2108947869.0, 393006217.0/1396673457.0, 123872331.0/1001029789.0, 0.0, 0.0, 0.0, 0.0],
    [-1028468189.0/846180014.0, 0.0, 0.0, 8478235783.0/508512852.0, 1311729495.0/1432422823.0, -10304129995.0/1701304382.0, -48777925059.0/3047939560.0, 15336726248.0/1032824649.0, -45442868181.0/3398467696.0, 3065993473.0/597172653.0, 0.0, 0.0, 0.0],
    [185892177.0/718116043.0, 0.0, 0.0, -3185094517.0/667107341.0, -477755414.0/1098053517.0, -703635378.0/230739211.0, 5731566787.0/1027545527.0, 5232866602.0/850066563.0, -4093664535.0/808688257.0, 3962137247.0/1805957418.0, 65686358.0/487910083.0, 0.0, 0.0],
    [403863854.0/491063109.0, 0.0, 0.0, -5068492393.0/434740067.0, -411421997.0/543043805.0, 652783627.0/914296604.0, 11173962825.0/925320556.0, -13158990841.0/6184727034.0, 3936647629.0/1978049680.0, -160528059.0/685178525.0, 248638103.0/1413531060.0, 0.0, 0.0],
];
const B8: [f64; 13] = [14005451.0/335480064.0, 0.0, 0.0, 0.0, 0.0, -59238493.0/1068277825.0, 181606767.0/758867731.0, 561292985.0/797845732.0, -1041891430.0/1371343529.0, 760417239.0/1151165299.0, 118820643.0/751138087.0, -528747749.0/2220607170.0, 1.0/4.0];
const B7: [f64; 13] = [13451932.0/455176623.0, 0.0, 0.0, 0.0, 0.0, -808719846.0/976000145.0, 1757004468.0/5645159321.0, 656045339.0/265891186.0, -3867574721.0/1518517206.0, 465885868.0/322736535.0, 53011238.0/667516719.0, 2.0/45.0, 0.0];

/// integ.rk78: Dormand-Prince 8(7), 13 stages, adaptive.
pub fn rk78<F: FnMut(f64, &Y) -> Y>(mut f: F, t0: f64, tf: f64, y0: Y, opts: &Opts) -> Result<Sol, IntegError> {
    let o = adopt(opts, t0, tf);
    let (mut t, mut y, mut h) = (t0, y0, o.h0.unwrap());
    let d0 = f(t, &y);
    if !finite(&d0) { return Err(IntegError::NonFinite(t)); }
    let mut s = Sol::default();
    s.t.push(t); s.y.push(y); s.d.push(d0);
    let mut iter = 0usize;
    let mut fk = [[0.0; 6]; 13];
    while t < tf - 1e-9*tf.abs().max(1.0) {
        iter += 1;
        if iter > o.maxsteps { return Err(IntegError::MaxSteps(t)); }
        if t + h > tf { h = tf - t; }
        fk[0] = f(t, &y);
        for j in 0..12 {
            let mut z = y;
            for i in 0..6 { let mut acc = 0.0; for m in 0..13 { acc += fk[m][i]*A78[j][m]; } z[i] += h*acc; }
            fk[j + 1] = f(t + C78[j]*h, &z);
        }
        let (mut y8, mut y7) = (y, y);
        for i in 0..6 {
            let (mut a8, mut a7) = (0.0, 0.0);
            for m in 0..13 { a8 += fk[m][i]*B8[m]; a7 += fk[m][i]*B7[m]; }
            y8[i] += h*a8; y7[i] += h*a7;
        }
        if !finite(&y8) { return Err(IntegError::NonFinite(t)); }
        let mut e = [0.0; 6]; for i in 0..6 { e[i] = y8[i] - y7[i]; }
        let err = rms_err(&y, &y8, &e, &o);
        if err <= 1.0 {
            let dn = f(t + h, &y8);
            if !finite(&dn) { return Err(IntegError::NonFinite(t + h)); }
            t += h; y = y8;
            s.t.push(t); s.y.push(y); s.d.push(dn);
        }
        let fac = if err == 0.0 { o.facmax } else { o.facmax.min(o.facmin.max(0.9*err.powf(-1.0/8.0))) };
        h = o.hmax.unwrap().min(o.hmin.max(h*fac));
    }
    Ok(s)
}

fn luther() -> ([[f64; 7]; 7], [f64; 7], [f64; 7]) {
    let q = 21f64.sqrt();
    let mut a = [[0.0; 7]; 7];
    a[1][0] = 1.0;
    a[2][..2].copy_from_slice(&[3.0/8.0, 1.0/8.0]);
    a[3][..3].copy_from_slice(&[8.0/27.0, 2.0/27.0, 8.0/27.0]);
    a[4][..4].copy_from_slice(&[3.0*(3.0*q - 7.0)/392.0, -8.0*(7.0 - q)/392.0, 48.0*(7.0 - q)/392.0, -3.0*(21.0 - q)/392.0]);
    a[5][..5].copy_from_slice(&[-5.0*(231.0 + 51.0*q)/1960.0, -40.0*(7.0 + q)/1960.0, -320.0*q/1960.0, 3.0*(21.0 + 121.0*q)/1960.0, 392.0*(6.0 + q)/1960.0]);
    a[6][..6].copy_from_slice(&[15.0*(22.0 + 7.0*q)/180.0, 120.0/180.0, 40.0*(7.0*q - 5.0)/180.0, -63.0*(3.0*q - 2.0)/180.0, -14.0*(49.0 + 9.0*q)/180.0, 70.0*(7.0 - q)/180.0]);
    let c = [0.0, 1.0, 0.5, 2.0/3.0, (7.0 - q)/14.0, (7.0 + q)/14.0, 1.0];
    let b = [1.0/20.0, 0.0, 16.0/45.0, 0.0, 49.0/180.0, 49.0/180.0, 1.0/20.0];
    (a, c, b)
}

fn luther_step<F: FnMut(f64, &Y) -> Y>(f: &mut F, t: f64, y: &Y, h: f64) -> Y {
    let (a, c, b) = luther();
    let mut k = [[0.0; 6]; 7];
    for i in 0..7 {
        let mut yi = *y;
        for j in 0..i { for m in 0..6 { yi[m] += h*a[i][j]*k[j][m]; } }
        k[i] = f(t + c[i]*h, &yi);
    }
    let mut o = *y;
    for m in 0..6 { let mut acc = 0.0; for i in 0..7 { acc += k[i][m]*b[i]; } o[m] += h*acc; }
    o
}

/// integ.rk6luther (fixed step, lands on tf).
pub fn rk6luther<F: FnMut(f64, &Y) -> Y>(mut f: F, t0: f64, tf: f64, y0: Y, h: f64) -> Sol {
    let n = (((tf - t0)/h).round() as usize).max(1);
    let h = (tf - t0)/n as f64;
    let mut s = Sol::default();
    let mut y = y0;
    s.t.push(t0); s.y.push(y); s.d.push(f(t0, &y));
    for k in 0..n {
        y = luther_step(&mut f, t0 + k as f64*h, &y, h);
        let tn = t0 + (k + 1) as f64*h;
        s.t.push(tn); s.y.push(y); s.d.push(f(tn, &y));
    }
    s
}

fn acc_of<F: FnMut(f64, &Y) -> Y>(f: &mut F, t: f64, r: &[f64; 3], v: &[f64; 3]) -> [f64; 3] {
    let d = f(t, &[r[0], r[1], r[2], v[0], v[1], v[2]]);
    [d[3], d[4], d[5]]
}

/// integ.nystrom4 (Runge-Kutta-Nystrom, fixed step).
pub fn nystrom4<F: FnMut(f64, &Y) -> Y>(mut f: F, t0: f64, tf: f64, y0: Y, h: f64) -> Sol {
    let n = (((tf - t0)/h).round() as usize).max(1);
    let h = (tf - t0)/n as f64;
    let s5 = 5f64.sqrt();
    let (d2, d3) = ((5.0 - s5)/10.0, (5.0 + s5)/10.0);
    let (a1, b2, c1, c3) = ((3.0 - s5)/20.0, (3.0 + s5)/20.0, (-1.0 + s5)/4.0, (3.0 - s5)/4.0);
    let (ah1, bh1, bh2) = ((5.0 - s5)/10.0, -(5.0 + 3.0*s5)/20.0, (3.0 + s5)/4.0);
    let (ch1, ch2, ch3) = (-(1.0 - 5.0*s5)/4.0, -(5.0 + 3.0*s5)/4.0, (5.0 - s5)/2.0);
    let (al1, al2, al3) = (1.0/12.0, (5.0 + s5)/24.0, (5.0 - s5)/24.0);
    let (be1, be2, be3, be4) = (1.0/12.0, 5.0/12.0, 5.0/12.0, 1.0/12.0);
    let mut r = [y0[0], y0[1], y0[2]];
    let mut v = [y0[3], y0[4], y0[5]];
    let mut s = Sol::default();
    let a0 = acc_of(&mut f, t0, &r, &v);
    s.t.push(t0); s.y.push(y0); s.d.push([v[0], v[1], v[2], a0[0], a0[1], a0[2]]);
    for k in 0..n {
        let t = t0 + k as f64*h;
        let k1 = acc_of(&mut f, t, &r, &v).map(|x| h*x);
        let (mut r2, mut v2) = ([0.0; 3], [0.0; 3]);
        for i in 0..3 { r2[i] = r[i] + d2*h*v[i] + h*(a1*k1[i]); v2[i] = v[i] + ah1*k1[i]; }
        let k2 = acc_of(&mut f, t + d2*h, &r2, &v2).map(|x| h*x);
        let (mut r3, mut v3) = ([0.0; 3], [0.0; 3]);
        for i in 0..3 { r3[i] = r[i] + d3*h*v[i] + h*(b2*k2[i]); v3[i] = v[i] + bh1*k1[i] + bh2*k2[i]; }
        let k3 = acc_of(&mut f, t + d3*h, &r3, &v3).map(|x| h*x);
        let (mut r4, mut v4) = ([0.0; 3], [0.0; 3]);
        for i in 0..3 { r4[i] = r[i] + h*v[i] + h*(c1*k1[i] + c3*k3[i]); v4[i] = v[i] + ch1*k1[i] + ch2*k2[i] + ch3*k3[i]; }
        let k4 = acc_of(&mut f, t + h, &r4, &v4).map(|x| h*x);
        for i in 0..3 {
            r[i] = r[i] + h*v[i] + h*(al1*k1[i] + al2*k2[i] + al3*k3[i]);
            v[i] = v[i] + be1*k1[i] + be2*k2[i] + be3*k3[i] + be4*k4[i];
        }
        let tn = t0 + (k + 1) as f64*h;
        let an = acc_of(&mut f, tn, &r, &v);
        s.t.push(tn); s.y.push([r[0], r[1], r[2], v[0], v[1], v[2]]); s.d.push([v[0], v[1], v[2], an[0], an[1], an[2]]);
    }
    s
}

#[rustfmt::skip]
const GJ_A: [[f64; 9]; 10] = [
[3250433.0/53222400.0,572741.0/5702400.0,-8701681.0/39916800.0,4026311.0/13305600.0,-917039.0/3193344.0,7370669.0/39916800.0,-1025779.0/13305600.0,754331.0/39916800.0,-330157.0/159667200.0],
[-330157.0/159667200.0,530113.0/6652800.0,518887.0/19958400.0,-27631.0/623700.0,44773.0/1064448.0,-531521.0/19958400.0,109343.0/9979200.0,-1261.0/475200.0,45911.0/159667200.0],
[45911.0/159667200.0,-185839.0/39916800.0,171137.0/1900800.0,73643.0/39916800.0,-25775.0/3193344.0,77597.0/13305600.0,-98911.0/39916800.0,24173.0/39916800.0,-3499.0/53222400.0],
[-3499.0/53222400.0,4387.0/4989600.0,-35039.0/4989600.0,90817.0/950400.0,-20561.0/3193344.0,2117.0/9979200.0,2059.0/6652800.0,-317.0/2851200.0,317.0/22809600.0],
[317.0/22809600.0,-2539.0/13305600.0,55067.0/39916800.0,-326911.0/39916800.0,14797.0/152064.0,-326911.0/39916800.0,55067.0/39916800.0,-2539.0/13305600.0,317.0/22809600.0],
[317.0/22809600.0,-317.0/2851200.0,2059.0/6652800.0,2117.0/9979200.0,-20561.0/3193344.0,90817.0/950400.0,-35039.0/4989600.0,4387.0/4989600.0,-3499.0/53222400.0],
[-3499.0/53222400.0,24173.0/39916800.0,-98911.0/39916800.0,77597.0/13305600.0,-25775.0/3193344.0,73643.0/39916800.0,171137.0/1900800.0,-185839.0/39916800.0,45911.0/159667200.0],
[45911.0/159667200.0,-1261.0/475200.0,109343.0/9979200.0,-531521.0/19958400.0,44773.0/1064448.0,-27631.0/623700.0,518887.0/19958400.0,530113.0/6652800.0,-330157.0/159667200.0],
[-330157.0/159667200.0,754331.0/39916800.0,-1025779.0/13305600.0,7370669.0/39916800.0,-917039.0/3193344.0,4026311.0/13305600.0,-8701681.0/39916800.0,572741.0/5702400.0,3250433.0/53222400.0],
[3250433.0/53222400.0,-11011481.0/19958400.0,6322573.0/2851200.0,-8660609.0/1663200.0,25162927.0/3193344.0,-159314453.0/19958400.0,18071351.0/3326400.0,-24115843.0/9979200.0,103798439.0/159667200.0],
];
#[rustfmt::skip]
const GJ_B: [[f64; 9]; 10] = [
[19087.0/89600.0,-427487.0/725760.0,3498217.0/3628800.0,-500327.0/403200.0,6467.0/5670.0,-2616161.0/3628800.0,24019.0/80640.0,-263077.0/3628800.0,8183.0/1036800.0],
[8183.0/1036800.0,57251.0/403200.0,-1106377.0/3628800.0,218483.0/725760.0,-69.0/280.0,530177.0/3628800.0,-210359.0/3628800.0,5533.0/403200.0,-425.0/290304.0],
[-425.0/290304.0,76453.0/3628800.0,5143.0/57600.0,-660127.0/3628800.0,661.0/5670.0,-4997.0/80640.0,83927.0/3628800.0,-19109.0/3628800.0,7.0/12800.0],
[7.0/12800.0,-23173.0/3628800.0,29579.0/725760.0,2497.0/57600.0,-2563.0/22680.0,172993.0/3628800.0,-6463.0/403200.0,2497.0/725760.0,-2497.0/7257600.0],
[-2497.0/7257600.0,1469.0/403200.0,-68119.0/3628800.0,252769.0/3628800.0,0.0,-252769.0/3628800.0,68119.0/3628800.0,-1469.0/403200.0,2497.0/7257600.0],
[2497.0/7257600.0,-2497.0/725760.0,6463.0/403200.0,-172993.0/3628800.0,2563.0/22680.0,-2497.0/57600.0,-29579.0/725760.0,23173.0/3628800.0,-7.0/12800.0],
[-7.0/12800.0,19109.0/3628800.0,-83927.0/3628800.0,4997.0/80640.0,-661.0/5670.0,660127.0/3628800.0,-5143.0/57600.0,-76453.0/3628800.0,425.0/290304.0],
[425.0/290304.0,-5533.0/403200.0,210359.0/3628800.0,-530177.0/3628800.0,69.0/280.0,-218483.0/725760.0,1106377.0/3628800.0,-57251.0/403200.0,-8183.0/1036800.0],
[-8183.0/1036800.0,263077.0/3628800.0,-24019.0/80640.0,2616161.0/3628800.0,-6467.0/5670.0,500327.0/403200.0,-3498217.0/3628800.0,427487.0/725760.0,-19087.0/89600.0],
[25713.0/89600.0,-9401029.0/3628800.0,5393233.0/518400.0,-9839609.0/403200.0,167287.0/4536.0,-135352319.0/3628800.0,10219841.0/403200.0,-40987771.0/3628800.0,3288521.0/1036800.0],
];

fn rk6_rv<F: FnMut(f64, &Y) -> Y>(f: &mut F, t: f64, r: &[f64; 3], v: &[f64; 3], h: f64) -> ([f64; 3], [f64; 3]) {
    let y = luther_step(f, t, &[r[0], r[1], r[2], v[0], v[1], v[2]], h);
    ([y[0], y[1], y[2]], [y[3], y[4], y[5]])
}

fn kahan(s: f64, x: f64, c: f64) -> (f64, f64) { let yk = x - c; let t = s + yk; ((t), (t - s) - yk) }

/// integ.gaussJackson8: 8th-order Gauss-Jackson (summed Stormer-Cowell), RK6 start-up,
/// PEC corrector to 1e-13, Kahan-compensated sums.
pub fn gauss_jackson8<F: FnMut(f64, &Y) -> Y>(mut f: F, t0: f64, tf: f64, y0: Y, h: f64) -> Sol {
    let n = (((tf - t0)/h).round() as usize).max(5);
    let h = (tf - t0)/n as f64;
    let h2 = h*h;
    let mut rr = [[0.0; 3]; 9];
    let mut vv = [[0.0; 3]; 9];
    let mut aa = [[0.0; 3]; 9];
    rr[4] = [y0[0], y0[1], y0[2]]; vv[4] = [y0[3], y0[4], y0[5]];
    aa[4] = acc_of(&mut f, t0, &rr[4], &vv[4]);
    for m in 1..=4 {
        let (rf, vf) = rk6_rv(&mut f, t0 + (m - 1) as f64*h, &rr[3 + m], &vv[3 + m], h);
        rr[4 + m] = rf; vv[4 + m] = vf; aa[4 + m] = acc_of(&mut f, t0 + m as f64*h, &rf, &vf);
        let (rb, vb) = rk6_rv(&mut f, t0 - (m - 1) as f64*h, &rr[5 - m], &vv[5 - m], -h);
        rr[4 - m] = rb; vv[4 - m] = vb; aa[4 - m] = acc_of(&mut f, t0 - m as f64*h, &rb, &vb);
    }
    let mut sn = [[0.0; 3]; 9];
    let mut ss = [[0.0; 3]; 9];
    for c in 0..3 {
        let mut acc = 0.0; for j in 0..9 { acc += GJ_B[4][j]*aa[j][c]; }
        sn[4][c] = vv[4][c]/h - acc;
        let mut acc = 0.0; for j in 0..9 { acc += GJ_A[4][j]*aa[j][c]; }
        ss[4][c] = rr[4][c]/h2 - acc;
    }
    for m in (0..4).rev() { for c in 0..3 { sn[m][c] = sn[m + 1][c] - (aa[m + 1][c] + aa[m][c])/2.0; } }
    for m in 5..9 { for c in 0..3 { sn[m][c] = sn[m - 1][c] + (aa[m - 1][c] + aa[m][c])/2.0; } }
    for m in (0..4).rev() { for c in 0..3 { ss[m][c] = ss[m + 1][c] - sn[m + 1][c] + aa[m + 1][c]/2.0; } }
    for m in 5..9 { for c in 0..3 { ss[m][c] = ss[m - 1][c] + sn[m - 1][c] + aa[m - 1][c]/2.0; } }
    let w = n + 5;                                   // window nodes 1..=W (MATLAB 1-based)
    let mut y = vec![[0.0; 3]; w + 2];
    let mut dy = vec![[0.0; 3]; w + 2];
    let mut ddy = vec![[0.0; 3]; w + 2];
    let mut s2 = vec![[0.0; 3]; w + 2];
    let mut s1 = vec![[0.0; 3]; w + 2];
    let mut s2c = vec![[0.0; 3]; w + 2];
    let mut s1c = vec![[0.0; 3]; w + 2];
    for m in 1..=9 { y[m] = rr[m - 1]; dy[m] = vv[m - 1]; ddy[m] = aa[m - 1]; s2[m] = ss[m - 1]; s1[m] = sn[m - 1]; }
    let (ap, bp, ac, bc) = (GJ_A[9], GJ_B[9], GJ_A[8], GJ_B[8]);
    let mut i = 9;
    while i < w {
        let t = t0 + (i as f64 - 5.0)*h;
        if i > 9 {
            let (mut a43, mut b43) = ([0.0; 3], [0.0; 3]);
            for j in 1..=8 { for c in 0..3 { a43[c] += ac[j - 1]*ddy[i - 9 + j][c]; b43[c] += bc[j - 1]*ddy[i - 9 + j][c]; } }
            for _ in 0..20 {
                for c in 0..3 { let xs = (ddy[i - 1][c] + ddy[i][c])/2.0; let (a, b) = kahan(s1[i - 1][c], xs, s1c[i][c]); s1[i][c] = a; s1c[i][c] = b; }
                let (mut yc, mut dyc) = ([0.0; 3], [0.0; 3]);
                for c in 0..3 { yc[c] = h2*(s2[i][c] + a43[c] + ac[8]*ddy[i][c]); dyc[c] = h*(s1[i][c] + b43[c] + bc[8]*ddy[i][c]); }
                let nrm = |a: &[f64; 3]| (a[0]*a[0] + a[1]*a[1] + a[2]*a[2]).sqrt();
                let rs = nrm(&y[i]).max(1.0);
                let vs = nrm(&dy[i]).max(1.0);
                let dr = [yc[0] - y[i][0], yc[1] - y[i][1], yc[2] - y[i][2]];
                let dv = [dyc[0] - dy[i][0], dyc[1] - dy[i][1], dyc[2] - dy[i][2]];
                let rel = (nrm(&dr)/rs).max(nrm(&dv)/vs);
                y[i] = yc; dy[i] = dyc; ddy[i] = acc_of(&mut f, t, &y[i], &dy[i]);
                if rel <= 1e-13 { break; }
            }
        }
        let (mut a54, mut b54) = ([0.0; 3], [0.0; 3]);
        for j in 1..=9 { for c in 0..3 { a54[c] += ap[j - 1]*ddy[i - 9 + j][c]; b54[c] += bp[j - 1]*ddy[i - 9 + j][c]; } }
        for c in 0..3 {
            let xs = s1[i][c] + ddy[i][c]/2.0;
            let (a, b) = kahan(s2[i][c], xs, s2c[i][c]); s2[i + 1][c] = a; s2c[i + 1][c] = b;
            y[i + 1][c] = h2*(s2[i + 1][c] + a54[c]);
            dy[i + 1][c] = h*(s1[i][c] + ddy[i][c]/2.0 + b54[c]);
        }
        ddy[i + 1] = acc_of(&mut f, t + h, &y[i + 1], &dy[i + 1]);
        s1c[i + 1] = [0.0; 3];
        i += 1;
    }
    let mut s = Sol::default();
    for j in 1..=n + 1 {
        let m = j + 4;
        s.t.push(t0 + (j - 1) as f64*h);
        s.y.push([y[m][0], y[m][1], y[m][2], dy[m][0], dy[m][1], dy[m][2]]);
        s.d.push([dy[m][0], dy[m][1], dy[m][2], ddy[m][0], ddy[m][1], ddy[m][2]]);
    }
    s
}

/// integ.hermite: cubic Hermite dense output between nodes.
pub fn hermite(s: &Sol, tq: f64) -> Y {
    let n = s.t.len();
    let mut i = 0;
    for k in 0..n { if s.t[k] <= tq { i = k; } }
    let i = i.min(n - 2);
    let h = s.t[i + 1] - s.t[i];
    let u = (tq - s.t[i])/h;
    let (u2, u3) = (u*u, u*u*u);
    let (h00, h10, h01, h11) = (2.0*u3 - 3.0*u2 + 1.0, u3 - 2.0*u2 + u, -2.0*u3 + 3.0*u2, u3 - u2);
    let mut o = [0.0; 6];
    for c in 0..6 { o[c] = h00*s.y[i][c] + h10*h*s.d[i][c] + h01*s.y[i + 1][c] + h11*h*s.d[i + 1][c]; }
    o
}

/// integ.run: method by name ("rk4", "rk45", "rk78", "rk6luther", "nystrom4", "gaussjackson8").
pub fn run<F: FnMut(f64, &Y) -> Y>(method: &str, f: F, t0: f64, tf: f64, y0: Y, opts: &Opts) -> Result<Sol, IntegError> {
    match method.to_ascii_lowercase().as_str() {
        "rk4" => Ok(rk4(f, t0, tf, y0, opts.h)),
        "rk45" => rk45(f, t0, tf, y0, opts),
        "rk78" => rk78(f, t0, tf, y0, opts),
        "rk6luther" => Ok(rk6luther(f, t0, tf, y0, opts.h)),
        "nystrom4" => Ok(nystrom4(f, t0, tf, y0, opts.h)),
        "gaussjackson8" => Ok(gauss_jackson8(f, t0, tf, y0, opts.h)),
        m => Err(IntegError::Method(format!("unknown integrator method \"{m}\" (rk4, rk45, rk78, rk6luther, nystrom4, gaussJackson8)"))),
    }
}
