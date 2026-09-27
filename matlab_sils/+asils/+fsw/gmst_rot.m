function C = gmst_rot(jd)
%ASILS.FSW.GMST_ROT  Onboard ECI->ECEF rotation from GMST (IAU-82), UT1 ~ UTC.
    T = (jd - 2451545.0)/36525;
    g = mod(67310.54841 + (876600*3600 + 8640184.812866)*T + 0.093104*T^2 - 6.2e-6*T^3, 86400)/240*pi/180;
    C = [cos(g) sin(g) 0; -sin(g) cos(g) 0; 0 0 1];
end
