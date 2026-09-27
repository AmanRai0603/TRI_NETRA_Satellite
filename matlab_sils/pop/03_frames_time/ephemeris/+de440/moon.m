function [r, v] = moon(jdTDB, eph)
%DE440.MOON  Geocentric Moon position/velocity in ICRF (SI: m, m/s).
    if nargin < 2 || isempty(eph); eph = de440.open(); end
    [rm, vm]  = de440.state(3, 301, jdTDB, eph);
    [ree,vee] = de440.state(3, 399, jdTDB, eph);
    r = (rm - ree) * 1000.0;   % m
    v = (vm - vee) * 1000.0;   % m/s
end
