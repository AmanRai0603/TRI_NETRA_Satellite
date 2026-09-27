function beta = doodson(jd_tt)
%OCEANTIDES.DOODSON  Six Doodson fundamental arguments [rad] at TT Julian date.
%   beta = [tau, s, h, p, Nprime, ps]:
%     tau  mean lunar time, s Moon mean longitude, h Sun mean longitude,
%     p lunar perigee, N' = -node, ps solar perigee. (IERS / Simon et al. 1994)
    T=(jd_tt-2451545.0)/36525.0; d2r=pi/180;
    s  = mod(218.3164477 + 481267.88123421*T, 360);
    h  = mod(280.4664567 +  36000.76982779*T, 360);
    p  = mod( 83.3532465 +   4069.0137287*T,  360);
    Np = mod(234.9554736 +   1934.1362608*T,  360);   % -Omega (asc node neg)
    ps = mod(282.9373409 +      1.7195366*T,  360);
    % GMST (deg) for tau = GMST + 180 - s
    Tu=T; gmst=mod(280.46061837 + 360.98564736629*(jd_tt-2451545.0),360);
    tau=mod(gmst+180-s,360);
    beta=[tau,s,h,p,Np,ps]*d2r;
end
