function y = decyear(jd)
%ASILS.UTIL.DECYEAR  Decimal year of a Julian date.
    u = asils.util.jd2utc(jd);
    j0 = asils.util.jd([u(1) 1 1 0 0 0]); j1 = asils.util.jd([u(1)+1 1 1 0 0 0]);
    y = u(1) + (jd - j0)/(j1 - j0);
end
