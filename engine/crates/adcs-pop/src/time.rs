//! Time scales -- port of `matlab_sils/pop/03_frames_time/ephemeris/+timeconv/*`
//! (cal2jd, jd2cal, jd2mjd, mjd2jd, leapTable, taiMinusUTC, utc2tai, tai2utc,
//! tai2tt, tt2tai, tt2utc, utc2tt, tt2tdb, utc2tdb, utc2ut1, tai2gps, utc2gps,
//! tt2jc, j2000sec, gpsWeekSow, gmst82, doy, convertUTC), plus
//! `01_core/+op/addsec.m` (with the Octave `datenum`/`datevec` it relies on).
//!
//! Every function keeps the MATLAB formula and order of operations, so results
//! agree with Octave to the last bit or within a few ulp. `convert_utc` is
//! allocation-free.

/// Octave's `mod(x, y)` for doubles (liboctave `octave::math::mod`): returns 0
/// when `y` is non-integer and `x/y` is within one relative eps of an integer,
/// otherwise `x - y*floor(x/y)`; the result takes the sign of `y`.
/// The POP formulas use `mod` everywhere, so this is the one used by the port.
#[inline]
pub fn omod(x: f64, y: f64) -> f64 {
    let mut r;
    if y == 0.0 {
        r = x;
    } else {
        let q = x / y;
        let nint_y = (y + 0.5).floor();
        let nint_q = if q.is_finite() { (q + 0.5).floor() } else { q };
        if nint_y != y && ((q - nint_q) / nint_q).abs() < f64::EPSILON {
            r = 0.0;
        } else {
            let n = q.floor();
            r = x - y * n;
        }
    }
    if x != y && y != 0.0 {
        r = r.abs().copysign(y);
    }
    r
}

/// Octave `deg2rad(deg) = deg * (pi/180)`.
#[inline]
pub fn deg2rad(deg: f64) -> f64 { deg * (std::f64::consts::PI / 180.0) }

/// `timeconv.cal2jd(Y,M,D,h,mi,s)`: Gregorian calendar -> Julian Date (scale-agnostic).
#[inline]
pub fn cal2jd(y: f64, mo: f64, d: f64, h: f64, mi: f64, s: f64) -> f64 {
    let a = ((14.0 - mo) / 12.0).floor();
    let yy = y + 4800.0 - a;
    let m = mo + 12.0 * a - 3.0;
    let jdn = d + ((153.0 * m + 2.0) / 5.0).floor() + 365.0 * yy + (yy / 4.0).floor() - (yy / 100.0).floor()
        + (yy / 400.0).floor() - 32045.0;
    jdn + (h - 12.0) / 24.0 + mi / 1440.0 + s / 86400.0
}

/// `timeconv.jd2cal(jd)`: Julian Date -> Gregorian `[Y M D h mi s]` (scale-agnostic).
pub fn jd2cal(jd: f64) -> [f64; 6] {
    let q = jd + 0.5;
    let z = q.floor();
    let f = q - z;
    let a = if z < 2299161.0 {
        z
    } else {
        let al = ((z - 1867216.25) / 36524.25).floor();
        z + 1.0 + al - (al / 4.0).floor()
    };
    let b = a + 1524.0;
    let c = ((b - 122.1) / 365.25).floor();
    let dd = (365.25 * c).floor();
    let e = ((b - dd) / 30.6001).floor();
    let day = b - dd - (30.6001 * e).floor() + f;
    let m = if e < 14.0 { e - 1.0 } else { e - 13.0 };
    let y = if m > 2.0 { c - 4716.0 } else { c - 4715.0 };
    let d = day.floor();
    let mut fr = (day - d) * 24.0;
    let h = fr.floor();
    fr = (fr - h) * 60.0;
    let mi = fr.floor();
    let s = (fr - mi) * 60.0;
    [y, m, d, h, mi, s]
}

/// `timeconv.jd2mjd`: JD -> MJD.
#[inline] pub fn jd2mjd(jd: f64) -> f64 { jd - 2400000.5 }
/// `timeconv.mjd2jd`: MJD -> JD.
#[inline] pub fn mjd2jd(mjd: f64) -> f64 { mjd + 2400000.5 }

/// `timeconv.leapTable()`: `[year month TAI-UTC(s)]` at 0h UTC of the 1st of that month.
pub const LEAP_TABLE: [[f64; 3]; 28] = [
    [1972., 1., 10.], [1972., 7., 11.], [1973., 1., 12.], [1974., 1., 13.], [1975., 1., 14.], [1976., 1., 15.],
    [1977., 1., 16.], [1978., 1., 17.], [1979., 1., 18.], [1980., 1., 19.], [1981., 7., 20.], [1982., 7., 21.],
    [1983., 7., 22.], [1985., 7., 23.], [1988., 1., 24.], [1990., 1., 25.], [1991., 1., 26.], [1992., 7., 27.],
    [1993., 7., 28.], [1994., 7., 29.], [1996., 1., 30.], [1997., 7., 31.], [1999., 1., 32.], [2006., 1., 33.],
    [2009., 1., 34.], [2012., 7., 35.], [2015., 7., 36.], [2017., 1., 37.],
];

/// `timeconv.taiMinusUTC(jd_utc)`: integer leap-second offset TAI-UTC (s) at a UTC JD
/// (10 before 1972; last table entry whose `cal2jd(Y,M,1)` is <= jd_utc).
#[inline]
pub fn tai_minus_utc(jd_utc: f64) -> f64 {
    let mut n = 10.0;
    for l in LEAP_TABLE.iter() {
        if jd_utc >= cal2jd(l[0], l[1], 1.0, 0.0, 0.0, 0.0) { n = l[2]; }
    }
    n
}

/// `timeconv.utc2tai`: UTC JD -> TAI JD.
#[inline] pub fn utc2tai(jd_utc: f64) -> f64 { jd_utc + tai_minus_utc(jd_utc) / 86400.0 }
/// `timeconv.tai2utc`: TAI JD -> UTC JD (leap lookup on the TAI date, as MATLAB).
#[inline] pub fn tai2utc(jd_tai: f64) -> f64 { jd_tai - tai_minus_utc(jd_tai) / 86400.0 }
/// `timeconv.tai2tt`: TAI JD -> TT JD (+32.184 s).
#[inline] pub fn tai2tt(jd_tai: f64) -> f64 { jd_tai + 32.184 / 86400.0 }
/// `timeconv.tt2tai`: TT JD -> TAI JD.
#[inline] pub fn tt2tai(jd_tt: f64) -> f64 { jd_tt - 32.184 / 86400.0 }
/// `timeconv.tt2utc`: TT JD -> UTC JD.
#[inline] pub fn tt2utc(jd_tt: f64) -> f64 { tai2utc(tt2tai(jd_tt)) }
/// `timeconv.utc2tt`: UTC JD -> TT JD.
#[inline] pub fn utc2tt(jd_utc: f64) -> f64 { tai2tt(utc2tai(jd_utc)) }
/// `timeconv.tt2tdb`: TT JD -> TDB JD (two-term periodic model).
#[inline]
pub fn tt2tdb(jd_tt: f64) -> f64 {
    let g = deg2rad(357.53 + 0.9856003 * (jd_tt - 2451545.0));
    jd_tt + (0.001658 * g.sin() + 0.000014 * (2.0 * g).sin()) / 86400.0
}
/// `timeconv.utc2tdb`: UTC JD -> TDB JD.
#[inline] pub fn utc2tdb(jd_utc: f64) -> f64 { tt2tdb(utc2tt(jd_utc)) }
/// `timeconv.utc2ut1(jd_utc, dUT1)`: UTC JD -> UT1 JD, dUT1 = UT1-UTC (s).
#[inline] pub fn utc2ut1(jd_utc: f64, dut1: f64) -> f64 { jd_utc + dut1 / 86400.0 }
/// `timeconv.tai2gps`: TAI JD -> GPS JD (-19 s).
#[inline] pub fn tai2gps(jd_tai: f64) -> f64 { jd_tai - 19.0 / 86400.0 }
/// `timeconv.utc2gps`: UTC JD -> GPS JD.
#[inline] pub fn utc2gps(jd_utc: f64) -> f64 { tai2gps(utc2tai(jd_utc)) }
/// `timeconv.tt2jc`: TT JD -> Julian centuries since J2000.
#[inline] pub fn tt2jc(jd_tt: f64) -> f64 { (jd_tt - 2451545.0) / 36525.0 }
/// `timeconv.j2000sec`: TT JD -> seconds past J2000 TT.
#[inline] pub fn j2000sec(jd_tt: f64) -> f64 { (jd_tt - 2451545.0) * 86400.0 }
/// `timeconv.gpsWeekSow`: GPS JD -> (GPS week, seconds of week); epoch 1980-01-06.
#[inline]
pub fn gps_week_sow(jd_gps: f64) -> (f64, f64) {
    let dt = jd_gps - 2444244.5;
    let week = (dt / 7.0).floor();
    (week, (dt - week * 7.0) * 86400.0)
}
/// `timeconv.gmst82(jd_ut1)`: Greenwich Mean Sidereal Time (IAU 1982), rad.
#[inline]
pub fn gmst82(jd_ut1: f64) -> f64 {
    let tu = (jd_ut1 - 2451545.0) / 36525.0;
    let sec = 67310.54841 + (876600.0 * 3600.0 + 8640184.812866) * tu + 0.093104 * tu.powf(2.0) - 6.2e-6 * tu.powf(3.0);
    deg2rad(omod(sec, 86400.0) / 240.0)
}
/// `timeconv.doy(Y,M,D)`: day of year (1..366).
#[inline]
pub fn doy(y: f64, m: f64, d: f64) -> f64 {
    cal2jd(y, m, d, 0.0, 0.0, 0.0) - cal2jd(y, 1.0, 1.0, 0.0, 0.0, 0.0) + 1.0
}

/// Output of `timeconv.convertUTC` -- one field per MATLAB struct field.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Times {
    /// `.utc_jd` UTC Julian date.
    pub utc_jd: f64,
    /// `.tai_jd` TAI Julian date.
    pub tai_jd: f64,
    /// `.tt_jd` TT Julian date.
    pub tt_jd: f64,
    /// `.tdb_jd` TDB Julian date (feed to DE440).
    pub tdb_jd: f64,
    /// `.ut1_jd` UT1 Julian date.
    pub ut1_jd: f64,
    /// `.gps_jd` GPS-time Julian date.
    pub gps_jd: f64,
    /// `.utc_mjd` UTC modified Julian date.
    pub utc_mjd: f64,
    /// `.tt_mjd` TT modified Julian date.
    pub tt_mjd: f64,
    /// `.tdb_mjd` TDB modified Julian date.
    pub tdb_mjd: f64,
    /// `.leap` TAI-UTC (s).
    pub leap: f64,
    /// `.dUT1` UT1-UTC (s), as passed in.
    pub dut1: f64,
    /// `.T_tt` Julian centuries TT since J2000.
    pub t_tt: f64,
    /// `.j2000_tt_sec` seconds past J2000 TT.
    pub j2000_tt_sec: f64,
    /// `.gps_week` GPS week.
    pub gps_week: f64,
    /// `.gps_sow` GPS seconds of week.
    pub gps_sow: f64,
    /// `.gmst_rad` GMST (IAU 1982) from UT1, rad.
    pub gmst_rad: f64,
    /// `.doy` day of year.
    pub doy: f64,
}

/// `timeconv.convertUTC(Y,M,D,h,mi,s,dUT1)`: one UTC calendar instant -> all time
/// scales. `dut1` = UT1-UTC (s); MATLAB's default is 0. Allocation-free; the leap
/// lookup (which MATLAB repeats inside each utc2* call) is done once -- it is a pure
/// function of the same UTC JD, so the values are identical.
pub fn convert_utc(y: f64, mo: f64, d: f64, h: f64, mi: f64, s: f64, dut1: f64) -> Times {
    let jdu = cal2jd(y, mo, d, h, mi, s);
    let leap = tai_minus_utc(jdu);
    let tai_jd = jdu + leap / 86400.0;
    let tt_jd = tai2tt(tai_jd);
    let tdb_jd = tt2tdb(tt_jd);
    let ut1_jd = utc2ut1(jdu, dut1);
    let gps_jd = tai2gps(tai_jd);
    let (gps_week, gps_sow) = gps_week_sow(gps_jd);
    Times {
        utc_jd: jdu,
        tai_jd,
        tt_jd,
        tdb_jd,
        ut1_jd,
        gps_jd,
        utc_mjd: jd2mjd(jdu),
        tt_mjd: jd2mjd(tt_jd),
        tdb_mjd: jd2mjd(tdb_jd),
        leap,
        dut1,
        t_tt: tt2jc(tt_jd),
        j2000_tt_sec: j2000sec(tt_jd),
        gps_week,
        gps_sow,
        gmst_rad: gmst82(ut1_jd),
        doy: doy(y, mo, d),
    }
}

/// Octave `datenum(year, month, day, hour, minute, second)` for scalar doubles
/// (days since year 0; used by `op.addsec`).
pub fn datenum(year: f64, month: f64, day: f64, hour: f64, minute: f64, second: f64) -> f64 {
    const MONTHSTART: [f64; 12] = [306., 337., 0., 31., 61., 92., 122., 153., 184., 214., 245., 275.];
    const MONTHLENGTH: [f64; 12] = [31., 28., 31., 30., 31., 30., 31., 31., 30., 31., 30., 31.];
    let is_leap = |y: f64| (omod(y, 4.0) == 0.0 && omod(y, 100.0) != 0.0) || omod(y, 400.0) == 0.0;
    let mut year = year;
    let mut month = if month < 1.0 { 1.0 } else { month };
    let mut day = day;
    if month != month.trunc() {
        let fracmonth = month - month.floor();
        month = month.floor();
        let mi = (omod(month - 1.0, 12.0) + 1.0) as usize - 1;
        if mi != 1 || !is_leap(year.floor()) {
            day += fracmonth * MONTHLENGTH[mi];
        } else {
            day += fracmonth * 29.0;
        }
    }
    year += ((month - 14.0) / 12.0).ceil();
    day += MONTHSTART[(omod(month - 1.0, 12.0) + 1.0) as usize - 1] + 60.0;
    if year != year.trunc() {
        let fracyear = year - year.floor();
        year = year.floor();
        day += fracyear * (365.0 + if is_leap(year + 1.0) { 1.0 } else { 0.0 });
    }
    day += 365.0 * year + (year / 4.0).floor() - (year / 100.0).floor() + (year / 400.0).floor();
    day + (hour + (minute + second / 60.0) / 60.0) / 24.0
}

/// Octave `datevec(datenum)` for a scalar: `[Y Mo D H Mi S]`, seconds rounded to
/// the datenum's resolution exactly as Octave does.
pub fn datevec(date: f64) -> [f64; 6] {
    let z = date.floor() - 60.0;
    let a = ((z - 0.25) / 36524.25).floor();
    let b = z - 0.25 + a - (a / 4.0).floor();
    let mut y = (b / 365.25).floor();
    let c = (b - (365.25 * y).floor()).trunc() + 1.0;
    let mut m = ((5.0 * c + 456.0) / 153.0).trunc();
    let d = c - ((153.0 * m - 457.0) / 5.0).trunc();
    if m > 12.0 {
        y += 1.0;
        m -= 12.0;
    }
    let fracd = date - date.floor();
    let mut tmps = (f64::EPSILON * 86400.0 * date).abs();
    if tmps == 0.0 { tmps = 1.0; }
    let srnd = 2f64.powf((-tmps.log2()).floor());
    let mut s = (86400.0 * fracd * srnd).round() / srnd;
    let h = (s / 3600.0).floor();
    s -= 3600.0 * h;
    let mi = (s / 60.0).floor();
    s -= 60.0 * mi;
    [y, m, d, h, mi, s]
}

/// `op.addsec(utc0, dt)`: add `sec` seconds to a `[Y Mo D H Mi S]` UTC vector
/// (via `datenum`/`datevec`, handles rollover; leap seconds are not represented,
/// exactly as in MATLAB).
pub fn addsec(utc: [f64; 6], sec: f64) -> [f64; 6] {
    let dn = datenum(utc[0], utc[1], utc[2], utc[3], utc[4], utc[5]) + sec / 86400.0;
    datevec(dn)
}
