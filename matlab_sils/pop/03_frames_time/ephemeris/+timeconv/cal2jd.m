function jd = cal2jd(Y, M, D, h, mi, s)
%CAL2JD  Gregorian calendar -> Julian Date (scale-agnostic; all Gregorian dates).
%   jd = timeconv.cal2jd(Y,M,D[,h,mi,s]).  Time-of-day defaults to 00:00:00.
    if nargin < 4, h  = 0; end
    if nargin < 5, mi = 0; end
    if nargin < 6, s  = 0; end
    a = floor((14 - M)/12); y = Y + 4800 - a; m = M + 12*a - 3;
    JDN = D + floor((153*m + 2)/5) + 365*y + floor(y/4) - floor(y/100) + floor(y/400) - 32045;
    jd  = JDN + (h - 12)/24 + mi/1440 + s/86400;
end
