function x = step(x, dt, I, Iinv, Aw, tau_ext, tau_w)
%ASILS.PLANT.STEP  One RK4 step of the attitude plant (torques held over dt);
%   the quaternion is renormalised after the step (SPEC.md 9.2 rule).
    k1 = asils.plant.deriv(x, I, Iinv, Aw, tau_ext, tau_w);
    k2 = asils.plant.deriv(x + 0.5*dt*k1, I, Iinv, Aw, tau_ext, tau_w);
    k3 = asils.plant.deriv(x + 0.5*dt*k2, I, Iinv, Aw, tau_ext, tau_w);
    k4 = asils.plant.deriv(x + dt*k3, I, Iinv, Aw, tau_ext, tau_w);
    x = x + dt/6*(k1 + 2*k2 + 2*k3 + k4);
    x(1:4) = x(1:4)/sqrt(x(1:4)'*x(1:4));
end
