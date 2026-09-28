/* adcs_env.h -- onboard time, frames and environment models (fsw/pseudocode/02).
 * Owner: Agastya. Copyright (c) 2026 Agastya. All rights reserved. */
#ifndef ADCS_ENV_H
#define ADCS_ENV_H
#include "adcs_math.h"

void adcs_gmst_rot(double jd, adcs_real C[3][3]);          /* ECI -> ECEF */
double adcs_decyear(double jd);
void adcs_sun_model(double jd, adcs_real s[3]);             /* unit, J2000 */
void adcs_geodetic(const adcs_real r_ecef[3], adcs_real *lat, adcs_real *lon, adcs_real *h);
/* interpolated Gauss coefficients for decimal year (195, nT) */
void adcs_igrf_gh(double decyear, adcs_real gh[195]);
void adcs_igrf_ned(const adcs_real gh[195], adcs_real lat, adcs_real lon, adcs_real alt_km, int nmax, adcs_real B[3]);
/* field in ECI [T] at r_eci [m] */
void adcs_field_eci(const adcs_real r_eci[3], double jd, const adcs_real gh[195], int nmax, adcs_real B[3]);

#endif
