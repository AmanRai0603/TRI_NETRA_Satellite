function t = wheel_motor(m, i, tc, om)
%ASILS.DEVICES.WHEEL_MOTOR  The torque wheel i's motor delivers for the drive's
%   demand tc at speed om [rad/s]: inside the motor's torque-speed line
%   (back-EMF, k_e = k_t in SI): k_t(+-V - k_t om)/R, i.e.
%       -T_s (1 + om/w_nl) <= t <= T_s (1 - om/w_nl),  T_s = k_t V/R, w_nl = V/k_t
%   and none that would speed it past the drive's speed limit m.speed_max.
%   Engine: adcs-sim-core actuators.rs wheel_motor.
    ts = m.t_stall(i); wn = m.w_nl(i);
    t = max(min(-ts*(1 + om/wn), 0), min(max(ts*(1 - om/wn), 0), tc));
    if abs(om) >= m.speed_max(i) && sign(t) == sign(om), t = 0; end
end
