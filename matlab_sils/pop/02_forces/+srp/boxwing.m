function [a, F] = boxwing(P, nu, sunUnit_sat2sun, R_b2i, sc)
%SRP.BOXWING  Physics-based box-wing SRP acceleration.
%   [a, F] = srp.boxwing(P, nu, sunUnit_sat2sun, R_b2i, sc)
%     a  -> acceleration [m/s^2] in ECI
%     F  -> total SRP force  [N]  in ECI (before lighting scaling)
%
%   Sums per-facet absorption + specular + diffuse contributions over all
%   illuminated facets. Solar-array facets pivot to track the Sun.
%
%   INPUTS
%     P               solar pressure at spacecraft [N/m^2]  (ephemInputs.P_srp)
%     nu              lighting fraction 0..1                (srp.eclipse)
%     sunUnit_sat2sun unit vector spacecraft->Sun, ECI
%     R_b2i           body->ECI rotation matrix (3x3)       (from ADCS/attitude)
%     sc              spacecraft struct: sc.mass, sc.facets  (see geometry/*.m)
%
%   Per-facet force (Montenbruck & Gill / McMahon form), a+rho_s+rho_d = 1:
%     F_i = -P*A_i*cos(th)*[ (a+rho_d)*sHat + 2*(rho_s*cos(th)+rho_d/3)*n ]
    sHat_b = R_b2i.' * sunUnit_sat2sun(:);       % Sun direction in body frame
    Fb = [0;0;0];
    for k = 1:numel(sc.facets)
        f = sc.facets(k);
        if strcmp(f.type,'array')
            n   = srp.arrayNormal(f.axis, sHat_b);
            cth = dot(n, sHat_b);
            if f.double && cth < 0, n = -n; cth = -cth; end
        else
            n   = f.n;
            cth = dot(n, sHat_b);
            if f.double && cth < 0, n = -n; cth = -cth; end
        end
        if cth <= 0, continue; end               % facet not illuminated
        Fb = Fb - P*f.A*cth*( (f.alpha + f.rho_d)*sHat_b ...
                              + 2*(f.rho_s*cth + f.rho_d/3)*n );
    end
    F = R_b2i * Fb;                               % force in ECI [N]
    a = nu * F / sc.mass;                         % acceleration in ECI [m/s^2]
end
