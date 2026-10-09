function y = step(x, dt, B, tau_ext, tau_r, gdot)
%ASILS.PLANT.STEP  One RK4 step of the plant with its inputs held (external torque, rotor momentum rates, gimbal rates):
%   the engine's integrator (adcs-sim-core plant.rs step, the core), in MATLAB. The state's rate is dyn's
%   (asils.models.rigidbody.plant_deriv). With a flexible mode the step is cut so that Omega h stays at most 0.5.
%   x: the state (asils.models.rigidbody.PlantState: q, w, h, d, eta, etad).
%   Owner: Agastya. Copyright (c) 2026 Agastya. All rights reserved.
    if B.flex.on
        n = max(-floor(-B.flex.omega*dt/0.5), 1);
        y = x;
        for k = 1:n, y = rk4_(y, dt/n, B, tau_ext, tau_r, gdot); end
        return
    end
    y = rk4_(x, dt, B, tau_ext, tau_r, gdot);
end

function y = rk4_(x, dt, B, tau_ext, tau_r, gdot)
    k1 = asils.models.rigidbody.plant_deriv(x, B.i, B.iinv, B.minv, B.m, B.flex, tau_ext, tau_r, gdot);
    k2 = asils.models.rigidbody.plant_deriv(axpy_(x, 0.5*dt, k1), B.i, B.iinv, B.minv, B.m, B.flex, tau_ext, tau_r, gdot);
    k3 = asils.models.rigidbody.plant_deriv(axpy_(x, 0.5*dt, k2), B.i, B.iinv, B.minv, B.m, B.flex, tau_ext, tau_r, gdot);
    k4 = asils.models.rigidbody.plant_deriv(axpy_(x, dt, k3), B.i, B.iinv, B.minv, B.m, B.flex, tau_ext, tau_r, gdot);
    y = x; c = dt/6.0; nr = B.m.nr; ng = B.m.ng;
    y.q = x.q + c*(k1.q + 2.0*k2.q + 2.0*k3.q + k4.q);
    y.w = x.w + c*(k1.w + 2.0*k2.w + 2.0*k3.w + k4.w);
    y.h(1:nr) = x.h(1:nr) + c*(k1.h(1:nr) + 2.0*k2.h(1:nr) + 2.0*k3.h(1:nr) + k4.h(1:nr));
    y.d(1:ng) = x.d(1:ng) + c*(k1.d(1:ng) + 2.0*k2.d(1:ng) + 2.0*k3.d(1:ng) + k4.d(1:ng));
    y.eta = x.eta + c*(k1.eta + 2.0*k2.eta + 2.0*k3.eta + k4.eta);
    y.etad = x.etad + c*(k1.etad + 2.0*k2.etad + 2.0*k3.etad + k4.etad);
    y.q = asils.la.qnorm(y.q);
end

function y = axpy_(x, h, k)
    y = x;
    y.q = x.q + h*k.q; y.w = x.w + h*k.w; y.h = x.h + h*k.h; y.d = x.d + h*k.d;
    y.eta = x.eta + h*k.eta; y.etad = x.etad + h*k.etad;
end
