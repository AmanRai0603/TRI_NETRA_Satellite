/* adcs_env.c -- see fsw/pseudocode/02_time_frames_models.md.
 * Owner: Agastya. Copyright (c) 2026 Agastya. All rights reserved. */
#include "adcs_env.h"
#include "adcs_igrf13.h"

void adcs_gmst_rot(double jd, adcs_real C[3][3])
{
    double T = (jd - 2451545.0)/36525.0;
    double g = fmod(67310.54841 + (876600.0*3600.0 + 8640184.812866)*T + 0.093104*T*T - 6.2e-6*T*T*T, 86400.0);
    if (g < 0) g += 86400.0;
    g = g/240.0*ADCS_D2R;
    C[0][0] = cos(g);  C[0][1] = sin(g); C[0][2] = 0;
    C[1][0] = -sin(g); C[1][1] = cos(g); C[1][2] = 0;
    C[2][0] = 0;       C[2][1] = 0;      C[2][2] = 1;
}

double adcs_decyear(double jd) { return 2000.0 + (jd - 2451544.5)/365.25; }

static double mod360(double x) { x = fmod(x, 360.0); return x < 0 ? x + 360.0 : x; }

void adcs_sun_model(double jd, adcs_real s[3])
{
    double T = (jd - 2451545.0)/36525.0;
    double L = mod360(280.460 + 36000.771*T);
    double M = mod360(357.5291092 + 35999.05034*T)*ADCS_D2R;
    double lam = (L + 1.914666471*sin(M) + 0.019994643*sin(2*M))*ADCS_D2R;
    double eps = (23.439291 - 0.0130042*T)*ADCS_D2R;
    double sm[3], as = ADCS_PI/(180.0*3600.0);
    double zeta = (2306.2181*T + 0.30188*T*T)*as, z = (2306.2181*T + 1.09468*T*T)*as, th = (2004.3109*T - 0.42665*T*T)*as;
    double a[3], b[3];
    sm[0] = cos(lam); sm[1] = cos(eps)*sin(lam); sm[2] = sin(eps)*sin(lam);
    /* s = R3(zeta) R2(-th) R3(z) s_mod,  R3(x) = [c s 0; -s c 0; 0 0 1], R2(x) = [c 0 -s; 0 1 0; s 0 c] */
    a[0] = cos(z)*sm[0] + sin(z)*sm[1];  a[1] = -sin(z)*sm[0] + cos(z)*sm[1];  a[2] = sm[2];
    b[0] = cos(-th)*a[0] - sin(-th)*a[2]; b[1] = a[1]; b[2] = sin(-th)*a[0] + cos(-th)*a[2];
    s[0] = cos(zeta)*b[0] + sin(zeta)*b[1]; s[1] = -sin(zeta)*b[0] + cos(zeta)*b[1]; s[2] = b[2];
    adcs_unit(s, s);
}

void adcs_geodetic(const adcs_real r[3], adcs_real *lat, adcs_real *lon, adcs_real *h)
{
    const double a = 6378137.0, f = 1.0/298.257223563, e2 = f*(2.0 - f);
    double p = sqrt(r[0]*r[0] + r[1]*r[1]), la, N = a, hh = 0;
    int k;
    *lon = atan2(r[1], r[0]);
    la = atan2(r[2], p*(1.0 - e2));
    for (k = 0; k < 3; k++) {
        double sl = sin(la);
        N = a/sqrt(1.0 - e2*sl*sl);
        hh = p/cos(la) - N;
        la = atan2(r[2], p*(1.0 - e2*N/(N + hh)));
    }
    *lat = la; *h = hh;
}

void adcs_igrf_gh(double decyear, adcs_real gh[195])
{
    int i = 0, k;
    double f;
    while (i + 1 < ADCS_IGRF_EPOCHS && adcs_igrf_year[i + 1] <= decyear) i++;
    if (i >= ADCS_IGRF_EPOCHS - 1) i = ADCS_IGRF_EPOCHS - 2;       /* extrapolate from the last interval */
    f = (decyear - adcs_igrf_year[i])/(adcs_igrf_year[i + 1] - adcs_igrf_year[i]);
    for (k = 0; k < ADCS_IGRF_NCOEF; k++) gh[k] = adcs_igrf_tab[i][k] + (adcs_igrf_tab[i + 1][k] - adcs_igrf_tab[i][k])*f;
}

void adcs_igrf_ned(const adcs_real gh[195], adcs_real lat, adcs_real lon, adcs_real alt_km, int nmax, adcs_real B[3])
{
    /* Standard Code env.igrf: geodetic -> geocentric, Schmidt semi-normalised recursion */
    const double Re = 6371.2, a = 6378.137, f = 1.0/298.257223563, b = a*(1.0 - f);
    double ct = cos(ADCS_PI/2 - lat), st = sin(ADCS_PI/2 - lat);
    double rho = sqrt((a*st)*(a*st) + (b*ct)*(b*ct));
    double r = sqrt(alt_km*alt_km + 2*alt_km*rho + (a*a*a*a*st*st + b*b*b*b*ct*ct)/(rho*rho));
    double cd = (alt_km + rho)/r, sd = (a*a - b*b)/rho*ct*st/r, oc = ct;
    double P[120], dP[120], cphi[14], sphi[14], Br = 0, Bt = 0, Bp = 0, a_r;
    int Pmax, Pi, m = 1, n = 0, ci = 0, k;
    if (nmax > 13) nmax = 13;
    ct = ct*cd - st*sd; st = st*cd + oc*sd;
    for (k = 1; k <= nmax; k++) { cphi[k] = cos(k*lon); sphi[k] = sin(k*lon); }
    Pmax = (nmax + 1)*(nmax + 2)/2;
    for (k = 0; k < Pmax; k++) { P[k] = 0; dP[k] = 0; }
    P[0] = 1; P[2] = st; dP[2] = ct;               /* 1-based P(1), P(3) */
    a_r = (Re/r)*(Re/r);
    for (Pi = 2; Pi <= Pmax; Pi++) {              /* Pi is the 1-based index of the MATLAB source */
        int ix = Pi - 1;
        if (n < m) { m = 0; n = n + 1; a_r = a_r*(Re/r); }
        if (m < n && Pi != 3) {
            int l1 = Pi - n - 1, l2 = Pi - 2*n;          /* 0-based of (Pi - n), (Pi - 2n + 1) */
            double k1 = (2.0*n - 1)/sqrt((double)n*n - (double)m*m), k2 = sqrt((((double)n - 1)*((double)n - 1) - (double)m*m)/((double)n*n - (double)m*m));
            P[ix] = k1*ct*P[l1] - k2*P[l2];
            dP[ix] = k1*(ct*dP[l1] - st*P[l1]) - k2*dP[l2];
        } else if (Pi != 3) {
            int ln = Pi - n - 2;                          /* 0-based of (Pi - n - 1) */
            double kk = sqrt(1.0 - 1.0/(2.0*m));
            P[ix] = kk*st*P[ln];
            dP[ix] = kk*(st*dP[ln] + ct*P[ln]);
        }
        if (m == 0) {
            double c = a_r*gh[ci];
            Br += (n + 1)*c*P[ix]; Bt -= c*dP[ix];
            ci += 1;
        } else {
            double gc = gh[ci]*cphi[m] + gh[ci + 1]*sphi[m];
            double gs = -gh[ci]*sphi[m] + gh[ci + 1]*cphi[m];
            double c = a_r*gc;
            Br += (n + 1)*c*P[ix]; Bt -= c*dP[ix];
            if (st == 0) Bp -= ct*a_r*gs*dP[ix];
            else Bp -= 1.0/st*a_r*m*gs*P[ix];
            ci += 2;
        }
        m = m + 1;
    }
    {
        double Bx = -Bt, By = Bp, Bz = -Br;
        B[0] = Bx*cd + Bz*sd; B[1] = By; B[2] = Bz*cd - Bx*sd;
    }
}

void adcs_field_eci(const adcs_real r_eci[3], double jd, const adcs_real gh[195], int nmax, adcs_real B[3])
{
    adcs_real C[3][3], re[3], lat, lon, h, Bn[3], Be[3];
    double sl, cl, so, co;
    adcs_gmst_rot(jd, C);
    adcs_mat3_vec(C, r_eci, re);
    adcs_geodetic(re, &lat, &lon, &h);
    adcs_igrf_ned(gh, lat, lon, h/1000.0, nmax, Bn);
    Bn[0] *= 1e-9; Bn[1] *= 1e-9; Bn[2] *= 1e-9;
    sl = sin(lat); cl = cos(lat); so = sin(lon); co = cos(lon);
    Be[0] = -sl*co*Bn[0] - so*Bn[1] - cl*co*Bn[2];
    Be[1] = -sl*so*Bn[0] + co*Bn[1] - cl*so*Bn[2];
    Be[2] = cl*Bn[0] - sl*Bn[2];
    adcs_mat3t_vec(C, Be, B);
}
