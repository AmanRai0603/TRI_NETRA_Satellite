function jd = tai2tt(jd_tai)
%TAI2TT  TAI JD -> TT JD (+32.184 s, exact).
    jd = jd_tai + 32.184/86400;
end
