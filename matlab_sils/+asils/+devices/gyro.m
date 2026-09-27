function [w_meas, D] = gyro(w_true, D, g, dt)
%ASILS.DEVICES.GYRO  MEMS rate gyro: scale/misalignment, turn-on bias,
%   rate random walk, angular random walk, range saturation.
%   Structure follows Standard Code sens.gyro (Detumbling.m:2315): scaled and
%   misaligned true rate + bias + white noise; parameters from SYN-GYRO-1.
    D.brw = D.brw + g.rrw*sqrt(dt)*randn(3,1);
    w_meas = D.M*w_true + D.b + D.brw + g.arw/sqrt(dt)*randn(3,1);
    w_meas = max(-g.range, min(g.range, w_meas));
end
