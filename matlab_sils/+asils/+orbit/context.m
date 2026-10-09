function x = context(O, t)
%ASILS.ORBIT.CONTEXT  Environment context at time t from the bracketing POP nodes.
%   Sun/Moon linear between nodes; density log-linear; ECI->ECEF by rotating
%   node 0's matrix through the Earth rate (exact for the 'gmst' frame build).
    h = O.t1 - O.t0; s = (t - O.t0)/h;
    x.sun_eci  = O.x0.sun_eci + s*(O.x1.sun_eci - O.x0.sun_eci);
    x.moon_eci = O.x0.moon_eci + s*(O.x1.moon_eci - O.x0.moon_eci);
    x.P_srp    = O.x0.P_srp + s*(O.x1.P_srp - O.x0.P_srp);
    if O.x0.rho > 0 && O.x1.rho > 0
        x.rho = exp(log(O.x0.rho) + s*(log(O.x1.rho) - log(O.x0.rho)));
    else
        x.rho = O.x0.rho + s*(O.x1.rho - O.x0.rho);
    end
    th = O.omega_e*(t - O.t0);
    c = cos(th); sn = sin(th);
    x.C = asils.la.mm([c sn 0; -sn c 0; 0 0 1], O.x0.C);   % each element summed left to right (the engine's la mm)
end
