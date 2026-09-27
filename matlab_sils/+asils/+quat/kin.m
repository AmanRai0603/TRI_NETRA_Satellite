function qd = kin(q, w)
%ASILS.QUAT.KIN  Attitude kinematics qdot = 0.5 * q (x) [w;0], w = body rate of B wrt I in B.
    qd = 0.5*[ q(4)*w(1) + q(2)*w(3) - q(3)*w(2);
               q(4)*w(2) - q(1)*w(3) + q(3)*w(1);
               q(4)*w(3) + q(1)*w(2) - q(2)*w(1);
              -q(1)*w(1) - q(2)*w(2) - q(3)*w(3)];
end
