function [alpha, beta, V, vbody] = windAngles(vrel, R_bi)
%DRAG.WINDANGLES  Angle of attack (alpha) and sideslip (beta) of the relative
%   wind, from the ECI relative velocity and the body->inertial DCM R_bi.
%   Body axes: x=forward(roll), y=right(pitch axis), z=down(yaw). Standard
%   aircraft convention (Stevens & Lewis 2003, Etkin):
%     [u v w] = R_bi' * vrel ;  alpha = atan2(w,u) ;  beta = asin(v/V)
    vrel=vrel(:); vbody = R_bi.'*vrel; u=vbody(1); v=vbody(2); w=vbody(3);
    V = norm(vbody);
    alpha = atan2(w, u);
    beta  = asin(max(min(v/V,1),-1));
end
