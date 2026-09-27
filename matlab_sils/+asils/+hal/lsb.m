function L = lsb(dev)
%ASILS.HAL.LSB  Register scaling of every device frame (one LSB in SI units).
%   Sensors (FSW reads):    gyro int32, magnetometer int16, sun vector Q15,
%   star tracker quaternion Q30, GNSS position cm / velocity mm/s (int32),
%   rotor momentum int32, gimbal angle int32.
%   Actuators (FSW writes): coil PWM duty Q15 of full dipole, rotor torque Q15
%   of full torque, gimbal rate Q15 of full rate, thruster on-time in ms.
    L.gyro = 5/2^23;                     % 24-bit, +/-5 rad/s
    L.mag = 1e-4/2^15;                   % 16-bit, +/-100 uT  (3.05 nT)
    L.sun = 1/2^15;
    L.q = 1/2^30;
    L.pos = 0.01; L.vel = 0.001;
    L.h = 1e-9; L.delta = 1e-7;
    L.coil = 1/2^15; L.torque = 1/2^15; L.grate = 1/2^15; L.rcs_ms = 1e-3;
    L.m_max = 1; if dev.mtq.fitted, L.m_max = dev.mtq.m_max; end
    L.tmax = []; if dev.mex.fitted, L.tmax = dev.mex.torque_max(:); end
    L.gmax = 1; if dev.mex.fitted && ~isempty(dev.mex.G), L.gmax = dev.mex.gimbal_rate_max; end
end
