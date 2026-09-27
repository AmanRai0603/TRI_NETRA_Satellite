function K = mekf_init(q0, sig_att0, sig_bias0, arw, rrw)
%ASILS.FSW.MEKF_INIT  Multiplicative EKF (Markley & Crassidis 2014, ch. 6.2).
%   State: q_B/ECI (global), gyro bias b; error state [dtheta; db] (6).
%   AD.MEKF was a PRIMED STUB in the Standard Code; this is its implementation.
    K.q = q0(:); K.b = zeros(3,1);
    K.P = blkdiag(sig_att0^2*eye(3), sig_bias0^2*eye(3));
    K.arw = arw; K.rrw = rrw;
end
