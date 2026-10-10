function [acc, x] = node(t, y, W)
%ASILS.ORBIT.NODE  The precision orbit's acceleration at an orbit node plus the context the ADCS needs.
%   x.sun_eci [m]   Earth->Sun (DE440)        x.P_srp [N/m^2]  SRP pressure at 1/r^2
%   x.C [3x3]       ECI->ECEF (GMST build)     x.rho [kg/m^3]   density (the drag's atmosphere)
%   x.moon_eci [m]  Earth->Moon
    [acc, C, E, rho] = asils.orbit.accel(W, t, y(1:3), y(4:6));
    x.t = t; x.C = C;
    if ~isempty(E)
        x.sun_eci = E.sun_eci(:); x.P_srp = E.p_srp; x.moon_eci = E.moon_eci(:);
    else
        [x.sun_eci, x.P_srp, x.moon_eci] = asils.models.forcemodel.no_ephem_env();   % env_force_model's, as adcs-pop's accel
    end
    x.rho = rho;
end
