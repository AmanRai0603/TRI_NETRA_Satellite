function [a, comp] = simple(rSat, rSun, CrAoM, doy)
%ERP.SIMPLE  Fast radial approximation: whole visible Earth as one source,
%   albedo + IR pushing the satellite along the zenith (anti-nadir).
%   Crude (no geometry integration) - use for quick sizing only.
    if nargin<4||isempty(doy), doy=80; end
    Kc=de440.constants(); Re=Kc.Re_earth; S=Kc.TSI; c=Kc.c;
    rSat=rSat(:); rSun=rSun(:); d=norm(rSat); zhat=rSat/d;
    [alb,emi]=erp.zonalCoeffs(0,doy);              % equatorial coeffs
    f=(Re/d)^2;                                    % Earth solid-angle factor
    cz=max(dot(zhat,rSun/norm(rSun)),0);           % crude lit fraction
    Esw=alb*S*cz*f; Elw=emi*(S/4)*f;
    comp.sw=CrAoM/c*Esw*zhat; comp.lw=CrAoM/c*Elw*zhat; a=comp.sw+comp.lw;
end
