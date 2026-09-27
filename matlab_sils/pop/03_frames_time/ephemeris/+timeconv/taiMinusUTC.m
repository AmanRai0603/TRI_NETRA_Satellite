function n = taiMinusUTC(jd_utc)
%TAIMINUSUTC  Integer leap-second offset TAI-UTC at a UTC Julian date.
    L = timeconv.leapTable(); n = 10;
    for i = 1:size(L,1)
        if jd_utc >= timeconv.cal2jd(L(i,1), L(i,2), 1), n = L(i,3); end
    end
end
