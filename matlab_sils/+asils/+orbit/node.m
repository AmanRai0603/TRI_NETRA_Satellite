function [acc, x] = node(t, y, W)
%ASILS.ORBIT.NODE  POP acceleration at an orbit node plus the context the ADCS needs.
%   x.sun_eci [m]   Earth->Sun (DE440)        x.P_srp [N/m^2]  SRP pressure at 1/r^2
%   x.C [3x3]       ECI->ECEF (POP frame)      x.rho [kg/m^3]   density (POP drag model)
%   x.moon_eci [m]  Earth->Moon                x.parts          each force's acceleration
    [acc, parts, ctx, info] = op.accel(t, y(1:3), y(4:6), W);
    x.t = t; x.C = ctx.C;
    if ~isempty(ctx.E)
        x.sun_eci = ctx.E.sun_eci(:); x.P_srp = ctx.E.P_srp; x.moon_eci = ctx.E.moon_eci(:);
    else
        [x.sun_eci, x.P_srp, x.moon_eci] = asils.models.forcemodel.no_ephem_env();   % env_force_model's, as adcs-pop's accel
    end
    if isfield(info, 'drag') && isfield(info.drag, 'atm')
        x.rho = info.drag.atm.rho;
    else
        x.rho = 0;
    end
    x.parts = parts;
end
