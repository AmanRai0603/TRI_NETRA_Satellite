function [lat, lon, alt] = geodetic(r_ecef, Re, f)
%OP.GEODETIC  ECEF Cartesian -> WGS84 geodetic (lat,lon in rad; alt in m).
%   [lat,lon,alt] = op.geodetic(r_ecef[, Re, f])   (defaults WGS84)
%   Bowring's method (a few Newton iterations); adequate for atmosphere lookup.
    K = de440.constants();
    if nargin<2||isempty(Re), Re=K.Re_earth; end
    if nargin<3||isempty(f),  f=K.f_earth; end
    x=r_ecef(1); y=r_ecef(2); z=r_ecef(3);
    e2=f*(2-f);
    lon=atan2(y,x);
    p=hypot(x,y);
    lat=atan2(z, p*(1-e2));                  % initial
    for it=1:5
        sph=sin(lat); N=Re/sqrt(1-e2*sph^2);
        alt=p/cos(lat)-N;
        lat=atan2(z, p*(1-e2*N/(N+alt)));
    end
    sph=sin(lat); N=Re/sqrt(1-e2*sph^2); alt=p/cos(lat)-N;
end
