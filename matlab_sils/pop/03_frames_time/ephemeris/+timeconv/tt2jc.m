function T = tt2jc(jd_tt)
%TT2JC  TT JD -> Julian centuries since J2000 (used by precession/nutation/GMST).
    T = (jd_tt - 2451545.0)/36525.0;
end
