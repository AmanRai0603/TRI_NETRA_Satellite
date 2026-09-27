function jd = tai2gps(jd_tai)
%TAI2GPS  TAI JD -> GPS-time JD (-19 s, constant).
    jd = jd_tai - 19/86400;
end
