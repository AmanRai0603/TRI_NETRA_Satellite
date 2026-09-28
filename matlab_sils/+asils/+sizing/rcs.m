function p = rcs(Dm, caseId, box)
%ASILS.SIZING.RCS  N2O cold-gas RCS (OUR product): six nozzle couples (12
%   nozzles), a +/- pure torque about each body axis.
%   THRUST: the same angular acceleration on every axis, enough for the case's
%   slew torque (tau_req / J_max) and to detumble in 10 min
%   (J w0 / 600 s / J_max): F = J_axis alpha / (2 arm_axis), the largest axis
%   rounded up to a valve class (5, 10, 20, 50, 100 mN).
%   ARMS: across the short face (X couples) 0.45 x the short side, along the
%   long axis (Y, Z couples) 0.45 x the long side.
%   PROPELLANT (Isp 60 s, the low end of N2O's 60-80 s, 20 % reserve), for
%   the RCS's jobs in our families -- momentum dumping and detumble (the
%   momentum devices fly the slews):
%     detumble  2 events x J_max w0 / arm
%     dumping   the secular momentum of every orbit of the life / arm
%   arm = harmonic mean of the three couple arms. The propellant for slews
%   flown on the thrusters (an agility option) is reported separately.
%   TANK: N2O self-pressurising liquid, 745 kg/m^3 at 25 C, 25 % ullage;
%   spherical Al-7075 vessel for a 70 bar MEOP (vapour pressure at 40 C),
%   allowable 250 MPa (safety factor 2).
%   Dry mass: tank + 12 x 10 g valve-nozzles + 50 g manifold and regulator-free
%   plumbing. Valve power 1 W per open valve. MIB 5 ms, resolution 1 ms.
    if nargin < 3, box = [0.34 0.10 0.10]; end
    g0 = 9.80665; isp = 60; J = Dm.J(:);
    arm = [0.45*box(2); 0.45*box(1); 0.45*box(1)];
    alpha = max(Dm.tau_req/max(J), Dm.h_detumble/600/max(J));
    Freq = max(J.*alpha./(2*arm));
    cls = [0.005 0.010 0.020 0.050 0.100];
    F = cls(find(cls >= Freq, 1)); if isempty(F), F = ceil(Freq/0.05)*0.05; end
    arm_eff = 3/sum(1./arm);
    orbits = Dm.life_yr*365.25*86400/Dm.orbit_period_s;
    It_det = 2*Dm.h_detumble/arm_eff;
    It_dump = Dm.h_secular*orbits/arm_eff;
    It_slew = Dm.slews_per_day*365.25*Dm.life_yr*2*Dm.h_slew/arm_eff;
    mprop = 1.2*(It_det + It_dump)/(isp*g0);
    mslew = 1.2*It_slew/(isp*g0);
    mprop = max(mprop, 0.01);
    V = 1.25*mprop/745;                                % m^3
    rt = (3*V/(4*pi))^(1/3); Pm = 70e5; sig = 250e6;
    t = max(Pm*rt/(2*sig), 0.5e-3);                    % wall, 0.5 mm minimum gauge
    mtank = 4*pi*rt^2*t*2810;
    mdry = mtank + 12*0.010 + 0.050;
    p = struct('part_number', sprintf('SZ-%s-RCS', caseId), 'kind', 'rcs', ...
        'name', ['Sized N2O cold-gas RCS, 6 couples (our product) — ' caseId], 'status', 'sized', ...
        'source', 'asils.sizing', 'made', 'in-house', 'descriptor_version', 1);
    p.nominal = struct('thrust_N', F, 'isp_s', 70, 'propellant', 'N2O', 'thrusters', 12, 'mib_s', 0.005, ...
        'valve_res_s', 0.001, 'arm_long_m', arm(2), 'arm_short_m', arm(1), 'propellant_kg', mprop, ...
        'tank_volume_L', V*1e3, 'tank_radius_m', rt, 'tank_wall_m', t, 'tank_pressure_bar', 50, 'meop_bar', 70, ...
        'valve_power_W', 1.0, 'power_steady_W', 0.05, 'dry_mass_kg', mdry, 'mass_kg', mdry + mprop, ...
        'volume_L', (2*rt)^3*1e3 + 0.05);
    p.dispersion = struct('thrust_scale', struct('dist', 'normal', 'mean', 1, 'sigma', 0.03), ...
        'isp_s', struct('dist', 'uniform', 'lo', 60, 'hi', 80), ...
        'axis_misalignment_rad', struct('dist', 'normal', 'mean', 0, 'sigma', 0.01));
    p.sizing = struct('F_req_N', Freq, 'impulse_detumble_Ns', It_det, 'impulse_dumping_Ns', It_dump, ...
        'impulse_slews_Ns', It_slew, 'propellant_if_slews_on_rcs_kg', mslew, 'life_yr', Dm.life_yr, 'isp_budget_s', isp, ...
        'law', 'N2O self-pressurised, Isp 60 s budget (60-80 s), Al-7075 sphere at 70 bar MEOP');
end
