function K = mekf_predict(K, w_meas, dt)
%ASILS.FSW.MEKF_PREDICT  Propagate the MEKF with the gyro over dt.
    w = w_meas - K.b;
    K.q = asils.quat.norm(asils.quat.mult(K.q, asils.quat.fromrotvec(w*dt)));
    W = asils.util.skew(w);
    Phi = [eye(3) - W*dt + 0.5*(W*W)*dt^2, -eye(3)*dt; zeros(3), eye(3)];
    sv2 = K.arw^2; su2 = K.rrw^2;
    Q = [(sv2*dt + su2*dt^3/3)*eye(3), -(su2*dt^2/2)*eye(3); -(su2*dt^2/2)*eye(3), su2*dt*eye(3)];
    K.P = Phi*K.P*Phi' + Q;
end
