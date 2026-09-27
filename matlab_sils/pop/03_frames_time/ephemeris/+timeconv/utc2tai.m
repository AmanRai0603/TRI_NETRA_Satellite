function jd = utc2tai(jd_utc)
%UTC2TAI  UTC JD -> TAI JD (adds leap seconds).
    jd = jd_utc + timeconv.taiMinusUTC(jd_utc)/86400;
end
