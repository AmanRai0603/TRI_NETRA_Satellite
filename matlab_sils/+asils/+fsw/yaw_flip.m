function gd = yaw_flip(gd, r, v, hyst)
%ASILS.FSW.YAW_FLIP  Nadir-family yaw flip with hysteresis (04_guidance.md): turn 180 deg
%   about the boresight when the power face gd.sun_axis would look away from the Sun.
%   Copyright (c) 2026 Agastya. All rights reserved.
    g0 = gd; g0.flip = false;
    q = asils.fsw.guidance('nadir', r, v, 0, g0);
    sb = asils.quat.dcm(q)*(gd.sun_eci(:)/norm(gd.sun_eci));
    a = [0; 0; -1]; if isfield(gd, 'sun_axis'), a = gd.sun_axis(:)/norm(gd.sun_axis); end
    d = a'*sb;
    if d < -hyst, gd.flip = true; elseif d > hyst, gd.flip = false; end
end
