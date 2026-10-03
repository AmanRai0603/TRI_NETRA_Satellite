//! The propagator's time and frame kernel against published reference values, not against its
//! own MATLAB original: the test vectors of ERFA (src/t_erfa_c.c, github.com/liberfa/erfa), the
//! BSD-licensed release of the IAU SOFA library, which carry SOFA's own t_sofa_c.c numbers.
//! Each check names the ERFA routine it reproduces; the tolerances are ERFA's.
use adcs_pop::frames::{c2ixys, era00, fund_args, pom00, s06, sp00, xy06};
use adcs_pop::frames::iau2006::cal2jd_sofa;
use adcs_pop::la::{mm, r3, M3};
use adcs_pop::time;
use std::f64::consts::TAU;

fn vvd(got: f64, want: f64, tol: f64, what: &str) {
    assert!((got - want).abs() <= tol, "{what}: {got:.19e} vs ERFA {want:.19e} (tolerance {tol:e})");
}

fn angle(got: f64, want: f64, tol: f64, what: &str) {
    // ERFA keeps the sign of fmod; the kernel reduces to [0, 2 pi): the same angle
    let d = (got - want).rem_euclid(TAU);
    assert!(d.min(TAU - d) <= tol, "{what}: {got} vs ERFA {want}");
}

fn mat(got: &M3, want: [[f64; 3]; 3], tol: [[f64; 3]; 3], what: &str) {
    for i in 0..3 { for j in 0..3 { vvd(got[i][j], want[i][j], tol[i][j], &format!("{what}[{i}][{j}]")); } }
}

/// TT Julian centuries since J2000.0 of a two-part date.
fn centuries(d1: f64, d2: f64) -> f64 { ((d1 - 2451545.0) + d2)/36525.0 }

#[test]
fn earth_rotation_angle_era00() {
    vvd(era00(2400000.5, 54388.0), 0.4022837240028158102, 1e-12, "eraEra00");
}

#[test]
fn fundamental_arguments_fa03() {
    let fa = fund_args(0.80);
    let want = [(0, 5.132369751108684150, "eraFal03"), (1, 6.226797973505507345, "eraFalp03"), (2, 0.2597711366745499518, "eraFaf03"),
                (3, 1.946709205396925672, "eraFad03"), (4, -5.973618440951302183, "eraFaom03"), (5, 5.417338184297289661, "eraFame03"),
                (6, 3.424900460533758000, "eraFave03"), (7, 1.744713738913081846, "eraFae03"), (8, 3.275506840277781492, "eraFama03"),
                (9, 5.275711665202481138, "eraFaju03"), (10, 5.371574539440827046, "eraFasa03"), (11, 5.180636450180413523, "eraFaur03"),
                (12, 2.079343830860413523, "eraFane03")];
    for (i, v, name) in want { angle(fa[i], v, 1e-12, name); }
    vvd(fa[13], 0.1950884762240000000e-1, 1e-12, "eraFapa03");
}

#[test]
fn cip_x_y_xy06_and_cio_locator_s06() {
    let t = centuries(2400000.5, 53736.0);
    let fa = fund_args(t);
    let (x, y) = xy06(t, &fa);
    vvd(x, 0.5791308486706010975e-3, 1e-15, "eraXy06 x");
    vvd(y, 0.4020579816732958141e-4, 1e-16, "eraXy06 y");
    let s = s06(t, &fa, 0.5791308486706011000e-3, 0.4020579816732961219e-4);
    vvd(s, -0.1220032213076463117e-7, 1e-18, "eraS06");
}

#[test]
fn tio_locator_sp00() {
    vvd(sp00(centuries(2400000.5, 52541.0)), -0.6216698469981019309e-11, 1e-12, "eraSp00");
}

#[test]
fn celestial_to_intermediate_c2ixys() {
    let m = c2ixys(0.5791308486706011000e-3, 0.4020579816732961219e-4, -0.1220040848472271978e-7);
    mat(&m, [[0.9999998323037157138, 0.5581984869168499149e-9, -0.5791308491611282180e-3],
             [-0.2384261642670440317e-7, 0.9999999991917468964, -0.4020579110169668931e-4],
             [0.5791308486706011000e-3, 0.4020579816732961219e-4, 0.9999998314954627590]], [[1e-12; 3]; 3], "eraC2ixys");
}

#[test]
fn polar_motion_pom00() {
    let m = pom00(2.55060238e-7, 1.860359247e-6, -0.1367174580728891460e-10);
    mat(&m, [[0.9999999999999674721, -0.1367174580728846989e-10, 0.2550602379999972345e-6],
             [0.1414624947957029801e-10, 0.9999999999982695317, -0.1860359246998866389e-5],
             [-0.2550602379741215021e-6, 0.1860359247002414021e-5, 0.9999999999982370039]],
        [[1e-12, 1e-16, 1e-16], [1e-16, 1e-12, 1e-16], [1e-16, 1e-16, 1e-12]], "eraPom00");
}

#[test]
fn celestial_to_terrestrial_c2t06a_from_its_parts() {
    // eraC2t06a(TT = UT1 = MJD 53736.0, xp, yp) = W(xp, yp, s') R3(ERA) C2I(X, Y, s). ERFA takes X, Y
    // from the full precession-nutation matrix; the kernel uses the X, Y series (eraXy06, checked
    // above to 1e-15), which ERFA documents as agreeing with it to about 1 microarcsecond (5e-12 rad)
    let (d1, d2) = (2400000.5, 53736.0);
    let t = centuries(d1, d2);
    let fa = fund_args(t);
    let (x, y) = xy06(t, &fa);
    let c2i = c2ixys(x, y, s06(t, &fa, x, y));
    let (xp, yp) = (2.55060238e-7, 1.860359247e-6);
    let m = mm(&pom00(xp, yp, sp00(t)), &mm(&r3(era00(d1, d2)), &c2i));
    mat(&m, [[-0.1810332128305897282, 0.9834769806938592296, 0.6555550962998436505e-4],
             [-0.9834768134136214897, -0.1810332203649130832, 0.5749800844905594110e-3],
             [0.5773474024748545878e-3, 0.3961816829632690581e-4, 0.9999998325501747785]], [[1e-11; 3]; 3], "eraC2t06a");
}

#[test]
fn calendar_and_leap_seconds_cal2jd_dat() {
    let (djm0, djm) = cal2jd_sofa(2003.0, 6.0, 1.0);
    assert_eq!((djm0, djm), (2400000.5, 52791.0), "eraCal2jd");
    for (y, m, d, want) in [(2003.0, 6.0, 1.0, 32.0), (2008.0, 1.0, 17.0, 33.0), (2017.0, 9.0, 1.0, 37.0)] {
        assert_eq!(time::tai_minus_utc(time::cal2jd(y, m, d, 0.0, 0.0, 0.0)), want, "eraDat {y}-{m}-{d}");
    }
}
