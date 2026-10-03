function xd = deriv(x, I, Iinv, M, tau_ext, tau_r, gdot)
%ASILS.PLANT.DERIV  Rigid body + momentum-exchange devices (+ one flexible mode).
%   x = [q(4); w(3); h(nr); delta(ng); eta; etad]   (eta, etad only with M.flex.on)
%     q      body wrt ECI, scalar-last         w      body rate [rad/s]
%     h      rotor momenta about their spin axes [N m s]: reaction wheels,
%            fluid momentum rings, CMG / VSCMG rotors
%     delta  gimbal angles [rad] of CMG / VSCMG units
%     eta    the flexible mode's coordinate and rate (hybrid coordinates)
%   M  momentum-device geometry (asils.plant.geometry):
%     M.A0 (3 x nr) spin axes at delta = 0, M.G (3 x ng) gimbal axes,
%     M.gi (1 x nr) gimbal index of each rotor (0 = fixed axis);
%     M.flex (optional): on, delta (3x1), omega, zeta, Minv = inv(I - delta delta')
%   tau_ext  external torque (disturbances + magnetorquers + thrusters) [N m]
%   tau_r    rate of change of each rotor momentum, hdot [N m] (device output)
%   gdot     gimbal rates [rad/s] (device output)
%   Euler with a variable-axis momentum H = sum h_i a_i(delta):
%     I wdot = tau_ext - sum(hdot_i a_i + h_i gdot_i (g_i x a_i)) - w x (I w + H)
%     a_i(delta) = cos(delta) a0_i + sin(delta) (g_i x a0_i)
%   With the flexible mode (Hughes, ch. 12; = the engine's plant::deriv):
%     I wdot + delta etadd = ... - w x (I w + H + delta etad)
%     etadd + 2 zeta Omega etad + Omega^2 eta + delta' wdot = 0
    q = x(1:4); w = x(5:7);
    nr = M.nr; ng = M.ng;
    fl = isfield(M, 'flex') && M.flex.on;
    if nr == 0 && ~fl
        H = I*w;
        wd = Iinv*(tau_ext - [w(2)*H(3)-w(3)*H(2); w(3)*H(1)-w(1)*H(3); w(1)*H(2)-w(2)*H(1)]);
        xd = [asils.quat.kin(q, w); wd];
        return
    end
    Hr = zeros(3,1); Hd = zeros(3,1);
    if nr > 0
        h = x(8:7+nr);
        if ng == 0
            A = M.A0; Hr = A*h; Hd = A*tau_r;
        else
            d = x(8+nr:7+nr+ng);
            A = M.A0;
            for i = 1:nr
                j = M.gi(i);
                if j > 0
                    a0 = M.A0(:,i); t0 = M.T0(:,i);
                    A(:,i) = cos(d(j))*a0 + sin(d(j))*t0;
                    ta = -sin(d(j))*a0 + cos(d(j))*t0;            % g x a_i
                    Hd = Hd + h(i)*gdot(j)*ta;
                end
            end
            Hr = A*h; Hd = Hd + A*tau_r;
        end
    end
    Ht = I*w + Hr;
    if fl
        F = M.flex; e = x(8+nr+ng); ed = x(9+nr+ng);
        Ht = Ht + F.delta*ed;
        pull = 2*F.zeta*F.omega*ed + F.omega^2*e;
        wd = F.Minv*(tau_ext - Hd - [w(2)*Ht(3)-w(3)*Ht(2); w(3)*Ht(1)-w(1)*Ht(3); w(1)*Ht(2)-w(2)*Ht(1)] + F.delta*pull);
    else
        wd = Iinv*(tau_ext - Hd - [w(2)*Ht(3)-w(3)*Ht(2); w(3)*Ht(1)-w(1)*Ht(3); w(1)*Ht(2)-w(2)*Ht(1)]);
    end
    xd = [asils.quat.kin(q, w); wd];
    if nr > 0, xd = [xd; tau_r]; end
    if ng > 0, xd = [xd; gdot]; end
    if fl, xd = [xd; ed; -pull - F.delta'*wd]; end
end
