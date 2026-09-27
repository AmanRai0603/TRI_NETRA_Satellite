function rho = jb2008_density(t, lon_deg, lat_deg, alt_km, idx, varargin)
%JB2008_DENSITY  Total mass density [kg/m^3] along a track from the JB2008 model.
%
% Input  : t (datetime vector), lon, lat [deg], alt [km], idx struct from get_jb2008_indices
% Process: computes Sun geometry per epoch -> looks up daily solar indices (t-24h/t-120h lags per
%          spec) and hourly DSTDTC -> calls JB2008 per point
% Output : rho vector [kg/m^3]
%
%
%   REFERENCE: Bowman et al. (2008), JB2008 empirical thermosphere (AIAA 2008-6438).
%   FORMULA: exospheric temperature T_inf from solar indices (F10,S10,M10,Y10 and their 81-day means)
%       + a Dst-driven correction dTc; density from the JB2008 temperature/composition integration.
%   PSEUDOCODE:
%       for each track point: lag SET indices to the point's epoch; compute Sun RA/Dec, sat RA=lon+GMST,
%       Dst correction; call the JB2008 core(MJD, sunRA,sunDec, satRA,lat, alt, indices, dTc) -> rho
%
%   Thin wrapper around the Jacchia-Bowman 2008 CORE function (the one from the
%   MATLAB File Exchange / SET, e.g. Mahooti's JB2008.m). This routine supplies
%   everything the core needs at each point: the lagged SET solar indices, the
%   Dst temperature correction, the Sun's right ascension/declination, and the
%   satellite right ascension (longitude + GMST).
%
%   REQUIRES on the MATLAB path a core function with the signature
%       [TEMP, RHO] = JB2008(MJD, SUN, SAT, F10,F10B,S10,S10B,XM10,XM10B,Y10,Y10B,DSTDTC)
%   where  SUN = [ra_sun; dec_sun]  (rad),
%          SAT = [ra_sat; gclat; alt_km]  (rad, rad, km),
%          MJD = modified Julian date (UTC).
%   (This is the common convention; if the local File-Exchange version differs, adjust
%    the JB2008(...) call at the bottom — units/order are the only thing to match.)
%
%   INPUTS
%     t        - datetime vector (UTC), the satellite epochs
%     lon_deg  - geodetic/geocentric longitude [deg]   (same length as t)
%     lat_deg  - latitude [deg]                          (geocentric preferred)
%     alt_km   - altitude [km]
%     idx      - struct from get_jb2008_indices (.sol daily, .dtc hourly)
%   OPTIONS
%     'evalStepSec' - evaluate the (scalar) core every N seconds and interpolate
%                     to full cadence (default 60; density is smooth at 10 s, this
%                     is ~100x faster than calling the core at every sample).
%                     Set to the data cadence to evaluate every point.
%
%   OUTPUT  rho - density [kg/m^3], same length as t.
%
%   Lag convention (Bowman et al. 2008): F10 & S10 lag 1 day, M10 lag 2 days,
%   Y10 lag 5 days. DSTDTC is taken at the epoch (hourly, interpolated).

ip = inputParser; ip.addParameter('evalStepSec',60,@isnumeric); ip.parse(varargin{:});
stepSec = ip.Results.evalStepSec;

if exist('JB2008','file')~=2
    error('jb2008_density:noCore', ...
        ['JB2008 core function not found on the path. Get it from the MATLAB File\n' ...
         'Exchange (Jacchia-Bowman 2008) or SET, and make sure JB2008.m is on the path.']);
end

t = t(:); lon_deg = lon_deg(:); lat_deg = lat_deg(:); alt_km = alt_km(:);
N = numel(t);

% --- decimate: evaluate the scalar core on a coarse grid, interpolate back ---
cad = median(seconds(diff(t)),'omitnan'); if ~isfinite(cad)||cad<=0, cad = 10; end
stride = max(1, round(stepSec/cad));
e = unique([1:stride:N, N]);                                  % evaluation indices
mjd = mjd_utc(t(e));

% --- resolve lagged daily indices at each evaluation epoch ---
solT = datenum(idx.sol.Time);
day  = mjd + 2400000.5 - 2400000.5;                          % (mjd) -> use datenum of the epoch's day
dnum = datenum(t(e));
F10 = interp1(solT, idx.sol.F10, dnum-1, 'linear','extrap');  F10B = interp1(solT, idx.sol.F81, dnum-1,'linear','extrap');
S10 = interp1(solT, idx.sol.S10, dnum-1, 'linear','extrap');  S10B = interp1(solT, idx.sol.S81, dnum-1,'linear','extrap');
M10 = interp1(solT, idx.sol.M10, dnum-2, 'linear','extrap');  M10B = interp1(solT, idx.sol.M81, dnum-2,'linear','extrap');
Y10 = interp1(solT, idx.sol.Y10, dnum-5, 'linear','extrap');  Y10B = interp1(solT, idx.sol.Y81, dnum-5,'linear','extrap');

% --- DSTDTC at the epoch (hourly, interpolated) ---
dtcT = datenum(idx.dtc.Time);
DSTDTC = interp1(dtcT, idx.dtc.DTC, dnum, 'linear','extrap');

% --- sun geometry + satellite right ascension ---
[raSun, decSun] = sun_radec(mjd);                            % rad
gmst = gmst_rad(mjd);                                        % rad
raSat = mod(deg2rad(lon_deg(e)) + gmst, 2*pi);              % right ascension of sub-sat point
latR  = deg2rad(lat_deg(e));

% --- call the core per evaluation point ---
rhoE = nan(numel(e),1);
for k = 1:numel(e)
    [~, RHO] = JB2008(mjd(k), [raSun(k); decSun(k)], [raSat(k); latR(k); alt_km(e(k))], ...
                      F10(k),F10B(k), S10(k),S10B(k), M10(k),M10B(k), Y10(k),Y10B(k), DSTDTC(k));
    rhoE(k) = RHO;
    if mod(k, max(1,round(numel(e)/10)))==0
        fprintf('   JB2008 %3.0f%%\n', 100*k/numel(e));
    end
end

% --- interpolate back to full cadence (log space: density is log-normal) ---
if numel(e) < N
    rho = exp(interp1(e(:), log(rhoE), (1:N)', 'linear','extrap'));
else
    rho = rhoE;
end
rho = rho(:);
end

% ======================= local astronomy helpers =======================
function m = mjd_utc(t)
    m = datenum(t) - 678942;                                  % datenum -> MJD (datenum of 1858-11-17 = 678942)
end
function [ra, dec] = sun_radec(mjd)
    % low-precision Sun (mean equinox of date), good to ~0.01 deg -> ample for density
    T  = (mjd - 51544.5)/36525;
    L  = mod(280.460 + 36000.771*T, 360);
    M  = deg2rad(mod(357.528 + 35999.050*T, 360));
    lam= deg2rad(L + 1.915*sin(M) + 0.020*sin(2*M));
    eps= deg2rad(23.439 - 0.0130*T);
    ra = mod(atan2(cos(eps).*sin(lam), cos(lam)), 2*pi);
    dec= asin(sin(eps).*sin(lam));
end
function g = gmst_rad(mjd)
    % IAU-82 GMST, radians.  MJD integer = 0h UT (midnight), so frac(mjd) IS the
    % UT fraction of the day.  (Earlier version subtracted 0.5 here, which rotated
    % GMST by ~180 deg and INVERTED the day/night diurnal bulge.)
    Tu    = (floor(mjd) - 51544.5)/36525;                     % centuries J2000 -> 0h UT of the day
    gmst0 = 24110.54841 + 8640184.812866*Tu + 0.093104*Tu.^2 - 6.2e-6*Tu.^3;  % sec at 0h UT
    frac  = mod(mjd, 1);                                       % UT fraction of day
    gsec  = mod(gmst0 + 1.0027379093*86400*frac, 86400);
    g = deg2rad(gsec/240);                                     % 86400 s -> 360 deg  (=> /240)
end
