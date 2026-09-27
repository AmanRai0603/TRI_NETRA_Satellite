function g = gmst82(jd_ut1)
%GMST82  Greenwich Mean Sidereal Time (IAU 1982), radians, from UT1 JD.
%   Convenience; your ECI<->ECEF chain may compute its own GMST/ERA.
    Tu  = (jd_ut1 - 2451545.0)/36525.0;
    sec = 67310.54841 + (876600*3600 + 8640184.812866)*Tu + 0.093104*Tu^2 - 6.2e-6*Tu^3;
    g   = deg2rad(mod(sec, 86400)/240.0);   % 240 s per degree
end
