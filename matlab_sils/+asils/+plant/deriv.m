function xd = deriv(x, I, Iinv, Aw, tau_ext, tau_w)
%ASILS.PLANT.DERIV  Rigid body + reaction wheels, state x = [q(4); w(3); h_w(nw)].
%   q     body wrt ECI, scalar-last          w   body rate [rad/s], body frame
%   h_w   wheel momenta about their spin axes [N m s]; Aw (3 x nw) spin axes in body
%   tau_ext  external torque (disturbances + magnetorquers) [N m], body
%   tau_w    motor torque applied TO each wheel [N m] (reaction on body = -Aw*tau_w)
%   Euler:  I wdot = tau_ext - Aw tau_w - w x (I w + Aw h_w),   h_w dot = tau_w
    q = x(1:4); w = x(5:7);
    nw = size(Aw, 2);
    if nw > 0
        hw = x(8:7+nw);
        H = I*w + Aw*hw;
        wd = Iinv*(tau_ext - Aw*tau_w - [w(2)*H(3)-w(3)*H(2); w(3)*H(1)-w(1)*H(3); w(1)*H(2)-w(2)*H(1)]);
        xd = [asils.quat.kin(q, w); wd; tau_w];
    else
        H = I*w;
        wd = Iinv*(tau_ext - [w(2)*H(3)-w(3)*H(2); w(3)*H(1)-w(1)*H(3); w(1)*H(2)-w(2)*H(1)]);
        xd = [asils.quat.kin(q, w); wd];
    end
end
