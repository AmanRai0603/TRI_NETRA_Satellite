function [a, comp] = knocke(rSat, rSun, CrAoM, doy, nrings, nseg)
%ERP.KNOCKE  Earth radiation pressure (albedo + IR), Knocke ring model.
%   [a, comp] = erp.knocke(rSat, rSun, CrAoM[, doy][, nrings][, nseg])
%   rSat,rSun ECI [m]; CrAoM = Cr*A/m [m^2/kg] (cannonball optics).
%   Integrates the visible Earth cap: each element reflects sunlight
%   (short-wave, sunlit only) + emits IR (long-wave), Lambertian; the
%   irradiance at the satellite drives radiation pressure along element->sat.
%   comp.sw / comp.lw split the albedo and IR contributions.
    if nargin<4||isempty(doy),   doy=80;  end
    if nargin<5||isempty(nrings),nrings=16;end
    if nargin<6||isempty(nseg),  nseg=48; end
    Kc=de440.constants(); Re=Kc.Re_earth; S=Kc.TSI; c=Kc.c;
    rSat=rSat(:); rSun=rSun(:);
    d=norm(rSat); zhat=rSat/d; shat=rSun/norm(rSun);
    rho_max=acos(min(Re/d,1));
    t=[1;0;0]; if abs(zhat(1))>0.9, t=[0;1;0]; end
    e1=cross(zhat,t); e1=e1/norm(e1); e2=cross(zhat,e1);
    asw=[0;0;0]; alw=[0;0;0];
    for ir=1:nrings
        psi=(ir-0.5)/nrings*rho_max; dpsi=rho_max/nrings;
        for js=1:nseg
            az=2*pi*(js-0.5)/nseg;
            n_el=cos(psi)*zhat+sin(psi)*(cos(az)*e1+sin(az)*e2);
            r_el=Re*n_el; svec=rSat-r_el; rho=norm(svec); es=svec/rho;
            cos_e=dot(n_el,es); if cos_e<=0, continue; end
            dA=Re*Re*sin(psi)*dpsi*(2*pi/nseg);
            lat=asin(max(min(n_el(3),1),-1));
            [alb,emi]=erp.zonalCoeffs(lat,doy);
            cz=dot(n_el,shat);
            Msw=0; if cz>0, Msw=alb*S*cz; end
            Mlw=emi*S/4;
            base=CrAoM/c*cos_e*dA/(pi*rho*rho);
            asw=asw+base*Msw*es; alw=alw+base*Mlw*es;
        end
    end
    comp.sw=asw; comp.lw=alw; a=asw+alw;
end
