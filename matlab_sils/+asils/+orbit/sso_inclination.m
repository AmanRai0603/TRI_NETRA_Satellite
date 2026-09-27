function inc_deg = sso_inclination(alt_km, ecc)
%ASILS.ORBIT.SSO_INCLINATION  Sun-synchronous inclination [deg] for a circular/near-circular orbit.
%   cos i = -2 a^(7/2) (1-e^2)^2 dOmega_ss / (3 J2 Re^2 sqrt(mu)),  dOmega_ss = 360 deg / tropical year
    if nargin < 2, ecc = 0; end
    mu = 3.986004418e14; Re = 6378137.0; J2 = 1.08262668e-3;
    a = Re + alt_km*1e3;
    dOm = 2*pi/(365.2421897*86400);
    c = -2*a^3.5*(1 - ecc^2)^2*dOm/(3*J2*Re^2*sqrt(mu));
    inc_deg = acosd(c);
end
