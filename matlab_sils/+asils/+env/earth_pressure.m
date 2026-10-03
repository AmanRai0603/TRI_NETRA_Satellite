function [p_alb, p_ir] = earth_pressure(r_eci, sun_rel_eci, P_sun)
%ASILS.ENV.EARTH_PRESSURE  Pressures [N/m^2] of the Earth's albedo and infrared on
%   a plate facing the Earth's centre at r: the view factor of a sphere to a
%   plate facing it is (Re/r)^2; the albedo falls as the cosine of the Sun's
%   zenith angle under the satellite (0 over the night side). P_sun is the
%   Sun's pressure at the satellite (W/c), so the albedo is a W (Re/r)^2 cos/c.
%   The Earth is a point source in the nadir direction (Knocke et al.'s rings
%   are not modelled). Albedo a = 0.30 and emitted flux 237 W/m^2: the annual
%   global means of the Earth's radiation budget (Kiehl & Trenberth 1997,
%   BAMS 78:197; Knocke, Ries & Tapley 1988, AIAA 88-4292).
%   Engine: adcs-sim-core torques.rs earth_pressure.
    a = 0.30; F_ir = 237.0; c = 299792458.0; Re = 6378137.0;
    rn = sqrt(r_eci'*r_eci);
    vf = (Re/rn)*(Re/rn);
    s = sun_rel_eci + r_eci; s = s/sqrt(s'*s);
    cz = max(0, s'*(r_eci/rn));
    p_alb = a*P_sun*vf*cz;
    p_ir = F_ir/c*vf;
end
