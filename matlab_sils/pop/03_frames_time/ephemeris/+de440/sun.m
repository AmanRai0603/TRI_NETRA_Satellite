function [r, v] = sun(jdTDB, eph)
%DE440.SUN  Geocentric Sun position/velocity in ICRF (SI: m, m/s).
%   This is the Earth->Sun vector used for SRP direction, flux 1/r^2 scaling,
%   eclipse geometry, tides and albedo.
    if nargin < 2 || isempty(eph); eph = de440.open(); end
    [rs, vs] = de440.state(0, 10, jdTDB, eph);
    [re, ve] = de440.earth(jdTDB, eph);
    r = (rs - re) * 1000.0;    % m
    v = (vs - ve) * 1000.0;    % m/s
end
