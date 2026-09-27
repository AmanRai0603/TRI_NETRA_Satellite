function out = dtm2020_density(alt_km, lat_deg, lon_deg, lst_hours, doy, F30, F30_bar, ap60, state)
%DTM2020_DENSITY  Convenience wrapper for the DTM2020 atmosphere model.
%
% Input  : alt [km], lat, lon [deg], LST [h], DOY, F30, F30_bar (F10.7 scale), ap60, coefficient
%          state
% Process: builds per-point driver arrays -> calls dtm5 point-by-point -> collects density,
%          temperatures and composition
% Output : struct out with rho [kg/m^3], Tinf, Tz and species densities
%
%
%   REFERENCE: Bruinsma & Boniface (2021), DTM2020 (research, F30/ap60 driven), SWAMI project.
%   FORMULA (per species i):  n_i(z) = n_i(120) * exp( G_i(local drivers) ) via the DTM barometric
%       profile; total rho = sum_i m_i n_i.  Local term G_i from gldtm_Hp (spherical-harmonic in
%       latitude, local solar time, day-of-year, F30, F30_bar, ap60).
%   PSEUDOCODE:
%       state <- DTM2020_coeffs_init() if not supplied
%       convert (deg,hours,km)->(rad,arrays); call dtm5(doy,F30,F30_bar,ap60,alt,...,state)
%       return struct(rho, T, per-species densities)
%
% Computes neutral density, temperature and composition at a single point,
% with human-friendly inputs (degrees, hours, km) instead of the raw
% radians/arrays the core dtm5() expects.
%
% ---------------------------------------------------------------------
% INPUTS  (only the first 5 are mandatory; the rest have defaults)
% ---------------------------------------------------------------------
%   alt_km    - altitude above Earth [km]              (must be > 120)
%   lat_deg   - geographic latitude  [deg]   -90..+90
%   lon_deg   - geographic longitude [deg]     0..360 (or -180..180)
%   lst_hours - LOCAL SOLAR time     [hours]    0..24
%   doy       - day of year          [1..366]
%
% OPTIONAL (defaults model a moderate-activity day):
%   F30       - F30 solar radio flux at t-24h  [sfu]   (default 120)
%   F30_bar   - 81-day mean F30 flux           [sfu]   (default = F30)
%   ap60      - either a scalar (same value in all 10 slots)
%               or a full [10x1] ap60 array     (default 12)
%   state     - coefficient struct from DTM2020_coeffs_init().
%               If omitted, it is loaded automatically.
%
% NOTE on local time: lst_hours is LOCAL SOLAR time, NOT UTC. For a quick
% approximation:  LST ≈ UTC_hours + lon_deg/15   (wrap into 0..24).
%
% ---------------------------------------------------------------------
% OUTPUT  (struct 'out')
% ---------------------------------------------------------------------
%   out.rho_gcm3   - total mass density [g/cm^3]   (model-native unit)
%   out.rho_kgm3   - total mass density [kg/m^3]   (SI, ×1000)
%   out.T_K        - temperature at altitude [K]
%   out.Tinf_K     - exospheric temperature [K]
%   out.n          - struct of NUMBER densities [1/cm^3]:
%                      .H .He .O .N2 .O2 .N
%   out.rho_species- struct of partial MASS densities [g/cm^3]
%   out.mbar_amu   - mean molecular mass [amu]
%
% ---------------------------------------------------------------------
% EXAMPLE
% ---------------------------------------------------------------------
%   st = DTM2020_coeffs_init();
%   o  = dtm2020_density(300, 23, 72, 12, 90, 120, 115, 12, st);
%   fprintf('rho = %.3e kg/m^3,  T = %.1f K\n', o.rho_kgm3, o.T_K);

% ---- defaults ----------------------------------------------------------
if nargin < 6 || isempty(F30),     F30     = 120;     end
if nargin < 7 || isempty(F30_bar), F30_bar = F30;     end
if nargin < 8 || isempty(ap60),    ap60    = 12;      end
if nargin < 9 || isempty(state),   state   = DTM2020_coeffs_init(); end

% ---- input checks ------------------------------------------------------
if alt_km <= 120
    error('dtm2020_density: altitude must be > 120 km (got %.1f).', alt_km);
end

% ---- pack into core-model format --------------------------------------
% NOTE: f(1)/fbar(1) must be F30 ALREADY RESCALED to the F10.7 scale (see
% f30_to_f107scale.m). DTM2020_Res was fit with F30 rescaled to F10.7 via the
% SWAMI drift-corrected regression, so the CALLER must rescale before calling
% this. (Per SWAMI, F10.7 may also be fed directly.) The benchmark's 80/180
% test values are already on this F10.7 scale.
f    = [F30;     0.0];
fbar = [F30_bar; 0.0];

if isscalar(ap60)
    ap60v = ap60 * ones(10,1);
else
    ap60v = ap60(:);
    if numel(ap60v) ~= 10
        error('dtm2020_density: ap60 must be a scalar or a 10-element array.');
    end
end

hl   = lst_hours / 24.0 * 2*pi;    % hours -> radians
alat = deg2rad(lat_deg);
xlon = deg2rad(lon_deg);

% ---- call the core model ----------------------------------------------
[tz, tinf, ro, d, wmm] = dtm5(doy, f, fbar, ap60v, alt_km, hl, alat, xlon, state);

% ---- particle masses [g] (same order as d): H He O N2 O2 N ------------
vma = [1.6606e-24, 6.6423e-24, 26.569e-24, 46.4958e-24, 53.1381e-24, 23.2479e-24];

% number density [1/cm^3] = mass density / particle mass
n_cm3 = d(:)' ./ vma;

% ---- assemble output ---------------------------------------------------
out.rho_gcm3 = ro;
out.rho_kgm3 = ro * 1000.0;        % g/cm^3 -> kg/m^3
out.T_K      = tz;
out.Tinf_K   = tinf;

out.n.H  = n_cm3(1);   out.n.He = n_cm3(2);   out.n.O  = n_cm3(3);
out.n.N2 = n_cm3(4);   out.n.O2 = n_cm3(5);   out.n.N  = n_cm3(6);

out.rho_species.H  = d(1);   out.rho_species.He = d(2);   out.rho_species.O  = d(3);
out.rho_species.N2 = d(4);   out.rho_species.O2 = d(5);   out.rho_species.N  = d(6);

% NOTE: dtm5's wmm output is ALREADY in amu (the formula divides by the
% amu reference mass vma(1)), despite the Fortran header saying "gram".
out.mbar_amu = wmm;

end
