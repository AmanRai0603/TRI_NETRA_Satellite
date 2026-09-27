function [tau, I_q] = control_law(q, w, q_ref, w_ref, I_q, dt, g, Ib, Hs, wdot_ref)
%ASILS.FSW.CONTROL_LAW  Three-axis attitude control torque (body) for the fine modes.
%   g.law  'pid' | 'lqr' | 'smc'
%   Common: q_e = conj(q_ref) (x) q, rate error w_e = w - A(q_e) w_ref,
%   gyroscopic compensation w x (Ib w + Hs), reference-acceleration feedforward.
%
%   pid  PORTED from Standard Code ctrl.nadirPointing pid_ (R42): clipped
%        vector part, integral of q_e (anti-windup clamp), rate error.
%        Gains from bandwidth wn, damping zeta: Kp = I wn^2, Kd = 2 zeta I wn.
%   lqr  per-axis optimal state feedback on [int theta; theta; omega] with the
%        weights from Bryson's rule (asils.fsw.lqr_gain, gains made at config).
%   smc  quaternion sliding mode (Crassidis & Markley 1996): surface
%        s = w_e + lambda sign(q4) q_ev, reaching law with a boundary layer phi.
    if nargin < 10, wdot_ref = zeros(3,1); end
    qe = asils.quat.mult(asils.quat.conj(q_ref), q);
    if qe(4) < 0, qe = -qe; end
    ev = max(-g.err_max, min(g.err_max, qe(1:3)));
    we = w - asils.quat.dcm(qe)*w_ref;
    H = Ib*w + Hs;
    gyro = [w(2)*H(3)-w(3)*H(2); w(3)*H(1)-w(1)*H(3); w(1)*H(2)-w(2)*H(1)];
    ff = Ib*wdot_ref;
    switch g.law
        case 'pid'
            I_q = max(-g.int_max, min(g.int_max, I_q + ev*dt));
            tau = -g.Kp.*ev - g.Kd.*we - g.Ki.*I_q;
        case 'lqr'
            I_q = max(-g.int_max, min(g.int_max, I_q + 2*ev*dt));
            tau = -(g.Klqr(:,1).*I_q + g.Klqr(:,2).*(2*ev) + g.Klqr(:,3).*we);
        case 'smc'
            s = we + g.lambda*ev;
            sat = max(-1, min(1, s/g.phi));
            evd = 0.5*(qe(4)*we + [ev(2)*we(3)-ev(3)*we(2); ev(3)*we(1)-ev(1)*we(3); ev(1)*we(2)-ev(2)*we(1)]);
            tau = -Ib*(g.lambda*evd + g.Gs.*sat);
        otherwise
            error('asils:fsw:law', 'unknown control law %s', g.law);
    end
    tau = tau + gyro + ff;
end
