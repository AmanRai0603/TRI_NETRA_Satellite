function [tz, tinf, ro, d, wmm] = dtm3(day, f, fbar, akp, alti, hl, alat, xlon, state)
%DTM3  DTM2020 operational atmosphere model (F10.7 / Kp variant).
%
% Input  : day-of-year, f/fbar (F10.7), Kp array, altitude, local hour angle, lat, lon, coefficient
%          state
% Process: evaluates G(L) via gldtm per constituent -> barometric/diffusive profile from 120 km ->
%          sums species mass densities
% Output : tz, tinf, ro (g/cm^3), d (species), wmm
%
%
% This is the OPERATIONAL model (DTM2020_Oper), driven by F10.7 and Kp.
% Counterpart of dtm5 (the research F30/ap60 model).
%
% INPUTS
%   day    - day of year [1-366]
%   f      - [2x1] f(1)=instantaneous F10.7 at t-24hr, f(2)=0
%   fbar   - [2x1] fbar(1)=81-day mean F10.7, fbar(2)=0
%   akp    - [4x1] geomagnetic Kp array:
%              akp(1) = Kp delayed by 3 hours
%              akp(2) = 0
%              akp(3) = mean Kp of last 24 hours
%              akp(4) = 0
%   alti   - altitude [km], > 120
%   hl     - local solar time [rad], 0-2pi = 0-24hr
%   alat   - geographic latitude [rad]
%   xlon   - geographic longitude [rad]
%   state  - coefficient struct from DTM2020_F107_coeffs_init()
%
% OUTPUTS
%   tz     - temperature at altitude [K]
%   tinf   - exospheric temperature [K]
%   ro     - total density [g/cm^3]
%   d      - [6x1] partial densities [g/cm^3]: H, He, O, N2, O2, N
%   wmm    - mean molecular mass [amu]  (despite Fortran header saying "gram")

% ---- constants ---------------------------------------------------------
re    = 6356.77;       gsurf = 980.665;   rgas = 831.4;
zlb0  = 120.0;
% magnetic pole (dipole approx) constants from the Fortran source
cpmg  = 0.19081;   spmg = 0.98163;   xlmg = -1.2392;

ma    = [1, 4, 16, 28, 32, 14];
alefa = [-0.40, -0.38, 0, 0, 0, 0];
vma   = [1.6606e-24, 6.6423e-24, 26.569e-24, ...
         46.4958e-24, 53.1381e-24, 23.2479e-24];

zlb = zlb0;

% ---- Legendre polynomials (geographic) ---------------------------------
c  = sin(alat);   c2 = c^2;   c4 = c2^2;
s  = cos(alat);   s2 = s^2;

plg.p10 = c;
plg.p20 = 1.5*c2 - 0.5;
plg.p30 = c*(2.5*c2 - 1.5);
plg.p40 = 4.375*c4 - 3.75*c2 + 0.375;
plg.p50 = c*(7.875*c4 - 8.75*c2 + 1.875);
plg.p60 = (5.5*c*plg.p50 - 2.5*plg.p40) / 3.0;
plg.p11 = s;
plg.p21 = 3.0*c*s;
plg.p31 = s*(7.5*c2 - 1.5);
plg.p41 = c*s*(17.5*c2 - 7.5);
plg.p51 = s*(39.375*c4 - 26.25*c2 + 1.875);
plg.p22 = 3.0*s2;
plg.p32 = 15.0*c*s2;
plg.p42 = s2*(52.5*c2 - 7.5);
plg.p52 = 3.0*c*plg.p42 - 2.0*plg.p32;
plg.p62 = 2.75*c*plg.p52 - 1.75*plg.p42;
plg.p33 = 15.0*s*s2;

% ---- geomagnetic latitude (DIPOLE approximation, computed inline) -------
% (operational model does NOT use the full geogm conversion)
clmlmg = cos(xlon - xlmg);
sp     = s*cpmg*clmlmg + c*spmg;
cmg    = sp;                 % magnetic-pole projection
cmg2   = cmg^2;   cmg4 = cmg2^2;
plg.p10mg = cmg;
plg.p20mg = 1.5*cmg2 - 0.5;
plg.p40mg = 4.375*cmg4 - 3.75*cmg2 + 0.375;

% ---- local time harmonics ----------------------------------------------
hloc.ch  = cos(hl);
hloc.sh  = sin(hl);
hloc.c2h = hloc.ch^2 - hloc.sh^2;
hloc.s2h = 2.0*hloc.ch*hloc.sh;
hloc.c3h = hloc.c2h*hloc.ch - hloc.s2h*hloc.sh;
hloc.s3h = hloc.s2h*hloc.ch + hloc.c2h*hloc.sh;

% ---- temperature profiles ----------------------------------------------
[~, gdelt]  = gldtm(f, fbar, akp, day, state.tt, plg, hloc, xlon, 1.0);
tinf   = state.tt(1) * (1.0 + gdelt);

[~, gdelt0] = gldtm(f, fbar, akp, day, state.t0, plg, hloc, xlon, 1.0);
t120   = state.t0(1) * (1.0 + gdelt0);

[~, gdeltp] = gldtm(f, fbar, akp, day, state.tp, plg, hloc, xlon, 1.0);
tp120  = state.tp(1) * (1.0 + gdeltp);

% ---- Bates profile -----------------------------------------------------
sigma   = tp120 / (tinf - t120);
dzeta   = (re + zlb) / (re + alti);
zeta    = (alti - zlb) * dzeta;
sigzeta = sigma * zeta;
expsz   = exp(-sigzeta);
tz      = tinf - (tinf - t120)*expsz;

% ---- species base densities --------------------------------------------
specnames = {'h','he','o','az2','o2','az'};
ff0vals   = [0, 0, 1, 1, 1, 1];

dbase = zeros(6,1);
for i = 1:6
    [~, gd] = gldtm(f, fbar, akp, day, state.(specnames{i}), plg, hloc, xlon, ff0vals(i));
    dbase(i) = state.(specnames{i})(1) * exp(gd);
end

% ---- altitude profiles for each species --------------------------------
glb    = gsurf / (1.0 + zlb/re)^2;
glb    = glb / (sigma * rgas * tinf);
t120tz = t120 / tz;

d  = zeros(6,1);
cc = zeros(6,1);
ro = 0.0;
for i = 1:6
    gamma  = ma(i) * glb;
    upapg  = 1.0 + alefa(i) + gamma;
    fz_i   = t120tz^upapg * exp(-sigzeta * gamma);
    cc(i)  = dbase(i) * fz_i;
    d(i)   = cc(i) * vma(i);
    ro     = ro + d(i);
end

wmm = ro / (vma(1) * sum(cc));

end  % dtm3
