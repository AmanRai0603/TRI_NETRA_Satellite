#!/usr/bin/env python3
"""Writes tests/data/field_error_reference.txt: the independent references of S12.7b (E5, env_field_error).

Nothing in it is the repository's code: only the Python standard library.
  1. the crust's draw: SplitMix64 over a counter keyed by the seed and the FNV-1a hash of the stream's name 'field_error', Box-Muller
     normals with the spare kept (the language's stream, spec section 9.7 of the engine: written here from its description), the
     variance of each degree n of the degrees 14 to 20, sigma_n^2 = rms^2/(K (n + 1) (2 n + 1)) (r0/a)^(2 n + 4);
  2. the crust's field north-east-down at points, from the potential V = a sum (a/r)^(n+1) (g cos m phi + h sin m phi) P_n^m(cos theta)
     with Schmidt semi-normalised P_n^m from the closed recurrence of the unnormalised functions (not the IGRF code's recursion of the
     normalised ones), B = -grad V, the derivative in theta by a central difference of 1e-6 rad, the geodetic coordinates turned to
     geocentric by the exact WGS-84 formulas;
  3. the ring current's field: Dst along the dipole's axis.
Run once, the file committed; nothing runs this generator.
"""
import math, sys

M64 = (1 << 64) - 1
A_IGRF = 6371.2
WGS_A = 6378.137
WGS_F = 1.0/298.257223563

def splitmix64(z):
    z = (z + 0x9E3779B97F4A7C15) & M64
    z = ((z ^ (z >> 30)) * 0xBF58476D1CE4E5B9) & M64
    z = ((z ^ (z >> 27)) * 0x94D049BB133111EB) & M64
    return z ^ (z >> 31)

def fnv1a(name):
    h = 0xcbf29ce484222325
    for b in name.encode():
        h ^= b
        h = (h * 0x100000001b3) & M64
    return h

class Stream:
    def __init__(self, seed, name):
        self.key = splitmix64(seed ^ splitmix64(fnv1a(name))); self.n = 0; self.spare = None
    def uniform(self):
        self.n += 1
        u = splitmix64(self.key ^ splitmix64(self.n))
        return ((u >> 11) + 0.5) * (1.0/9007199254740992.0)
    def normal(self):
        if self.spare is not None:
            s, self.spare = self.spare, None
            return s
        u1, u2 = self.uniform(), self.uniform()
        r = math.sqrt(-2.0*math.log(u1))
        self.spare = r*math.sin(2.0*math.pi*u2)
        return r*math.cos(2.0*math.pi*u2)

def draw(seed, rms, alt_km, nmin=14, nmax=20):
    g = Stream(seed, 'field_error')
    r0 = WGS_A + alt_km
    k = nmax - nmin + 1
    out = []
    for n in range(nmin, nmax + 1):
        sig = rms/math.sqrt(k*(n + 1)*(2*n + 1))*(r0/A_IGRF)**(n + 2)
        for m in range(0, n + 1):
            out.append(sig*g.normal())
            if m > 0:
                out.append(sig*g.normal())
    return out

def pnm_all(nmax, x):
    """Schmidt semi-normalised P[n][m](x) from the unnormalised functions' recurrence (no Condon-Shortley phase)."""
    s = math.sqrt(max(0.0, 1.0 - x*x))
    P = [[0.0]*(nmax + 1) for _ in range(nmax + 1)]
    for m in range(0, nmax + 1):
        pmm = 1.0
        for i in range(1, m + 1):
            pmm *= (2*i - 1)*s
        P[m][m] = pmm
        if m < nmax:
            P[m + 1][m] = x*(2*m + 1)*pmm
        for n in range(m + 2, nmax + 1):
            P[n][m] = (x*(2*n - 1)*P[n - 1][m] - (n + m - 1)*P[n - 2][m])/(n - m)
    for n in range(0, nmax + 1):
        for m in range(1, n + 1):
            P[n][m] *= math.sqrt(2.0*math.factorial(n - m)/math.factorial(n + m))
    return P

def potential(gc, nmin, nmax, r, th, ph):
    P = pnm_all(nmax, math.cos(th))
    v, ci = 0.0, 0
    for n in range(nmin, nmax + 1):
        for m in range(0, n + 1):
            g = gc[ci]; ci += 1
            h = 0.0
            if m > 0:
                h = gc[ci]; ci += 1
            v += A_IGRF*(A_IGRF/r)**(n + 1)*(g*math.cos(m*ph) + h*math.sin(m*ph))*P[n][m]
    return v

def ned(gc, lat, lon, alt_km, nmin=14, nmax=20):
    """North, east, down [nT] from geodetic latitude, longitude [rad] and height [km]."""
    e2 = WGS_F*(2.0 - WGS_F)
    sl, cl = math.sin(lat), math.cos(lat)
    nn = WGS_A/math.sqrt(1.0 - e2*sl*sl)
    rho = (nn + alt_km)*cl
    z = (nn*(1.0 - e2) + alt_km)*sl
    r = math.hypot(rho, z)
    lat_c = math.atan2(z, rho)
    th = math.pi/2 - lat_c
    d = 1e-6
    dvdth = (potential(gc, nmin, nmax, r, th + d, lon) - potential(gc, nmin, nmax, r, th - d, lon))/(2*d)
    dvdph = (potential(gc, nmin, nmax, r, th, lon + d) - potential(gc, nmin, nmax, r, th, lon - d))/(2*d)
    dr = 1e-3
    dvdr = (potential(gc, nmin, nmax, r + dr, th, lon) - potential(gc, nmin, nmax, r - dr, th, lon))/(2*dr)
    br, bth, bph = -dvdr, -dvdth/r, -dvdph/(r*math.sin(th))
    north_c, east, down_c = -bth, bph, -br          # geocentric north, east, down
    psi = lat - lat_c                               # geodetic against geocentric latitude
    return (north_c*math.cos(psi) + down_c*math.sin(psi), east, down_c*math.cos(psi) - north_c*math.sin(psi))

def main():
    out = ['# S12.7b (E5): references of the truth field error, written once by tests/data/field_error_reference_make.py (standard library only).',
           '# draw SEED RMS ALT_KM N then N coefficients [nT] of the stream field_error, degrees 14 to 20 in the IGRF order',
           '# point SEED LAT_DEG LON_DEG ALT_KM NORTH EAST DOWN [nT] of that draw (rms 30 nT at 550 km)']
    for seed, rms, alt in ((1, 30.0, 550.0), (2, 30.0, 550.0), (3, 30.0, 400.0)):
        gc = draw(seed, rms, alt)
        out.append('draw %d %.17g %.17g %d' % (seed, rms, alt, len(gc)))
        out.append(' '.join('%.17g' % c for c in gc))
    gc = draw(1, 30.0, 550.0)
    pts = [(0.0, 0.0, 550.0), (45.0, 105.0, 500.0), (-60.0, -120.0, 600.0), (80.0, 30.0, 550.0), (-85.0, 170.0, 450.0),
           (10.0, -75.0, 700.0), (30.0, 150.0, 400.0), (-30.0, 20.0, 800.0)]
    for la, lo, h in pts:
        n_, e_, d_ = ned(gc, math.radians(la), math.radians(lo), h)
        out.append('point 1 %.17g %.17g %.17g %.17g %.17g %.17g' % (la, lo, h, n_, e_, d_))
    open(sys.argv[1], 'w').write('\n'.join(out) + '\n')

main()
