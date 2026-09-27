function E = ephemInputs(jdTDB, eph)
%EPHEMINPUTS  All ephemeris-derived quantities the VLEO force/torque stack needs,
%   packaged and labelled by consumer. This is the single interface point:
%   every force/torque model pulls its Sun/Moon inputs from this struct.
%
%   Units : SI (m, m/s, N/m^2, m^3/s^2)
%   Frame : ICRF (== GCRF/ECI to < 1 mas)
%   Time  : jdTDB (TDB Julian date; TT acceptable, error < 1.7 ms)
%
%   NOTE on density models: JB2008 / DTM2020 already take their own Sun input
%   (RA/Dec, local solar time). E.sun_unit / E.sun_ra_dec are provided here so
%   you *can* unify onto DE440, but by design this tool does not modify your
%   existing density feed unless you choose to wire it in.
    if nargin < 2 || isempty(eph); eph = de440.open(); end
    c = eph.const;
    [rs, vs] = de440.sun(jdTDB, eph);
    [rm, vm] = de440.moon(jdTDB, eph);
    d = norm(rs);

    E.jd_tdb = jdTDB;

    % ---- Solar radiation pressure + eclipse (box-wing) ----
    E.sun_unit   = rs / d;                 % Earth->Sun unit vector
    E.sun_dist   = d;                      % m
    E.flux_scale = (c.AU_m / d)^2;         % 1/r^2 flux factor (x P0)
    E.P_srp      = c.P0 * E.flux_scale;    % N/m^2 at spacecraft heliocentric distance

    % ---- Third-body point-mass gravity ----
    E.sun_eci  = rs;  E.sun_vel  = vs;     % m, m/s
    E.moon_eci = rm;  E.moon_vel = vm;
    E.GM_sun   = c.GM_sun;
    E.GM_moon  = c.GM_moon;

    % ---- Solid Earth + ocean tides (Sun & Moon positions) ----
    E.tide_sun  = rs;
    E.tide_moon = rm;

    % ---- Albedo / IR (Sun direction chooses lit Earth elements) ----
    E.albedo_sun_unit = E.sun_unit;

    % ---- Relativity: de Sitter needs Earth heliocentric state ----
    E.earth_helio_pos = -rs;               % Earth w.r.t. Sun = -(Sun w.r.t. Earth)
    E.earth_helio_vel = -vs;

    % ---- Optional density-model hook (RA/Dec of Sun in ICRF) ----
    E.sun_ra  = atan2(rs(2), rs(1));       % rad
    E.sun_dec = asin(rs(3)/d);             % rad

    E.const = c;
end
