function [Y, M, D, h, mi, s] = jd2cal(jd)
%JD2CAL  Julian Date -> Gregorian calendar [Y M D h mi s] (scale-agnostic).
    Q = jd + 0.5; Z = floor(Q); F = Q - Z;
    if Z < 2299161, A = Z;
    else, al = floor((Z - 1867216.25)/36524.25); A = Z + 1 + al - floor(al/4); end
    B = A + 1524; C = floor((B - 122.1)/365.25); Dd = floor(365.25*C); E = floor((B - Dd)/30.6001);
    day = B - Dd - floor(30.6001*E) + F;
    if E < 14, M = E - 1; else, M = E - 13; end
    if M > 2,  Y = C - 4716; else, Y = C - 4715; end
    D = floor(day); fr = (day - D)*24; h = floor(fr); fr = (fr - h)*60; mi = floor(fr); s = (fr - mi)*60;
end
