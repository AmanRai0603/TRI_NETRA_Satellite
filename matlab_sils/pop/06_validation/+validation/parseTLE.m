function tle = parseTLE(l1, l2, name)
%VALIDATION.PARSETLE  Parse a two-line element set into a struct.
%   tle = validation.parseTLE(line1, line2[, name])
%   Returns .name .satnum .epoch([Y Mo D H Mi S] UTC) .inc .raan .ecc .argp
%   .M (mean anomaly) .n (mean motion rev/day) .bstar, all angles in radians.
    if nargin<3, name=''; end
    tle.name = strtrim(name);
    tle.satnum = str2double(l1(3:7));
    yy = str2double(l1(19:20)); doy = str2double(l1(21:32));
    yr = 2000+yy; if yy>56, yr=1900+yy; end
    jan0 = datenum(yr,1,0);
    tle.epoch = datevec(jan0 + doy);
    % bstar (assumed decimal point, exponent)
    bs = l1(54:61); m=str2double([bs(1:6)])*1e-5; e=str2double(bs(7:8));
    tle.bstar = m*10^e;
    tle.inc  = deg2rad(str2double(l2(9:16)));
    tle.raan = deg2rad(str2double(l2(18:25)));
    tle.ecc  = str2double(['0.' strtrim(l2(27:33))]);
    tle.argp = deg2rad(str2double(l2(35:42)));
    tle.M    = deg2rad(str2double(l2(44:51)));
    tle.n    = str2double(l2(53:63));                 % rev/day
end
