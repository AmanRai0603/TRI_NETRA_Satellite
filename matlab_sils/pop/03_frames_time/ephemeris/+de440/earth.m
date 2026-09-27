function [r, v] = earth(jdTDB, eph)
%DE440.EARTH  Geocentre position/velocity w.r.t. Solar System Barycentre (km, km/s).
    if nargin < 2 || isempty(eph); eph = de440.open(); end
    [re, ve]  = de440.state(0, 3,   jdTDB, eph);   % SSB -> Earth-Moon barycentre
    [ree,vee] = de440.state(3, 399, jdTDB, eph);   % EMB -> Earth
    r = re + ree; v = ve + vee;
end
