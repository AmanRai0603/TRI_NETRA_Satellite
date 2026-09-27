function [dut1, dlod] = tidal_ut1_libration(rmjd)
%TIDAL_UT1_LIBRATION  Subdiurnal LIBRATION in UT1 (dut1) and LOD (dlod), in
%  MICROSECONDS (dlod in us/day). Derived-work MATLAB port (IERS Conventions
%  Software License) of IERS UTLIBR.F (Brzezinski). Renamed; NOT IERS software.
%  Validated to ~1e-8 us vs the official tests (rmjd 44239.1 -> 2.44114383,
%  -14.7897125 ; rmjd 55227.4 -> -2.65570584, 27.3944583).
persistent TB
if isempty(TB), TB = [2 -2 0 -2 0 -2 0.5377239 0.05 -0.03 -0.3 -0.6;
    2 0 0 -2 -2 -2 0.5363232 0.06 -0.03 -0.4 -0.7;
    2 -1 0 -2 0 -2 0.5274312 0.35 -0.2 -2.4 -4.1;
    2 1 0 -2 -2 -2 0.5260835 0.07 -0.04 -0.5 -0.8;
    2 0 0 -2 0 -1 0.5175645 -0.07 0.04 0.5 0.8;
    2 0 0 -2 0 -2 0.5175251 1.75 -1.01 -12.2 -21.3;
    2 1 0 -2 0 -2 0.5079842 -0.05 0.03 0.3 0.6;
    2 0 -1 -2 2 -2 0.5006854 0.04 -0.03 -0.3 -0.6;
    2 0 0 -2 2 -2 0.5 0.76 -0.44 -5.5 -9.6;
    2 0 0 0 0 0 0.4986348 0.21 -0.12 -1.5 -2.6;
    2 0 0 0 0 -1 0.4985982 0.06 -0.04 -0.4 -0.8]; end   % [IARG(6) PER DUT1S DUT1C DLODS DLODC], 11 rows
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
IA=TB(:,1:6); US=TB(:,8); UC=TB(:,9); LS=TB(:,10); LC=TB(:,11);
ang=mod(IA*ARG,6.283185307179586476925287);
dut1=sum(US.*sin(ang)+UC.*cos(ang));
dlod=sum(LS.*sin(ang)+LC.*cos(ang));
end
