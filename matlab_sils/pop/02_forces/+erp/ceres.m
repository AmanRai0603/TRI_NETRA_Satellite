function [a, comp] = ceres(rSat, rSun, CrAoM, gridFcn, nrings, nseg)
%ERP.CERES  Data-driven Earth radiation pressure (advanced).
%   Same ring integration as knocke, but albedo/emissivity come from a
%   user-supplied grid function gridFcn(lat,lon) -> [albedo, emissivity]
%   (e.g. interpolant built from CERES SYN1deg data). Provide your own data;
%   see docs/INPUTS.md for the CERES source. Falls back to Knocke zonal if
%   gridFcn is empty.
    if nargin<5||isempty(nrings), nrings=16; end
    if nargin<6||isempty(nseg),   nseg=48;  end
    Kc=de440.constants(); Re=Kc.Re_earth; S=Kc.TSI; c=Kc.c;
    rSat=rSat(:); rSun=rSun(:); d=norm(rSat); zhat=rSat/d; shat=rSun/norm(rSun);
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
            lat=asin(max(min(n_el(3),1),-1)); lon=atan2(n_el(2),n_el(1));
            if isempty(gridFcn), [alb,emi]=erp.zonalCoeffs(lat,80);
            else, v=gridFcn(lat,lon); alb=v(1); emi=v(2); end
            cz=dot(n_el,shat); Msw=0; if cz>0, Msw=alb*S*cz; end
            Mlw=emi*S/4; base=CrAoM/c*cos_e*dA/(pi*rho*rho);
            asw=asw+base*Msw*es; alw=alw+base*Mlw*es;
        end
    end
    comp.sw=asw; comp.lw=alw; a=asw+alw;
end
