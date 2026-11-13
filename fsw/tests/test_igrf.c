/*
 * test_igrf.c -- the flight software's geomagnetic field against the committed IGRF-14 reference (tests/data/igrf14_reference.txt,
 * S12.6, TS1(f)): 50 points computed by two independent implementations, ppigrf 2.1.0 and pyIGRF14 1.0.4, at degree 13 and cut to
 * degree 10. The software's field (adcs_igrf_gh, adcs_igrf_ned) is held to each package at its degree, to 1 nT; the packages are held
 * to each other, to 0.01 nT, so the reference itself is checked. Run as `test_igrf FILE` (make test passes the path).
 * Owner: Agastya. Copyright (c) 2026 Agastya. All rights reserved.
 */
#include <math.h>
#include <stdio.h>
#include <stdlib.h>
#include "adcs_env.h"

#define PI 3.14159265358979323846

int main(int argc, char **argv)
{
    FILE *f = fopen(argc > 1 ? argv[1] : "../tests/data/igrf14_reference.txt", "r");
    char line[1024];
    int n_rows = 0, bad = 0;
    double worst[2] = {0, 0}, spread = 0;
    if (!f) { printf("FAIL cannot open the reference file\n"); return 1; }
    while (fgets(line, sizeof line, f)) {
        double v[16], gh[195], b[3];
        char *p = line, *e;
        int n, deg, k, who, m;
        if (line[0] == '#' || line[0] == '\n') continue;
        n = (int)strtol(p, &e, 10);
        for (m = 0, p = e; m < 16; m++, p = e) { v[m] = strtod(p, &e); if (e == p) break; }
        if (m != 16) { printf("FAIL row %d: %d numbers after the index, not 16\n", n_rows + 1, m); bad++; continue; }
        /* v: year lat lon alt, then north east down of ppigrf 13, pyIGRF14 13, ppigrf 10, pyIGRF14 10 */
        n_rows++;
        adcs_igrf_gh(v[0], gh);
        for (deg = 0; deg < 2; deg++) {
            adcs_igrf_ned(gh, v[1] * PI / 180.0, v[2] * PI / 180.0, v[3], deg ? 10 : 13, b);
            for (who = 0; who < 2; who++) {
                const double *ref = &v[4 + 6 * deg + 3 * who];
                for (k = 0; k < 3; k++) {
                    double d = fabs(b[k] - ref[k]);
                    if (d > worst[deg]) worst[deg] = d;
                    if (d >= 1.0) { printf("FAIL point %d degree %d %s component %d: %.6f vs %.6f nT\n", n, deg ? 10 : 13, who ? "pyIGRF14" : "ppigrf", k, b[k], ref[k]); bad++; }
                }
            }
            for (k = 0; k < 3; k++) {
                double d = fabs(v[4 + 6 * deg + k] - v[4 + 6 * deg + 3 + k]);
                if (d > spread) spread = d;
            }
        }
    }
    fclose(f);
    if (n_rows != 50) { printf("FAIL the reference has %d points, not 50\n", n_rows); bad++; }
    if (spread >= 0.01) { printf("FAIL the two packages differ by %.6f nT\n", spread); bad++; }
    printf("test_igrf: %d points; worst difference from the references %.6f nT at degree 13, %.6f nT at degree 10 (limit 1 nT); the packages differ by %.6f nT at most (limit 0.01); %s\n",
           n_rows, worst[0], worst[1], spread, bad ? "FAILED" : "ok");
    return bad != 0;
}
