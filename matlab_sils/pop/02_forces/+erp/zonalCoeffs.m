function [alb, emi] = zonalCoeffs(lat, doy)
%ERP.ZONALCOEFFS  Knocke (1988) 2nd-degree zonal albedo & emissivity.
%   lat [rad] geocentric latitude, doy day-of-year (annual P1 term).
%   a(f)=a0+a1 cos(w(t-t0)) P1(sin f)+a2 P2(sin f); similarly emissivity.
    a0=0.34; a1=0.10; a2=0.29;      % albedo (visible)   (Knocke 1988 nominal)
    e0=0.68; e1=-0.07; e2=-0.18;    % emissivity (IR)
    w=2*pi/365.25; t0=0;            % annual phase (t0 ~ periapsis of season)
    s=sin(lat); P1=s; P2=0.5*(3*s*s-1);
    ann=cos(w*(doy-t0));
    alb=a0+a1*ann*P1+a2*P2;
    emi=e0+e1*ann*P1+e2*P2;
end
