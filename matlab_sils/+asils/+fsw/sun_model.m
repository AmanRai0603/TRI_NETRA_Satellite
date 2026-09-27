function s = sun_model(jd)
%ASILS.FSW.SUN_MODEL  Onboard low-precision Sun unit vector, ECI (Vallado alg. 29).
%   Ported from Standard Code env.sun. Deliberately NOT the DE440 truth.
    T = (jd - 2451545.0)/36525;
    L = mod(280.460 + 36000.771*T, 360);
    M = mod(357.5291092 + 35999.05034*T, 360)*pi/180;
    lam = (L + 1.914666471*sin(M) + 0.019994643*sin(2*M))*pi/180;
    eps = (23.439291 - 0.0130042*T)*pi/180;
    s = [cos(lam); cos(eps)*sin(lam); sin(eps)*sin(lam)];
end
