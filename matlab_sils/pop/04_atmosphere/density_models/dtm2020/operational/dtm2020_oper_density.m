function out = dtm2020_oper_density(alt_km, lat_deg, lon_deg, lst_hours, doy, F107, F107_bar, Kp, state)
%DTM2020_OPER_DENSITY  Convenience wrapper for the OPERATIONAL DTM2020 model.
%
% Input  : alt [km], lat, lon [deg], LST [h], DOY, F107, F107_bar, Kp, coefficient state
% Process: builds per-point driver arrays -> calls dtm3 point-by-point
% Output : struct out with rho [kg/m^3], Tinf, Tz and species densities
%
%
%   REFERENCE: DTM2020 OPERATIONAL variant (F10.7 + Kp driven); mirror of the research wrapper.
%   FORMULA: same DTM barometric/spherical-harmonic form as research, evaluated by dtm3 with the
%       F107/Kp coefficient set (DTM2020_F107_coeffs_init).
%   PSEUDOCODE:
%       state <- DTM2020_F107_coeffs_init() if not supplied
%       convert inputs; call dtm3(doy,F107,F107_bar,Kp,alt,...,state); return struct(rho,T,species)
%
% Operational model (DTM2020_Oper) driven by F10.7 + Kp. Mirror of
% dtm2020_density (which uses the research F30/ap60 model).
%
% ---------------------------------------------------------------------
% INPUTS (first 5 mandatory)
%   alt_km    - altitude [km] (>120)
%   lat_deg   - geographic latitude [deg]
%   lon_deg   - geographic longitude [deg]
%   lst_hours - local solar time [hours] 0..24
%   doy       - day of year [1..366]
% OPTIONAL (defaults = moderate activity):
%   F107      - F10.7 flux at t-24h [sfu]      (default 120)
%   F107_bar  - 81-day mean F10.7 [sfu]        (default = F107)
%   Kp        - either a scalar Kp (used for both the 3h-delayed and the
%               24h-mean slots) OR a [4x1] akp array [Kp_3hdelay;0;Kp_24hmean;0]
%               (default 3)
%   state     - coeffs from DTM2020_F107_coeffs_init() (auto-loaded if omitted)
%
% OUTPUT struct 'out' (same fields as dtm2020_density):
%   out.rho_gcm3, out.rho_kgm3, out.T_K, out.Tinf_K
%   out.n.{H,He,O,N2,O2,N}  [1/cm^3]
%   out.rho_species.{...}   [g/cm^3]
%   out.mbar_amu
%
% EXAMPLE
%   st = DTM2020_F107_coeffs_init();
%   o  = dtm2020_oper_density(300, 23, 72, 12, 90, 150, 145, 3, st);
%   fprintf('rho = %.3e kg/m^3, T = %.1f K\n', o.rho_kgm3, o.T_K);

if nargin < 6 || isempty(F107),     F107     = 120;  end
if nargin < 7 || isempty(F107_bar), F107_bar = F107; end
if nargin < 8 || isempty(Kp),       Kp       = 3;    end
if nargin < 9 || isempty(state),    state    = DTM2020_F107_coeffs_init(); end

if alt_km <= 120
    error('dtm2020_oper_density: altitude must be > 120 km (got %.1f).', alt_km);
end

f    = [F107;     0.0];
fbar = [F107_bar; 0.0];

% Build the 4-element akp array
if isscalar(Kp)
    akp = [Kp; 0.0; Kp; 0.0];   % same Kp for 3h-delayed and 24h-mean slots
else
    akp = Kp(:);
    if numel(akp) ~= 4
        error('dtm2020_oper_density: Kp must be a scalar or a 4-element akp array.');
    end
end

hl   = lst_hours / 24.0 * 2*pi;
alat = deg2rad(lat_deg);
xlon = deg2rad(lon_deg);

[tz, tinf, ro, d, wmm] = dtm3(doy, f, fbar, akp, alt_km, hl, alat, xlon, state);

vma = [1.6606e-24, 6.6423e-24, 26.569e-24, 46.4958e-24, 53.1381e-24, 23.2479e-24];
n_cm3 = d(:)' ./ vma;

out.rho_gcm3 = ro;
out.rho_kgm3 = ro * 1000.0;
out.T_K      = tz;
out.Tinf_K   = tinf;
out.n.H  = n_cm3(1);   out.n.He = n_cm3(2);   out.n.O  = n_cm3(3);
out.n.N2 = n_cm3(4);   out.n.O2 = n_cm3(5);   out.n.N  = n_cm3(6);
out.rho_species.H  = d(1);   out.rho_species.He = d(2);   out.rho_species.O  = d(3);
out.rho_species.N2 = d(4);   out.rho_species.O2 = d(5);   out.rho_species.N  = d(6);
out.mbar_amu = wmm;   % already in amu

end
