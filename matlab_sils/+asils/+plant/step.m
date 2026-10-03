function x = step(x, dt, I, Iinv, M, tau_ext, tau_r, gdot)
%ASILS.PLANT.STEP  One RK4 step of the attitude plant (inputs held over dt);
%   the quaternion is renormalised after the step (SPEC.md 9.2 rule). With a flexible mode the
%   step is cut so that Omega h stays at most 0.5 (= the engine's plant::step).
    if nargin < 8, gdot = zeros(0,1); end
    n = 1;
    if isfield(M, 'flex') && M.flex.on, n = max(1, ceil(M.flex.omega*dt/0.5)); end
    h = dt/n;
    for s = 1:n
        k1 = asils.plant.deriv(x, I, Iinv, M, tau_ext, tau_r, gdot);
        k2 = asils.plant.deriv(x + 0.5*h*k1, I, Iinv, M, tau_ext, tau_r, gdot);
        k3 = asils.plant.deriv(x + 0.5*h*k2, I, Iinv, M, tau_ext, tau_r, gdot);
        k4 = asils.plant.deriv(x + h*k3, I, Iinv, M, tau_ext, tau_r, gdot);
        x = x + h/6*(k1 + 2*k2 + 2*k3 + k4);
        x(1:4) = x(1:4)/sqrt(x(1:4)'*x(1:4));
    end
end
