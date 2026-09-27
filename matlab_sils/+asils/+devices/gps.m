function [r_m, v_m] = gps(r, v, g)
%ASILS.DEVICES.GPS  GNSS receiver fix: position/velocity + white noise (Standard Code sens.gps values).
    r_m = r + g.pos_sigma*randn(3,1);
    v_m = v + g.vel_sigma*randn(3,1);
end
