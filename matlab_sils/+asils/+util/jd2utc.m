function utc = jd2utc(jd)
%ASILS.UTIL.JD2UTC  Julian date -> UTC [Y M D h m s] (Fliegel-Van Flandern).
    z = floor(jd + 0.5); f = jd + 0.5 - z;
    a = floor((z - 1867216.25)/36524.25); A = z + 1 + a - floor(a/4);
    B = A + 1524; C = floor((B - 122.1)/365.25); D = floor(365.25*C); E = floor((B - D)/30.6001);
    day = B - D - floor(30.6001*E);
    if E < 14, mon = E - 1; else, mon = E - 13; end
    if mon > 2, yr = C - 4716; else, yr = C - 4715; end
    s = f*86400; h = floor(s/3600); s = s - 3600*h; mi = floor(s/60); s = s - 60*mi;
    utc = [yr mon day h mi s];
end
