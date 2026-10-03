function [r_m, v_m] = gps(r, v, g)
%ASILS.DEVICES.GPS  GNSS receiver fix: position/velocity + white noise (Standard Code sens.gps values).
%   r, v: the state the fix solves for -- the truth latency_s ago
%   (asils.devices.gps_delayed), which the receiver reports as current.
    r_m = r + g.pos_sigma*randn(3,1);
    v_m = v + g.vel_sigma*randn(3,1);
end
