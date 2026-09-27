function K = mekf_vector(K, b_meas, r_ref, sigma)
%ASILS.FSW.MEKF_VECTOR  MEKF update with one unit-vector measurement (Sun, field).
%   b_meas body unit vector, r_ref its ECI reference model, sigma [rad] 1-sigma.
    b_meas = b_meas/norm(b_meas); r_ref = r_ref/norm(r_ref);
    bh = asils.quat.dcm(K.q)*r_ref;
    H = [asils.util.skew(bh), zeros(3)];
    R = sigma^2*eye(3);
    S = H*K.P*H' + R;
    G = K.P*H'/S;
    dx = G*(b_meas - bh);
    K = apply_(K, dx, G, H, R);
end
function K = apply_(K, dx, G, H, R)
    K.q = asils.quat.norm(asils.quat.mult(K.q, [0.5*dx(1:3); 1]));
    K.b = K.b + dx(4:6);
    IKH = eye(6) - G*H;
    K.P = IKH*K.P*IKH' + G*R*G';
end
