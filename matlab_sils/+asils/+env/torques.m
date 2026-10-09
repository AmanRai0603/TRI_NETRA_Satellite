function parts = torques(q, r, v_rel, b_eci, sun_rel, nu, p_srp, rho, inertia, facets, m_res, mu, on)
%ASILS.ENV.TORQUES  The environment's torques on the body [N m, body]: columns gravity gradient, aero, radiation (Sun,
%   albedo, Earth IR), residual dipole. env's methods, generated from the design into +asils/+models (gravgrad, facets,
%   radiation, dipoletorque); here only their call and which parts the run switches on (the engine's torques.rs).
%   Owner: Agastya. Copyright (c) 2026 Agastya. All rights reserved.
    rm = asils.la.dcm(q);
    parts = zeros(3, 4);
    if on(1), parts(:, 1) = asils.models.gravgrad.gravity_gradient_torque(rm, r, inertia, mu); end
    if on(2), parts(:, 2) = asils.models.facets.aero_torque(rm, v_rel, rho, facets); end
    if on(3), parts(:, 3) = asils.models.radiation.light_torque(rm, r, sun_rel, nu, p_srp, facets); end
    if on(4), parts(:, 4) = asils.models.dipoletorque.residual_dipole_torque(rm, m_res, b_eci); end
end
