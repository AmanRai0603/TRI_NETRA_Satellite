function p = rw(Dm, caseId)
%ASILS.SIZING.RW  Reaction wheel (benchmark) sized to the case: one per body axis.
%   Momentum h = Dm.h_req, torque tau = Dm.tau_req, floored at the smallest
%   practical CubeSat wheel (1 mN m s, 0.1 mN m) so the benchmark is a part
%   that could be bought.
%   Rotor: rim flywheel at 6000 rpm, J = h/W, rim radius r = 21 mm (h/10 mNms)^0.2,
%   rotor mass J/(0.9 r^2). Unit mass 2.2 m_rotor + 60 g (motor, bearings,
%   housing, drive electronics). Power: 0.2 W + 30 W/(N m s) h at speed
%   (bearing and drive losses), peak adds tau W / 0.5. Envelope: cylinder
%   (2.4 r + 10 mm) across, (0.6 r + 20 mm) tall.
%   JITTER: ISO 1940 balance grade G2.5 -> eccentricity e = G/W, static
%   imbalance U_s = m_rotor e, dynamic U_d = U_s r/2 (force U_s W^2, torque
%   U_d W^2 at the wheel speed).
%   Anchor: reproduces SYN-RW-10 (10 mNms: J 1.6e-5, 0.15 kg, U_s 1e-7 class).
    h = max(Dm.h_req, 1e-3); tau = max(Dm.tau_req, 1e-4);
    W = 6000*2*pi/60; J = h/W;
    r = 0.021*(h/0.01)^0.2; mr = J/(0.9*r^2);
    Gg = 2.5e-3; Us = mr*Gg/W; Ud = Us*r/2;
    Pst = 0.2 + 30*h; Ppk = Pst + tau*W/0.5;
    dia = 2.4*r + 0.010; hgt = 0.6*r + 0.020;
    p = part_(caseId, 'RW', 'reaction_wheel', 'Sized reaction wheel (benchmark)');
    p.nominal = struct('h_max_Nms', h, 'torque_max_Nm', tau, 'speed_max_rad_s', W, 'rotor_inertia_kgm2', J, ...
        'rotor_radius_m', r, 'rotor_mass_kg', mr, 'friction_coulomb_Nm', 1e-5*sqrt(h/0.01), 'friction_viscous_Nms', 1e-8, ...
        'static_imbalance_kgm', Us, 'dynamic_imbalance_kgm2', Ud, 'power_steady_W', Pst, 'power_peak_W', Ppk, ...
        'mass_kg', 2.2*mr + 0.06, 'volume_L', pi*dia^2/4*hgt*1e3);
    % motor and bearing (B3.5; the rule of tools/catalogue.py wheel_motor): a 5 V drive, no-load
    % speed 1.25 W, the torque-speed line through tau at W; breakaway 1.5 x Coulomb, Stribeck 1 rad/s
    V = 5; wnl = 1.25*W; Ts = tau/(1 - W/wnl); kt = V/wnl;
    p.nominal.motor_kt_Nm_per_A = kt; p.nominal.motor_resistance_ohm = kt*V/Ts; p.nominal.bus_voltage_V = V;
    p.nominal.friction_static_Nm = 1.5*p.nominal.friction_coulomb_Nm; p.nominal.stribeck_speed_rad_s = 1.0;
    p.dispersion = struct('torque_scale', struct('dist', 'normal', 'mean', 1, 'sigma', 0.01), ...
        'friction_scale', struct('dist', 'uniform', 'lo', 0.5, 'hi', 2.0), ...
        'axis_misalignment_rad', struct('dist', 'normal', 'mean', 0, 'sigma', 0.001));
    p.sizing = struct('h_req_Nms', Dm.h_req, 'tau_req_Nm', Dm.tau_req, 'floor', 'h >= 1 mNms, tau >= 0.1 mNm', ...
        'law', 'rim flywheel 6000 rpm, ISO 1940 G2.5, anchored on SYN-RW-10');
end

function p = part_(caseId, tag, kind, name)
    p = struct('part_number', sprintf('SZ-%s-%s', caseId, tag), 'kind', kind, 'name', [name ' — ' caseId], ...
        'status', 'sized', 'source', 'asils.sizing', 'made', 'bought', 'descriptor_version', 1);
end
