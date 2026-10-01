function [m_B, P_W, D, m_end] = mtq(m_cmd, D, t, dt)
%ASILS.DEVICES.MTQ  Magnetorquers: per-coil command (along each coil axis) ->
%   body dipole with per-coil saturation, scale error and axis misalignment.
%   RL DYNAMICS: each coil is a series RL circuit behind a current-limited
%   driver, so its dipole follows the saturated command mc as a first-order
%   lag with the L/R time constant tau (part time_constant_s), exact over a
%   step dt with mc held:
%       m(dt) = mc + (m0 - mc) e^(-dt/tau)                    (m_end)
%       mean  = mc + (m0 - mc) tau/dt (1 - e^(-dt/tau))        (m_B: the torque)
%   Air-core coils: no core, so no hysteresis. A failed coil is open (no
%   current at once). Power linear in the mean drive (Standard Code
%   cfg.actuators power model). Engine: adcs-sim-core actuators.rs Mtq.
    mc = max(-t.m_max, min(t.m_max, m_cmd(:)'));   % per-coil [A m^2]
    mc(D.dead) = 0; D.m(D.dead) = 0;
    [mend, mavg] = lag_(D.m, mc, t.tau, dt);
    D.m = mend; D.mc = mc;
    m_B = D.A * (mavg .* D.scale)';
    m_end = D.A * (mend .* D.scale)';
    P_W = sum(abs(mavg)/t.m_max) * t.p_max;
end

function [xe, xa] = lag_(x0, u, tau, dt)
    if tau <= 0, xe = u; xa = u; return, end
    e = exp(-dt/tau);
    xe = u + (x0 - u)*e;
    xa = u + (x0 - u)*tau/dt*(1 - e);
end
