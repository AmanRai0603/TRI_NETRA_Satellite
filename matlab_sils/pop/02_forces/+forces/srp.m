function a = srp(ctx)
%FORCES.SRP  Solar radiation pressure acceleration (ECI), with eclipse.
%   Reads ctx.cfg.forces.srp:
%     .model   'cannonball' | 'boxwing'
%     .eclipse 'cylindrical' | 'conical' | 'fine'
%     .Cr      radiation-pressure coeff (cannonball)
%   Uses ctx.E.P_srp (solar pressure at s/c) and ctx.E.sun_eci.  Spacecraft
%   mass/area/facets/optics come from ctx.sc.
    c  = ctx.cfg.forces.srp;
    sc = ctx.sc;
    rSun = ctx.E.sun_eci;
    sat2sun = rSun - ctx.r_eci;  shat = sat2sun/norm(sat2sun);
    nu = srp.eclipse(ctx.r_eci, rSun, getf(c,'eclipse','conical'));
    switch lower(getf(c,'model','cannonball'))
        case 'cannonball'
            % Cr PRECEDENCE: forces.srp.Cr -> spacecraft.Cr -> 1.3.
            % Same fault as the Cd one in forces/drag.m: this read ONLY
            % cfg.forces.srp.Cr and fell back to a hardcoded 1.3, while every
            % script writes the real value into cfg.spacecraft.Cr. Three scripts
            % carry the line "cfg.forces.srp.Cr = cfg.spacecraft.Cr" purely to
            % paper over it -- which is the tell that the default was wrong, and
            % which does nothing for anyone who forgets it.
            a = srp.cannonball(ctx.E.P_srp, nu, shat, getf(c,'Cr',getf(sc,'Cr',1.3)), sc.Aref/sc.mass);
        case 'boxwing'
            R_b2i = getf(sc,'R_bi',eye(3));
            a = srp.boxwing(ctx.E.P_srp, nu, shat, R_b2i, sc);
        otherwise
            error('forces:srp','unknown srp model "%s"',c.model);
    end
end
function val=getf(s,f,d), if isfield(s,f)&&~isempty(s.(f)), val=s.(f); else, val=d; end, end
