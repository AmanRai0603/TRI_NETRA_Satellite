function [tz, tinf, ro, d, wmm] = dtm5(day, f, fbar, ap60, alti, hl, alat, xlon, state)
%DTM5  DTM2020 atmosphere model (F30/ap60 variant)
%
% Input  : day-of-year, f/fbar (F30 on F10.7 scale), ap60, altitude, local hour angle, lat, lon,
%          coefficient state
% Process: evaluates G(L) via gldtm_Hp per constituent -> barometric/diffusive equilibrium profile
%          from 120 km -> sums species mass densities
% Output : tz (T at z), tinf, ro (total density g/cm^3), d (species), wmm (mean molecular mass)
%
%
% INPUTS
%   day    - day of year [1-366]
%   f      - [2x1] f(1)=instantaneous F30 flux at t-24hr, f(2)=0
%   fbar   - [2x1] mean F30 flux of last 81 days, fbar(2)=0
%   ap60   - [10x1] ap60 geomagnetic index array:
%              (1) 4hr delayed   (2) 0hr   (3) 1hr   (4) 2hr   (5) 3hr
%              (6) mean 24hr     (7) mean 5-6-7hr delayed
%              (8) mean 9-10-11hr (9) mean 14-15-16hr (10) mean 19-20-21hr
%   alti   - altitude [km], must be > 120
%   hl     - local solar time [rad], 0-2pi corresponds to 0-24hr
%   alat   - geographic latitude [rad]
%   xlon   - geographic longitude [rad]
%   state  - struct from dtm2020_load() containing model coefficients
%
% OUTPUTS
%   tz     - temperature at altitude alti [K]
%   tinf   - exospheric temperature [K]
%   ro     - total density [g/cm^3]
%   d      - [6x1] partial densities [g/cm^3]:
%              d(1)=H, d(2)=He, d(3)=O, d(4)=N2, d(5)=O2, d(6)=N
%   wmm    - mean molecular mass [g]

% ---- constants ---------------------------------------------------------
nlatm  = 96;
re     = 6356.77;       % Earth radius [km]
rgas   = 831.4;         % gas constant [erg/(K·mol)]
gsurf  = 980.665;       % surface gravity [cm/s^2]
zlb0   = 120.0;         % base altitude [km]

ma    = [1, 4, 16, 28, 32, 14];          % atomic masses
alefa = [-0.40, -0.38, 0, 0, 0, 0];      % thermal diffusion coefficients
vma   = [1.6606e-24, 6.6423e-24, 26.569e-24, ...
         46.4958e-24, 53.1381e-24, 23.2479e-24];  % mass per particle [g]

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

% ---- geographic → geomagnetic ------------------------------------------
crd   = 180.0 / pi;
dlat  = alat * crd;
dlon  = xlon * crd;
[gmlatd, ~] = geogm(dlat, dlon);
gmlatr = gmlatd * (pi/180.0);

cm  = sin(gmlatr);   cm2 = cm^2;   cm4 = cm2^2;
sm  = cos(gmlatr);   sm2 = sm^2;

plg.p10mg = cm;
plg.p20mg = 1.5*cm2 - 0.5;
plg.p30mg = cm*(2.5*cm2 - 1.5);
plg.p40mg = 4.375*cm4 - 3.75*cm2 + 0.375;
plg.p50mg = cm*(7.875*cm4 - 8.75*cm2 + 1.875);
plg.p60mg = (5.5*cm*plg.p50mg - 2.5*plg.p40mg) / 3.0;
plg.p11mg = sm;
plg.p22mg = 3.0*sm2;
plg.p31mg = sm*(7.5*cm2 - 1.5);

% ---- local time harmonics ----------------------------------------------
hloc.hl0 = hl;
hloc.ch  = cos(hl);
hloc.sh  = sin(hl);
hloc.c2h = hloc.ch^2 - hloc.sh^2;
hloc.s2h = 2.0*hloc.ch*hloc.sh;
hloc.c3h = hloc.c2h*hloc.ch - hloc.s2h*hloc.sh;
hloc.s3h = hloc.s2h*hloc.ch + hloc.c2h*hloc.sh;

% ---- ap60 → Kp (open-ended) -------------------------------------------
latabs = floor(abs(dlat));

if latabs >= 70
    xl75   = (90 - latabs) / 20.0;
    ap_eff = (1-xl75)*((ap60(3)+ap60(2)+ap60(4))/3) + ...
              xl75  *((ap60(3)+ap60(4)+ap60(5))/3);
elseif latabs >= 30
    xl45   = (69 - latabs) / 40.0;
    ap_eff = (1-xl45)*((ap60(3)+ap60(4)+ap60(5))/3) + ...
              xl45  *((ap60(5)+ap60(4)+ap60(1))/3);
else
    ap_eff = (ap60(4)+ap60(5)+ap60(1)) / 3.0;
end

akp    = zeros(8,1);
akp(1) = bint_oe(ap_eff);
akp(2) = bint_oe(ap60(2)) - bint_oe(ap60(3));
akp(3) = bint_oe(ap60(6));
akp(4) = 0.0;
akp(5) = bint_oe(ap60(7));
akp(6) = bint_oe(ap60(8));
akp(7) = bint_oe(ap60(9));
akp(8) = bint_oe(ap60(10));

% ---- temperature profiles ----------------------------------------------
[~, gdelt]  = gldtm_Hp(f, fbar, akp, day, state.tt,  plg, hloc, xlon, 1.0);
tinf   = state.tt(1)  * (1.0 + gdelt);

[~, gdelt0] = gldtm_Hp(f, fbar, akp, day, state.t0,  plg, hloc, xlon, 1.0);
t120   = state.t0(1)  * (1.0 + gdelt0);

[~, gdeltp] = gldtm_Hp(f, fbar, akp, day, state.tp,  plg, hloc, xlon, 1.0);
tp120  = state.tp(1)  * (1.0 + gdeltp);

% ---- Bates profile -----------------------------------------------------
sigma   = tp120 / (tinf - t120);
dzeta   = (re + zlb) / (re + alti);
zeta    = (alti - zlb) * dzeta;
sigzeta = sigma * zeta;
expsz   = exp(-sigzeta);
tz      = tinf - (tinf - t120)*expsz;

% ---- species base densities --------------------------------------------
specnames = {'h','he','o','az2','o2','az'};
ff0vals   = [0, 0, 1, 1, 1, 1];    % flag: density (0) or number density (1)

dbase = zeros(6,1);
gdels = zeros(6,1);
for i = 1:6
    [~, gdels(i)] = gldtm_Hp(f, fbar, akp, day, state.(specnames{i}), plg, hloc, xlon, ff0vals(i));
    dbase(i) = state.(specnames{i})(1) * exp(gdels(i));
end

% ---- altitude profiles for each species --------------------------------
glb   = gsurf / (1.0 + zlb/re)^2;
glb   = glb / (sigma * rgas * tinf);

t120tz = t120 / tz;

d  = zeros(6,1);
cc = zeros(6,1);
ro = 0.0;

for i = 1:6
    gamma   = ma(i) * glb;
    upapg   = 1.0 + alefa(i) + gamma;
    fz_i    = t120tz^upapg * exp(-sigzeta * gamma);
    cc(i)   = dbase(i) * fz_i;
    d(i)    = cc(i) * vma(i);
    ro      = ro + d(i);
end

wmm = ro / (vma(1) * sum(cc));

end  % dtm5
