function tau = mtq_pd(q, w, q_ref, w_ref, g)
%ASILS.FSW.MTQ_PD  Magnetic-only three-axis pointing law (coarse, AIS case).
%   Quaternion PD torque request, later mapped to a dipole by torque2dipole:
%     tau = -Kp * sign(q_e4) q_e,vec - Kd (w - A(q_e) w_ref)
%   Averaged over an orbit the field rotates, so all three axes are
%   controllable; the Kp/Kd are chosen far below the magnetic authority
%   (Lovera & Astolfi 2004, Wisniewski 1999).
    qe = asils.quat.mult(asils.quat.conj(q_ref), q);
    s = sign(qe(4)); if s == 0, s = 1; end
    we = w - asils.quat.dcm(qe)*w_ref;      % reference rate carried into body axes
    tau = -g.Kp.*(s*qe(1:3)) - g.Kd.*we;
end
