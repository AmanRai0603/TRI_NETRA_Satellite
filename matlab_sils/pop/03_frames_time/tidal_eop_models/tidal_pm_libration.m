function [dxp, dyp] = tidal_pm_libration(rmjd)
%TIDAL_PM_LIBRATION  Diurnal lunisolar LIBRATION in polar motion (dxp,dyp) in
%  MICROARCSECONDS. Derived-work MATLAB port (IERS Conventions Software License)
%  of IERS PMSDNUT2.F (Brzezinski & Capitaine). Renamed; NOT IERS software.
%  Validated to ~1e-7 uas vs the official test (rmjd 54335 -> 24.8314424,
%  -14.0924069 uas). IBAND=1 (10 diurnal terms).
persistent TB
if isempty(TB), TB = [1 -1 0 -2 0 -1 1.1196992 -0.4 0.3 -0.3 -0.4;
    1 -1 0 -2 0 -2 1.1195149 -2.3 1.3 -1.3 -2.3;
    1 1 0 -2 -2 -2 1.1134606 -0.4 0.3 -0.3 -0.4;
    1 0 0 -2 0 -1 1.0759762 -2.1 1.2 -1.2 -2.1;
    1 0 0 -2 0 -2 1.0758059 -11.4 6.5 -6.5 -11.4;
    1 -1 0 0 0 0 1.0347187 0.8 -0.5 0.5 0.8;
    1 0 0 -2 2 -2 1.0027454 -4.8 2.7 -2.7 -4.8;
    1 0 0 0 0 0 0.9972696 14.3 -8.2 8.2 14.3;
    1 0 0 0 0 -1 0.9971233 1.9 -1.1 1.1 1.9;
    1 1 0 0 0 0 0.9624365 0.8 -0.4 0.4 0.8]; end   % [IARG(6) PER XS XC YS YC], rows 16..25
% --- IERS-2010 Delaunay fundamental arguments (rad) + GMST+pi (rad) ---
    T = (rmjd - 51544.5)/36525.0;
    AS2R=4.848136811095359935899141e-6; TURNAS=1296000.0; TWOPI=6.283185307179586476925287; PI=3.141592653589793238462643;
    L =mod(485868.249036+T.*(1717915923.2178+T.*(31.8792+T.*(0.051635+T.*(-0.00024470)))),TURNAS)*AS2R;
    LP=mod(1287104.793048+T.*(129596581.0481+T.*(-0.5532+T.*(0.000136+T.*(-0.00001149)))),TURNAS)*AS2R;
    F =mod(335779.526232+T.*(1739527262.8478+T.*(-12.7512+T.*(-0.001037+T.*(0.00000417)))),TURNAS)*AS2R;
    D =mod(1072260.703692+T.*(1602961601.2090+T.*(-6.3706+T.*(0.006593+T.*(-0.00003169)))),TURNAS)*AS2R;
    OM=mod(450160.398036+T.*(-6962890.5431+T.*(7.4722+T.*(0.007702+T.*(-0.00005939)))),TURNAS)*AS2R;
    GMST=mod(67310.54841+T.*((8640184.812866+3155760000.0)+T.*(0.093104+T.*(-0.0000062))),86400.0);
    ARG=[mod(GMST/(86400.0/TWOPI)+PI,TWOPI); L; LP; F; D; OM];
IA=TB(:,1:6); XS=TB(:,8); XC=TB(:,9); YS=TB(:,10); YC=TB(:,11);
ang=mod(IA*ARG,6.283185307179586476925287);
dxp=sum(XS.*sin(ang)+XC.*cos(ang));
dyp=sum(YS.*sin(ang)+YC.*cos(ang));
end
