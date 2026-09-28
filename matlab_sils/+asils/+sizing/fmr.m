function P = fmr(Dm, caseId, box)
%ASILS.SIZING.FMR  Fluid momentum loop (OUR product), one ring per body axis,
%   each laid in the face normal to its axis (IDMAS panel geometry).
%   h = N rho A 2 S v   (N loops of bore d in a face of enclosed area S)
%   Per ring: S = 0.8 x the face (a 5 mm frame), v_max 0.5 m/s. The bore is
%   chosen for h_req at v_max, 2 to 6 mm; beyond 6 mm the speed rises to
%   1 m/s, then loops are added. Galinstan: rho 6440 kg/m^3, mu 2.4 mPa s.
%   Loss: Darcy friction, laminar 64/Re below Re 2300, Blasius above.
%   Power at the cruise speed (0.4 v_max): hydraulic / pump efficiency (0.10)
%   PLUS the pump field power while pumping: 2 W at a 3 mm bore, scaled with
%   the bore (IDMAS v2 §03C: 1-3 W per unit, to be measured). The field power
%   dominates: the loop spins down in about a second, so it pumps whenever it
%   holds momentum. power_steady_permanent_magnet_W gives the same loop with a
%   permanent-magnet yoke (no field power): the design lever this exposes.
%   Mass: fluid + 30 g channel + 60 g (d/3 mm) pump yoke. Envelope: the panel
%   (face x 6 mm). JITTER: no rotating mass; flow ripple only (none modelled
%   beyond the flow-sensor noise).
%   Anchor: reproduces SYN-MFP-1 (3 mm, S 0.022 m^2, 1 mNms at 0.5 m/s, 0.12 kg).
%   Returns three part structs (X, Y, Z rings).
    if nargin < 3, box = [0.34 0.10 0.10]; end
    rho = 6440; mu = 0.0024; eta = 0.10;
    faces = [box(2)*box(3), box(1)*box(3), box(1)*box(2)];      % face normal to X, Y, Z
    per = 2*[box(2)+box(3), box(1)+box(3), box(1)+box(2)];
    h = max(Dm.h_req, 2e-4);
    ax = 'XYZ'; P = cell(1, 3);
    for i = 1:3
        S = 0.8*faces(i); L1 = 0.8*per(i);
        v = 0.5; N = 1;
        d = sqrt(4*h/(rho*pi*2*S*v));
        if d > 0.006, d = 0.006; v = min(1.0, h/(rho*pi*d^2/4*2*S)); end
        A = pi*d^2/4;
        if rho*A*2*S*v < h, N = ceil(h/(rho*A*2*S*v)); end
        d = max(d, 0.002); A = pi*d^2/4;
        hmax = N*rho*A*2*S*v; L = N*L1;
        vc = 0.4*v; Re = rho*vc*d/mu;
        if Re < 2300, f = 64/Re; else, f = 0.316*Re^-0.25; end
        dP = f*(L/d)*rho*vc^2/2; Ph = dP*A*vc;
        Pfield = 2.0*(d/0.003);
        mfl = rho*A*L;
        p = struct('part_number', sprintf('SZ-%s-FMR-%s', caseId, ax(i)), 'kind', 'magneto_fluidic_panel', ...
            'name', sprintf('Sized fluid momentum loop, %s axis (our product) — %s', ax(i), caseId), ...
            'status', 'sized', 'source', 'asils.sizing', 'made', 'in-house', 'descriptor_version', 1);
        p.nominal = struct('bore_m', d, 'enclosed_area_m2', N*S, 'loops', N, 'channel_length_m', L, 'fluid', 'galinstan', ...
            'fluid_density_kg_m3', rho, 'fluid_viscosity_Pa_s', mu, 'fluid_mass_kg', mfl, 'v_cruise_m_s', vc, ...
            'v_max_m_s', v, 'h_max_Nms', hmax, 'reynolds_cruise', Re, 'pump_type', 'dc-conduction-yoke', ...
            'pump_efficiency', eta, 'field_power_W', Pfield, 'hydraulic_power_cruise_W', Ph, ...
            'power_steady_W', Pfield + Ph/eta, 'power_steady_permanent_magnet_W', Ph/eta, 'melt_point_K', 254, ...
            'dipole_max_Am2', 0, 'dipole_per_amp_Am2_per_A', 0, 'current_max_A', 0, ...
            'mass_kg', mfl + 0.03 + 0.06*(d/0.003), 'volume_L', faces(i)*0.006*1e3);
        p.dispersion = struct('friction_scale', struct('dist', 'uniform', 'lo', 0.8, 'hi', 1.2), ...
            'pump_efficiency', struct('dist', 'uniform', 'lo', 0.05, 'hi', 0.15), ...
            'flow_sensor_noise_m_s', struct('dist', 'normal', 'mean', 0, 'sigma', 0.002), ...
            'axis_misalignment_rad', struct('dist', 'normal', 'mean', 0, 'sigma', 0.005));
        p.sizing = struct('h_req_Nms', Dm.h_req, 'face_m2', faces(i), 'laminar', Re < 2300, ...
            'law', 'h = N rho A 2 S v; Darcy loss; field power 2 W x d/3mm (IDMAS v2 §03C, to be measured)');
        P{i} = p;
    end
end
