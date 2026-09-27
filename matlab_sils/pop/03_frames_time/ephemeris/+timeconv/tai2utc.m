function jd = tai2utc(jd_tai)
%TAI2UTC  TAI JD -> UTC JD (leap lookup; exact except within a leap-second window).
    jd = jd_tai - timeconv.taiMinusUTC(jd_tai)/86400;
end
