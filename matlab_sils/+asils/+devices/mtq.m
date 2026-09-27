function [m_B, P_W] = mtq(m_cmd, D, t)
%ASILS.DEVICES.MTQ  Magnetorquers: per-coil command (along each coil axis) ->
%   body dipole with per-coil saturation, scale error and axis misalignment.
%   Power linear in duty (Standard Code cfg.actuators power model).
    mc = max(-t.m_max, min(t.m_max, m_cmd(:)'));   % per-coil [A m^2]
    m_B = D.A * (mc .* D.scale)';
    P_W = sum(abs(mc)/t.m_max) * t.p_max;
end
