function x = context(O, t)
%ASILS.ORBIT.CONTEXT  Environment context at time t from the bracketing POP nodes.
%   Sun/Moon linear between nodes; density log-linear; ECI->ECEF turned on from node 0 towards node 1 about its own axis
%   (exact at both nodes, on any frame build).
    h = O.t1 - O.t0; s = (t - O.t0)/h;
    x.sun_eci  = O.x0.sun_eci + s*(O.x1.sun_eci - O.x0.sun_eci);
    x.moon_eci = O.x0.moon_eci + s*(O.x1.moon_eci - O.x0.moon_eci);
    x.P_srp    = O.x0.P_srp + s*(O.x1.P_srp - O.x0.P_srp);
    if O.x0.rho > 0 && O.x1.rho > 0
        x.rho = exp(log(O.x0.rho) + s*(log(O.x1.rho) - log(O.x0.rho)));
    else
        x.rho = O.x0.rho + s*(O.x1.rho - O.x0.rho);
    end
    x.C = frac_rot_(O.x0.C, O.x1.C, s);
end

function C = frac_rot_(c0, c1, s)
% The Earth frame at the fraction s of the step: (C1 C0')^s C0, the rotation between the two nodes' frames taken to the power s
% about its own axis (Rodrigues), so the frame is exact at both nodes (the engine's adcs-pop frac_rot). Turning node 0's frame about
% the z axis of the Earth-fixed frame by the Earth rate times the time is exact only when that axis is the pole the Earth turns
% about; on build A it is the CIP, about 1.5e-6 rad off, and a 10 s step left 1.3e-9 rad.
    d = asils.la.mm(c1, c0');
    v = [d(3,2) - d(2,3); d(1,3) - d(3,1); d(2,1) - d(1,2)];     % 2 sin(theta) n
    sv = sqrt(v(1)*v(1) + v(2)*v(2) + v(3)*v(3));
    if sv == 0, C = c0; return, end
    th = atan2(0.5*sv, 0.5*(d(1,1) + d(2,2) + d(3,3) - 1));
    n = [v(1)/sv; v(2)/sv; v(3)/sv];
    sn = sin(s*th); cs = cos(s*th);
    k = [0, -n(3), n(2); n(3), 0, -n(1); -n(2), n(1), 0];
    r = zeros(3, 3);
    for i = 1:3
        for j = 1:3
            r(i, j) = (i == j)*cs + (1 - cs)*n(i)*n(j) + sn*k(i, j);
        end
    end
    C = asils.la.mm(r, c0);
end
