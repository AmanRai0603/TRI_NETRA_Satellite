function R_bi = attitudeFromAoA(vrel, aoa_deg, sideslip_deg)
%ATTITUDEFROMAOA  Body->inertial DCM at a commanded angle of attack and sideslip
%   relative to the ram direction (pitch about body-y, then yaw about body-z).
    if nargin<3, sideslip_deg=0; end
    Rram=dgeom.ramAttitude(vrel); a=deg2rad(aoa_deg); b=deg2rad(sideslip_deg);
    Ry=[cos(a) 0 sin(a); 0 1 0; -sin(a) 0 cos(a)];
    Rz=[cos(b) -sin(b) 0; sin(b) cos(b) 0; 0 0 1];
    R_bi=Rram*Ry*Rz;
end
