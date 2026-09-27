function jd = tt2tdb(jd_tt)
%TT2TDB  TT JD -> TDB JD (periodic model, ~microsecond accuracy).
    g  = deg2rad(357.53 + 0.9856003*(jd_tt - 2451545.0));   % Sun mean anomaly
    jd = jd_tt + (0.001658*sin(g) + 0.000014*sin(2*g))/86400;
end
