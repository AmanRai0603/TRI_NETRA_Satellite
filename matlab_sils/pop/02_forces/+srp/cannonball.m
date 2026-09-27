function a = cannonball(P, nu, sunUnit_sat2sun, Cr, AoverM)
%SRP.CANNONBALL  Simple isotropic ("cannonball") SRP acceleration.
%   a = srp.cannonball(P, nu, sunUnit_sat2sun, Cr, AoverM)   -> [m/s^2] ECI
%
%   Treats the spacecraft as a sphere: one scalar coefficient, attitude-free.
%   Force is anti-sunward (along -sunUnit).
%
%   INPUTS
%     P               solar pressure at spacecraft [N/m^2]  (ephemInputs.P_srp)
%     nu              lighting fraction 0..1                (srp.eclipse)
%     sunUnit_sat2sun unit vector spacecraft->Sun, ECI      (see note)
%     Cr              radiation-pressure coeff (~1 absorb .. 2 reflect; est. in OD)
%     AoverM          cross-section area / mass [m^2/kg]
%
%   NOTE  Use the spacecraft->Sun vector (rSun - rSat), not Earth->Sun, for the
%   direction; at LEO they differ by ~10 arcsec which is usually negligible but
%   free to do correctly.
    a = -Cr * P * AoverM * nu * sunUnit_sat2sun(:);
end
