/* adcs_env.c -- see fsw/pseudocode/02_time_frames_models.md. Every function delegates to its
 * translation of 02_time_frames_models.pc (fsw/alg/src/frames.c, the IGRF table fsw/alg/src/igrf13.c).
 * Owner: Agastya. Copyright (c) 2026 Agastya. All rights reserved. */
#include "adcs_env.h"
#include "adcs_alg_glue.h"

/* written from the design: frames::gmst_rot (fsw/alg) */
void adcs_gmst_rot(double jd, adcs_real C[3][3]) { gl_om33(frames_gmst_rot(jd), &C[0][0]); }

/* J2000 -> mean of date (IAU-76 precession): P = R3(-z) R2(theta) R3(-zeta) */
/* written from the design: frames::prec_rot (fsw/alg) */
void adcs_prec_rot(double jd, adcs_real P[3][3]) { gl_om33(frames_prec_rot(jd), &P[0][0]); }

/* J2000 -> ECEF: precession to the mean equator of date, then GMST (nutation and polar motion omitted) */
/* written from the design: frames::eci2ecef (fsw/alg) */
void adcs_eci2ecef(double jd, adcs_real C[3][3]) { gl_om33(frames_eci2ecef(jd), &C[0][0]); }

/* written from the design: frames::decyear (fsw/alg) */
double adcs_decyear(double jd) { return frames_decyear(jd); }

/* written from the design: frames::sun_model (fsw/alg) */
void adcs_sun_model(double jd, adcs_real s[3]) { gl_o3(frames_sun_model(jd), s); }

/* written from the design: frames::geodetic (fsw/alg) */
void adcs_geodetic(const adcs_real r[3], adcs_real *lat, adcs_real *lon, adcs_real *h)
{
    frames_geodetic_out g = frames_geodetic(gl_v3(r));
    *lat = g.lat; *lon = g.lon; *h = g.h;
}

/* written from the design: frames::igrf_gh (fsw/alg) */
void adcs_igrf_gh(double decyear, adcs_real gh[195])
{
    pc_a195f g = frames_igrf_gh(decyear);
    int k;
    for (k = 0; k < 195; k++) gh[k] = g.v[k];
}

/* written from the design: frames::igrf_ned (fsw/alg) */
void adcs_igrf_ned(const adcs_real gh[195], adcs_real lat, adcs_real lon, adcs_real alt_km, int nmax, adcs_real B[3])
{
    pc_a195f g;
    int k;
    for (k = 0; k < 195; k++) g.v[k] = gh[k];
    gl_o3(frames_igrf_ned(g, lat, lon, alt_km, nmax), B);
}

/* written from the design: frames::field_eci (fsw/alg) */
void adcs_field_eci(const adcs_real r_eci[3], double jd, const adcs_real gh[195], int nmax, adcs_real B[3])
{
    pc_a195f g;
    int k;
    for (k = 0; k < 195; k++) g.v[k] = gh[k];
    gl_o3(frames_field_eci(gl_v3(r_eci), jd, g, nmax), B);
}
