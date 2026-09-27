function [tau_w, P_W] = rw(tau_cmd, h_w, D, w)
%ASILS.DEVICES.RW  Reaction wheels: commanded motor torque -> torque on each rotor.
%   Torque limit, speed (momentum) limit, Coulomb + viscous friction, 0.1 %
%   torque noise (of max torque, white at the loop rate), torque scale error. Friction and limits are what make a
%   wheel's momentum drift and saturate. Power: steady + |tau*omega|/eta.
%   Model structure after Standard Code act.rwModel (R42 wheel plant).
%   The wheel DRIVER closes its own current/speed loop and cancels friction
%   with its model: only (1 - w.friction_comp) of the friction reaches the
%   rotor (default 5 % residual), plus the torque noise.
    n = numel(h_w);
    tc = max(-w.torque_max, min(w.torque_max, tau_cmd(:))) .* D.tscale(:);
    om = h_w(:)/w.J;
    fr = (w.coulomb*sign(om) + w.viscous*om) .* D.fscale(:);
    tau_w = tc - (1 - w.friction_comp)*fr + w.torque_noise*w.torque_max*randn(n,1);
    over = abs(h_w(:)) >= w.h_max & sign(tau_w) == sign(h_w(:));
    tau_w(over) = -fr(over);                    % driver will not spin past the limit
    P_W = n*w.p_steady + sum(abs(tc.*om))/w.eta;
end
