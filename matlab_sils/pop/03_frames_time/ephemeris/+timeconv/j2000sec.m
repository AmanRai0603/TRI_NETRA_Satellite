function s = j2000sec(jd_tt)
%J2000SEC  TT JD -> seconds past J2000 TT epoch (common propagator time base).
    s = (jd_tt - 2451545.0)*86400.0;
end
