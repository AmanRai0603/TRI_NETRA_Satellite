function B_meas = magnetometer(B_true_B, D, m, m_coil)
%ASILS.DEVICES.MAGNETOMETER  Three-axis magnetometer [T], body frame.
%   Structure follows Standard Code sens.magnetometer (Detumbling.m:1980):
%   scale factor x (misaligned) true field + bias + white noise. ADDED: coil
%   cross-coupling (the reason the flight code measures with the coils off):
%   B += k_coil * m_coil (T per A m^2), and range saturation.
    B_meas = D.M*B_true_B + D.b + m.noise*randn(3,1) + m.k_coil*m_coil;
    B_meas = max(-m.range, min(m.range, B_meas));
end
