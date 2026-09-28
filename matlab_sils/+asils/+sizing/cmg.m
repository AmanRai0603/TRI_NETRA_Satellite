function p = cmg(Dm, caseId, variable)
%ASILS.SIZING.CMG  Four-unit pyramid (skew 54.74 deg) of single-gimbal CMGs
%   (benchmark), or of variable-speed CMGs when variable = true.
%   Rotor momentum h0 = h_req/2 (the pyramid holds ~2.5 h0 in its weakest
%   direction; 2 h0 keeps clear of the internal singular surfaces), floored at
%   1 mNms. Gimbal rate so that h0 dgimbal covers tau_req with 20 % margin,
%   0.5 to 3 rad/s. Rotor 800 rad/s constant (VSCMG: +/-100 % speed, a wheel
%   torque of 0.2 tau_req). Rotor radius 15 mm (h0/4 mNms)^0.2; unit mass
%   2.5 m_rotor + 60 g (gimbal motor, slip ring, encoder); VSCMG x1.1.
%   Power per unit: rotor 0.2 W + 30 W/(N m s) h0, gimbal 0.3 W at 1.5 rad/s.
%   JITTER: rotor imbalance at a CONSTANT speed, ISO 1940 G2.5.
%   Anchor: reproduces TRN-CMG-1 (h0 4 mNms, J 5e-6, 0.12 kg).
    if nargin < 3, variable = false; end
    h0 = max(Dm.h_req/2, 1e-3); tau = max(Dm.tau_req, 1e-4);
    W = 800; J = h0/W; r = 0.015*(h0/0.004)^0.2; mr = J/(0.9*r^2);
    gr = min(3, max(0.5, 1.2*tau/h0));
    Us = mr*2.5e-3/W; Ud = Us*r/2;
    dia = 2.4*r + 0.015; hgt = 2.4*r + 0.020;
    m = 2.5*mr + 0.06; Pst = 0.2 + 30*h0;
    tag = 'CMG'; kind = 'cmg'; name = 'Sized single-gimbal CMG (benchmark, one of 4)';
    if variable, tag = 'VSCMG'; kind = 'vscmg'; name = 'Sized variable-speed CMG (benchmark, one of 4)'; m = 1.1*m; Pst = Pst + 0.1; end
    p = struct('part_number', sprintf('SZ-%s-%s', caseId, tag), 'kind', kind, 'name', [name ' — ' caseId], ...
        'status', 'sized', 'source', 'asils.sizing', 'made', 'bought', 'descriptor_version', 1);
    p.nominal = struct('rotor_momentum_Nms', h0, 'rotor_inertia_kgm2', J, 'rotor_radius_m', r, 'rotor_mass_kg', mr, ...
        'rotor_speed_rad_s', W, 'rotor_torque_max_Nm', max(1e-4, 0.2*tau), 'gimbal_rate_max_rad_s', gr, ...
        'gimbal_power_W', 0.3*gr/1.5, 'power_steady_W', Pst, 'static_imbalance_kgm', Us, 'dynamic_imbalance_kgm2', Ud, ...
        'mass_kg', m, 'volume_L', pi*dia^2/4*hgt*1e3);
    p.dispersion = struct('torque_scale', struct('dist', 'normal', 'mean', 1, 'sigma', 0.01), ...
        'axis_misalignment_rad', struct('dist', 'normal', 'mean', 0, 'sigma', 0.001));
    if variable
        p.nominal.h_max_Nms = 2*h0; p.nominal.friction_coulomb_Nm = 1e-5; p.nominal.friction_viscous_Nms = 1e-8;
        p.dispersion.friction_scale = struct('dist', 'uniform', 'lo', 0.5, 'hi', 2.0);
    end
    p.sizing = struct('h_req_Nms', Dm.h_req, 'tau_req_Nm', Dm.tau_req, 'units', 4, ...
        'law', 'pyramid 54.74 deg, h0 = h_req/2, rotor 800 rad/s, ISO 1940 G2.5, anchored on TRN-CMG-1');
end
