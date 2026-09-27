function x = step(x, dt, I, Iinv, M, tau_ext, tau_r, gdot)
%ASILS.PLANT.STEP  One RK4 step of the attitude plant (inputs held over dt);
%   the quaternion is renormalised after the step (SPEC.md 9.2 rule).
    if nargin < 8, gdot = zeros(0,1); end
    k1 = asils.plant.deriv(x, I, Iinv, M, tau_ext, tau_r, gdot);
    k2 = asils.plant.deriv(x + 0.5*dt*k1, I, Iinv, M, tau_ext, tau_r, gdot);
    k3 = asils.plant.deriv(x + 0.5*dt*k2, I, Iinv, M, tau_ext, tau_r, gdot);
    k4 = asils.plant.deriv(x + dt*k3, I, Iinv, M, tau_ext, tau_r, gdot);
    x = x + dt/6*(k1 + 2*k2 + 2*k3 + k4);
    x(1:4) = x(1:4)/sqrt(x(1:4)'*x(1:4));
end
