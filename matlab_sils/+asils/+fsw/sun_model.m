function s = sun_model(jd)
%ASILS.FSW.SUN_MODEL  Onboard low-precision Sun unit vector, J2000 (GCRF) frame.
%   Vallado alg. 29 (ported from Standard Code env.sun) gives the Sun in the
%   mean equator and equinox OF DATE; precessed back to J2000 with the IAU 1976
%   angles (zeta, z, theta), as the rest of the loop works in J2000. Without
%   it the reference is off by the precession since 2000 -- 0.37 deg in 2027,
%   which the SILS found as the floor of Sun referencing. Deliberately not the
%   DE440 truth: the remaining error is the almanac's (~0.01 deg).
    T = (jd - 2451545.0)/36525;
    L = mod(280.460 + 36000.771*T, 360);
    M = mod(357.5291092 + 35999.05034*T, 360)*pi/180;
    lam = (L + 1.914666471*sin(M) + 0.019994643*sin(2*M))*pi/180;
    eps = (23.439291 - 0.0130042*T)*pi/180;
    s_mod = [cos(lam); cos(eps)*sin(lam); sin(eps)*sin(lam)];
    as = pi/(180*3600);
    zeta = (2306.2181*T + 0.30188*T^2)*as; z = (2306.2181*T + 1.09468*T^2)*as; th = (2004.3109*T - 0.42665*T^2)*as;
    R3 = @(a) [cos(a) sin(a) 0; -sin(a) cos(a) 0; 0 0 1];
    R2 = @(a) [cos(a) 0 -sin(a); 0 1 0; sin(a) 0 cos(a)];
    s = R3(zeta)*R2(-th)*R3(z)*s_mod;          % MOD -> J2000 (inverse of P = R3(-z) R2(th) R3(-zeta))
    s = s/norm(s);
end
