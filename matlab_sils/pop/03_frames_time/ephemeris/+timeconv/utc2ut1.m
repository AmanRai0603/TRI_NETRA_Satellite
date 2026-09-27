function jd = utc2ut1(jd_utc, dUT1)
%UTC2UT1  UTC JD -> UT1 JD.  dUT1 (= UT1-UTC, seconds) comes from your EOP feed.
    if nargin < 2 || isempty(dUT1), dUT1 = 0; end
    jd = jd_utc + dUT1/86400;
end
