function [tau_w_cmd, tau_B, I_q] = pd_alloc(q, w, q_ref, w_ref, h_w, I_q, dt, g, Ib, Aw, Awp)
%ASILS.FSW.PD_ALLOC  Quaternion PID + gyroscopic compensation + wheel allocation
%   (catalogue algorithm 'pd_alloc').
%   PID PORTED from Standard Code ctrl.nadirPointing pid_ (R42
%   compute_PID_controller_nadir_pointing_obc): error quaternion
%   q_e = conj(q_ref) (x) q, clipped vector part, integral of q_e, rate error.
%   Gains from bandwidth wn and damping zeta: Kp = I wn^2, Kd = 2 zeta I wn,
%   Ki = I wn^3 / 10 (the spec's bandwidth/damping parameters).
%   ADDED: w x (I w + Aw h) compensation; integral anti-windup clamp; body
%   torque -> wheel torques with the pseudo-inverse of the wheel axes.
    qe = asils.quat.mult(asils.quat.conj(q_ref), q);
    if qe(4) < 0, qe = -qe; end
    ev = max(-g.err_max, min(g.err_max, qe(1:3)));
    I_q = max(-g.int_max, min(g.int_max, I_q + ev*dt));
    we = w - asils.quat.dcm(qe)*w_ref;      % reference rate carried into body axes
    H = Ib*w + Aw*h_w;
    tau_B = -g.Kp.*ev - g.Kd.*we - g.Ki.*I_q + asils.util.cross3(w, H);
    tau_w_cmd = -Awp*tau_B;          % wheel motor torques whose reaction is tau_B
end
