function [duty, m_out] = drive(m_cmd, m_res_est, m_max)
%ASILS.COMP.MAGNETORQUER.DRIVE  Coil driver: dipole command -> PWM duty per
%   coil (the HAL's adcs_hal_pwm_set, Q15 of the part's full dipole):
%   residual-dipole compensation, then direction-preserving saturation
%   (Standard Code act.saturateDipole). The measure-then-drive schedule
%   (coils off 0.2 s of each second) is the caller's (asils.fsw.step).
    m_out = m_cmd - m_res_est;
    if any(m_out), m_out = m_out*min(1, m_max/max(abs(m_out))); end
    duty = m_out/m_max;
end
