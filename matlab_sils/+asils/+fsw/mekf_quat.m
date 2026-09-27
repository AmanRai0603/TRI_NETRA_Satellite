function K = mekf_quat(K, q_meas, sig_cross, sig_roll, bs)
%ASILS.FSW.MEKF_QUAT  MEKF update with a star-tracker attitude (latency already compensated).
%   Measurement noise: cross-boresight sig_cross, roll about boresight bs sig_roll.
    dq = asils.quat.mult(asils.quat.conj(K.q), q_meas);
    if dq(4) < 0, dq = -dq; end
    y = 2*dq(1:3);
    H = [eye(3), zeros(3)];
    R = sig_cross^2*eye(3) + (sig_roll^2 - sig_cross^2)*(bs*bs');
    S = H*K.P*H' + R;
    G = K.P*H'/S;
    dx = G*y;
    K.q = asils.quat.norm(asils.quat.mult(K.q, [0.5*dx(1:3); 1]));
    K.b = K.b + dx(4:6);
    IKH = eye(6) - G*H;
    K.P = IKH*K.P*IKH' + G*R*G';
end
