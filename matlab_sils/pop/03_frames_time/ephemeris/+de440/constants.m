function C = constants()
%DE440.CONSTANTS  Physical constants (SI) consistent with DE440 / IAU 2009.
%   Override any field downstream if your force models use different values.
    C.AU_m     = 149597870700.0;          % m (IAU 2012 definition)
    C.c        = 299792458.0;             % m/s
    C.GM_sun   = 1.32712440041279419e20;  % m^3/s^2 (DE440)
    C.GM_earth = 3.98600435507e14;        % m^3/s^2 (DE440, ephemeris-consistent; ~1.6e-8 rel from mu_earth)
    C.GM_moon  = 4.902800118e12;          % m^3/s^2 (DE440)
    C.EMRAT    = 81.3005682214972154;     % Earth/Moon mass ratio (DE440)
    C.TSI      = 1361.0;                   % W/m^2 mean total solar irradiance at 1 AU
    C.P0       = C.TSI / C.c;              % N/m^2 solar radiation pressure at 1 AU
    % --- Earth geodetic (WGS84) & solar radius, single source for the whole app ---
    C.Re_earth = 6378137.0;               % WGS84 equatorial radius [m]
    C.f_earth  = 1/298.257223563;         % WGS84 flattening
    C.Rp_earth = C.Re_earth*(1 - C.f_earth); % polar radius [m] = 6356752.314245
    C.mu_earth = 3.986004418e14;          % WGS84 GM [m^3/s^2] -- use for PROPAGATION
    C.omega_earth = 7.2921150e-5;         % rad/s  Earth mean rotation rate (IERS/WGS84 nominal)
    C.N_A      = 6.02214076e23;           % 1/mol  Avogadro constant (SI 2019)
    C.Rsun     = 6.957e8;                 % IAU 2015 nominal solar radius [m]
end
