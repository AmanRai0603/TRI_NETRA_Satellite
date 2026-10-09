function dev = load(productId)
%ASILS.PRODUCT.LOAD  A catalogue product -> the device descriptors a run flies (the engine's adcs-sim product.rs Dev::load,
%   in MATLAB): data/products/<id>.json and every part it fills from data/parts/ (or a case-sized one in store/sized).
%
%   Reading only: each fitted device's descriptor is the design's record (the generated models' GyroDesc, MagDesc,
%   SunDesc, CssDesc, StDesc, EsDesc, CoilDesc, RotorDesc, ThrusterDesc: dev.<device>.rec) filled from its part, the
%   descriptors' rules called where the design states one (act's motor_constants, ring_constants, ring_pump_max,
%   ring_flow_noise, couple_arms, couple_torques; sens's st_calibrated, es_boresight, st_focal; pnt's rotor_imbalance;
%   the budget's budget_line and budget_total), the values a product leaves to the design read from data/stated.json
%   (gdn's axes, act's device defaults and telemetry noises). Every value a fitted device reads must be stated by its
%   part; one it does not state refuses the product by name, never read as 0. A slot not filled is fitted = false.
%   Owner: Agastya. Copyright (c) 2026 Agastya. All rights reserved.
    R = asils.util.root();
    pf = asils.product.find(R, 'products', productId);
    pr = asils.util.readjson(pf);
    st = asils.util.stated();
    fills = asils.util.getf(pr, 'fill', {}); if ~iscell(fills), fills = num2cell(fills); end
    dev = struct('id', asils.util.getf(pr, 'id', productId), 'label', asils.util.getf(pr, 'label', ''), ...
                 'family', asils.util.getf(pr, 'family', ''), 'algorithms', {cellstr_(asils.util.getf(pr, 'algorithms', {}))});
    if isfield(pr, 'selected'), dev.selected = pr.selected; end
    dev.boresight = st.v3('gdn_payload_boresight'); dev.sun_axis = st.v3('gdn_sun_axis');
    if isfield(pr, 'payload_boresight_body'), dev.boresight = unit_(pr.payload_boresight_body); end
    if isfield(pr, 'sun_axis_body'), dev.sun_axis = unit_(pr.sun_axis_body); end
    dev.rotor_tlm_noise = st.get('act_rotor_tlm_noise'); dev.gimbal_tlm_noise = st.get('act_gimbal_tlm_noise');
    % every device, not fitted, its record zero (the engine's Dev::default)
    dev.gyro = struct('fitted', false, 'rec', asils.models.gyro.GyroDesc_zero());
    dev.mag = struct('fitted', false, 'rec', asils.models.mag.MagDesc_zero());
    dev.sun = struct('fitted', false, 'rec', asils.models.finesun.SunDesc_zero());
    dev.css = struct('fitted', false, 'rec', asils.models.css.CssDesc_zero());
    dev.st = struct('fitted', false, 'rec', asils.models.sttracker.StDesc_zero(), 'nh', 0, 'rate_hz', 0, 'model', 1, ...
                    'cam', asils.models.strender.StCamera_zero());
    dev.es = struct('fitted', false, 'rec', asils.models.earthsensor.EsDesc_zero(), 'rate_hz', 0);
    dev.gps = struct('fitted', false, 'pos_sigma', 0, 'vel_sigma', 0, 'rate_hz', 0, 'latency', 0);
    dev.mtq = struct('fitted', false, 'rec', asils.models.coilset.CoilDesc_zero());
    dev.rcs = struct('fitted', false, 'rec', asils.models.thrusters.ThrusterDesc_zero());
    x = asils.models.rotorset.RotorDesc_zero();
    x.torque_noise = st.get('act_rw_torque_noise'); x.friction_comp = st.get('act_rw_friction_comp');
    x.eta = st.get('act_rw_drive_efficiency'); x.k_speed = st.get('act_cmg_speed_gain');
    x.k_flow = st.get('act_fmr_flow_gain'); x.flow_tau = st.get('act_fmr_flow_tau');
    dev.imbalance = zeros(0, 3);          % each rotor's [static kg m, dynamic kg m^2, known] (pnt's rotor_imbalance)
    dev.rotor_part = {};
    dev.files = {pf};
    ms = zeros(0, 1); ps = ms; vs = ms; items = {};
    for i = 1:numel(fills)
        f = fills{i};
        part = asils.util.getf(f, 'part', '');
        partf = asils.product.find(R, 'parts', part);
        pj = asils.util.readjson(partf);
        dev.files{end+1} = partf;
        P = struct('f', f, 'nm', asils.util.getf(pj, 'nominal', struct()), 'ds', asils.util.getf(pj, 'dispersion', struct()), ...
                   'missing', {{}}, 'part', part);
        slot = asils.util.getf(f, 'slot', '');
        capacity_(dev.id, part, slot, f, P, x);
        selector_(dev.id, slot, f);
        n0 = x.n;
        [dev, x, P, done] = fit_actuator_(dev, x, slot, P);
        if done
            % the imbalance of every rotor this slot added, as the jitter takes it (gp_4's rotor_imbalance)
            [us, ud, known] = asils.models.jitter.rotor_imbalance(strcmp(slot, 'rings'), stated_(P.nm, 'static_imbalance_kgm'), ...
                                                                   stated_(P.nm, 'dynamic_imbalance_kgm2'));
            for k = n0 + 1:x.n
                dev.imbalance(end+1, :) = [us, ud, known];
                dev.rotor_part{end+1} = part;
            end
        else
            [dev, P, done] = fit_sensor_(dev, slot, P);
        end
        if ~done, error('asils:product:refused', 'product %s: unknown slot %s', dev.id, slot); end
        if ~isempty(P.missing)
            error('asils:product:refused', 'part %s (%s of product %s) does not state %s, which the twin needs', ...
                  part, slot, dev.id, strjoin(P.missing, ', '));
        end
        % the ADCS's mass, power and volume, per fill (design/sizebudget.pc)
        nm = P.nm;
        [nu, m, p, v] = asils.models.sizebudget.budget_line(slot_of_(slot), count_(f, 'axes_body'), count_(f, 'spin_axes_body'), ...
            count_(f, 'boresights_body'), count_(f, 'normals_body'), stated_(nm, 'mass_kg'), stated_(nm, 'power_steady_W'), ...
            stated_(nm, 'power_W'), stated_(nm, 'power_at_max_W'), stated_(nm, 'volume_L'));
        items{end+1} = struct('slot', slot, 'part', part, 'n', nu, 'mass_kg', m, 'power_W', p, 'volume_L', v); %#ok<AGROW>
        ms(end+1, 1) = m; ps(end+1, 1) = p; vs(end+1, 1) = v; %#ok<AGROW>
    end
    [bm, bp, bv] = asils.models.sizebudget.budget_total(ms, ps, vs, numel(ms));
    dev.budget = struct('mass_kg', bm, 'power_W', bp, 'volume_L', bv, 'items', {items});
    dev.mex = struct('rec', x, 'n', x.n, 'ng', x.ng, 'fitted', x.n > 0);
end

% ---------------- the slots ----------------
function [dev, x, P, done] = fit_actuator_(dev, x, slot, P)
    done = true;
    switch slot
        case 'coils'
            a = axes_(P, 'axes_body');
            c = asils.models.coilset.CoilDesc_zero();
            c.fitted = true; c.n = size(a, 1); [c.m_max, P] = n_(P, 'dipole_max_Am2'); [c.p_max, P] = n_(P, 'power_at_max_W');
            c.scale_sigma = sig_(P, 'dipole_scale'); c.misalign = sig_(P, 'axis_misalignment_rad'); [c.tau, P] = nonneg_(P, 'time_constant_s');
            c.axes(1:size(a, 1), :) = a;
            dev.mtq = struct('fitted', true, 'rec', c);
        case 'wheels'
            [flo, fhi] = lohi_(P.ds, 'friction_scale', 1.0);
            % the motor's torque-speed line from its constants: stall torque k_t V / R, no-load speed V / k_t
            [kt, P] = pos_(P, 'motor_kt_Nm_per_A'); [rw, P] = pos_(P, 'motor_resistance_ohm'); [vb, P] = pos_(P, 'bus_voltage_V');
            [wmax, P] = pos_(P, 'speed_max_rad_s'); [fst, P] = pos_(P, 'friction_static_Nm'); [wst, P] = pos_(P, 'stribeck_speed_rad_s');
            [cou, P] = n_(P, 'friction_coulomb_Nm');
            if fst < cou, P.missing{end+1} = sprintf('friction_static_Nm of at least friction_coulomb_Nm (it states %g)', fst); end
            a = axes_(P, 'axes_body');
            for k = 1:size(a, 1)
                [hm, P] = n_(P, 'h_max_Nms'); [tm, P] = n_(P, 'torque_max_Nm'); [jr, P] = n_(P, 'rotor_inertia_kgm2');
                [cc, P] = n_(P, 'friction_coulomb_Nm'); [vi, P] = n_(P, 'friction_viscous_Nms'); [pst, P] = n_(P, 'power_steady_W');
                x = add_rotor_(x, 0, a(k, :), 0, hm, tm, jr, cc, vi, pst, sig_(P, 'torque_scale'), flo, fhi, sig_(P, 'axis_misalignment_rad'));
                i = x.n;
                % act's l3_rw_row_01: the motor's stall torque and no-load speed
                [ts, wn] = asils.models.wheelmotor.motor_constants(kt, rw, vb);
                x.speed_max(i) = wmax; x.t_stall(i) = ts; x.w_nl(i) = wn; x.f_static(i) = fst; x.w_stribeck(i) = wst;
            end
        case 'rings'
            % act's l3_fmr_row_12: the bore's area, the momentum per flow speed, the spin-down time, the capacity
            [s, P] = n_(P, 'enclosed_area_m2'); [bore, P] = n_(P, 'bore_m'); [rho, P] = n_(P, 'fluid_density_kg_m3');
            [mu, P] = n_(P, 'fluid_viscosity_Pa_s'); [vmax, P] = n_(P, 'v_max_m_s');
            [ac, k_hv, tsd, hmax] = asils.models.ringlimits.ring_constants(bore, s, rho, mu, vmax);
            [flo, fhi] = lohi_(P.ds, 'friction_scale', 1.0);
            [elo, ehi] = lohi_(P.ds, 'pump_efficiency', 1.0);
            fp = getnum_(P.nm, 'field_power_W', 0.0);
            a = axes_(P, 'axes_body');
            for k = 1:size(a, 1)
                % an electromagnetic pump designed by adcs-design states its pressure-limited torque
                tq = getnum_(P.nm, 'pump_torque_max_Nm', NaN);
                x = add_rotor_(x, 1, a(k, :), 0, hmax, asils.models.ringlimits.ring_pump_max(tq, hmax, tsd), k_hv, 0, 0, 0, 0, flo, fhi, ...
                               sig_(P, 'axis_misalignment_rad'));
                i = x.n;
                [cl, P] = n_(P, 'channel_length_m');
                x.t_sd(i) = tsd; x.k_hv(i) = k_hv; x.ac(i) = ac; x.s(i) = s; x.l(i) = cl;
                % act's l3_fmr_row_10: the flow sensor's noise in momentum
                x.flow_noise_h(i) = asils.models.ringnoise.ring_flow_noise(k_hv, sig_(P, 'flow_sensor_noise_m_s'));
                if isnan(fp), x.field_power(i) = 0.0; else, x.field_power(i) = fp; end
                x.eta_lo(i) = elo; x.eta_hi(i) = ehi;
            end
        case {'cmg', 'vscmg'}
            g = axes_(P, 'gimbal_axes_body');
            a = axes_(P, 'spin_axes_body');
            g0 = x.ng;
            for j = 1:size(g, 1), x.g(g0 + j, :) = g(j, :); end
            x.ng = x.ng + size(g, 1);
            [flo, fhi] = lohi_(P.ds, 'friction_scale', 1.0);
            for k = 1:size(a, 1)
                if strcmp(slot, 'cmg')
                    [hm, P] = n_(P, 'rotor_momentum_Nms'); [tm, P] = n_(P, 'rotor_torque_max_Nm'); [jr, P] = n_(P, 'rotor_inertia_kgm2');
                    [pst, P] = n_(P, 'power_steady_W');
                    x = add_rotor_(x, 2, a(k, :), g0 + k, hm, tm, jr, 0, 0, pst, sig_(P, 'torque_scale'), 1, 1, sig_(P, 'axis_misalignment_rad'));
                else
                    [hm, P] = n_(P, 'h_max_Nms'); [tm, P] = n_(P, 'rotor_torque_max_Nm'); [jr, P] = n_(P, 'rotor_inertia_kgm2');
                    [cc, P] = n_(P, 'friction_coulomb_Nm'); [vi, P] = n_(P, 'friction_viscous_Nms'); [pst, P] = n_(P, 'power_steady_W');
                    x = add_rotor_(x, 3, a(k, :), g0 + k, hm, tm, jr, cc, vi, pst, sig_(P, 'torque_scale'), flo, fhi, sig_(P, 'axis_misalignment_rad'));
                end
                [x.h0(x.n), P] = n_(P, 'rotor_momentum_Nms');
            end
            [x.gimbal_rate_max, P] = n_(P, 'gimbal_rate_max_rad_s');
            [x.gimbal_power, P] = n_(P, 'gimbal_power_W');
        case 'rcs'
            [fth, P] = n_(P, 'thrust_N'); [as, P] = n_(P, 'arm_short_m'); [al, P] = n_(P, 'arm_long_m');
            arm = asils.models.rcstorque.couple_arms(as, al);           % act's l3_rcs_row_01
            r = asils.models.thrusters.ThrusterDesc_zero();
            r.fitted = true; r.nc = 6; r.thrust = fth; [r.isp, P] = n_(P, 'isp_s'); [r.mib, P] = n_(P, 'mib_s');
            [r.res, P] = n_(P, 'valve_res_s'); [r.prop_kg, P] = n_(P, 'propellant_kg'); [r.valve_power, P] = n_(P, 'valve_power_W');
            r.thrust_sigma = sig_(P, 'thrust_scale'); r.misalign = sig_(P, 'axis_misalignment_rad');
            [isp, P] = n_(P, 'isp_s');
            [r.isp_lo, r.isp_hi] = lohi_(P.ds, 'isp_s', isp);
            r.tau = asils.models.rcstorque.couple_torques(fth, arm);    % act's l3_rcs_row_01: +-2 F arm about each axis
            dev.rcs = struct('fitted', true, 'rec', r);
        otherwise
            done = false;
    end
end

function [dev, P, done] = fit_sensor_(dev, slot, P)
    done = true; f = P.f;
    switch slot
        case 'star_tracker'
            if isfield(f, 'boresights_body'), bs = axes_(P, 'boresights_body'); else, bs = axes_(P, 'boresight_body'); end
            s = asils.models.sttracker.StDesc_zero();
            s.nh = size(bs, 1); s.bs(1:s.nh, :) = bs;
            [s.noise_cross, P] = n_(P, 'noise_cross_rad'); [s.noise_roll, P] = n_(P, 'noise_roll_rad'); [rate, P] = n_(P, 'rate_Hz');
            [s.latency, P] = n_(P, 'latency_s'); [s.max_rate, P] = n_(P, 'max_rate_rad_s'); [s.sun_excl, P] = n_(P, 'sun_exclusion_rad');
            [s.earth_excl, P] = n_(P, 'earth_exclusion_rad'); [s.fov, P] = n_(P, 'fov_half_angle_rad');
            switch asils.util.getf(f, 'model', 'quest')
                case 'noise', model = 0;
                case 'image', model = 2;
                otherwise, model = 1;
            end
            s.model = model;                                       % sttracker's StModel: noise, quest, image
            s.bias_sigma = sig_(P, 'bias_rad'); s.misalign_sigma = sig_(P, 'axis_misalignment_rad');
            [s.moon_excl, P] = nonneg_(P, 'moon_exclusion_rad'); [s.blind_s, P] = nonneg_(P, 'blind_recovery_s');
            [s.noise_rate_ref, P] = pos_(P, 'noise_doubling_rate_rad_s');
            % sens_star_tracker: a tracker calibrated on ground flies its residual as its bias, and no misalignment
            cal = getnum_(f, 'calibrated_residual_rad', NaN);
            [s.bias_sigma, s.misalign_sigma] = asils.models.sttracker.st_calibrated(s.bias_sigma, s.misalign_sigma, ~isnan(cal), ...
                                                                                    nan2zero_(cal));
            cam = asils.models.strender.StCamera_zero();
            if model == 2
                % the rendered-frame chain: the camera and the onboard chain's settings, from the part
                [npx, P] = whole_(P, 'detector_px', 16, 8192);
                [cam.psf_px, P] = pos_(P, 'psf_sigma_px'); [cam.flux0, P] = pos_(P, 'flux_mag6_e'); [cam.bg, P] = nonneg_(P, 'background_e');
                [cam.read_noise, P] = nonneg_(P, 'read_noise_e'); [cam.k_sigma, P] = pos_(P, 'centroid_k_sigma');
                [cam.max_spots, P] = whole_(P, 'max_spots', 3, 32); [cam.id_tol, P] = pos_(P, 'id_tol_rad');
                [cam.mag_tol, P] = pos_(P, 'id_mag_tol'); [cam.fit_tol, P] = pos_(P, 'fit_tol_rad');
                cam.n = npx; cam.fov = s.fov;
                [cam.f, cam.c] = asils.models.strender.st_focal(s.fov, npx);
            end
            dev.st = struct('fitted', true, 'rec', s, 'nh', s.nh, 'rate_hz', rate, 'model', model, 'cam', cam);
        case 'magnetometer'
            m = asils.models.mag.MagDesc_zero();
            [m.noise, P] = n_(P, 'noise_T_rms'); [m.bias_t, P] = n_(P, 'bias_T'); m.bias_sigma = sig_(P, 'bias_T');
            [m.range, P] = n_(P, 'range_T'); m.sf_sigma = sig_(P, 'scale_factor'); m.misalign = sig_(P, 'axis_misalignment_rad');
            [m.k_coil, P] = n_(P, 'coil_coupling_T_per_Am2');
            dev.mag = struct('fitted', true, 'rec', m);
        case 'sun_sensors'
            a = axes_(P, 'normals_body');
            s = asils.models.finesun.SunDesc_zero();
            s.n = size(a, 1); [s.noise, P] = n_(P, 'accuracy_rad'); [s.fov, P] = n_(P, 'fov_half_angle_rad'); s.bias_sigma = sig_(P, 'bias_rad');
            if strcmp(asils.util.getf(f, 'level', 'model'), 'chain')
                % quadrant currents -> angles: the head's aperture, height, current noise and threshold
                s.chain = true;
                [s.head.a, P] = pos_(P, 'aperture_side_m'); [s.head.h, P] = pos_(P, 'aperture_height_m');
                [s.head.noise, P] = nonneg_(P, 'current_noise_frac'); [s.head.min_frac, P] = pos_(P, 'current_min_frac');
            end
            s.normals(1:s.n, :) = a;
            dev.sun = struct('fitted', true, 'rec', s);
        case 'coarse_sun_sensors'
            a = axes_(P, 'normals_body');
            s = asils.models.css.CssDesc_zero();
            s.n = size(a, 1); [s.noise, P] = n_(P, 'noise_frac'); [s.albedo, P] = n_(P, 'albedo');
            s.scale_sigma = sig_(P, 'scale'); s.misalign = sig_(P, 'axis_misalignment_rad');
            s.normals(1:s.n, :) = a;
            dev.css = struct('fitted', true, 'rec', s);
        case 'gyro'
            g = asils.models.gyro.GyroDesc_zero();
            [g.arw, P] = n_(P, 'arw_rad_per_sqrt_s'); [g.rrw, P] = n_(P, 'rrw_rad_per_s_sqrt_s'); [g.range, P] = n_(P, 'range_rad_s');
            g.bias_sigma = sig_(P, 'bias_rad_s'); g.sf_sigma = sig_(P, 'scale_factor'); g.misalign = sig_(P, 'axis_misalignment_rad');
            dev.gyro = struct('fitted', true, 'rec', g);
        case 'earth_sensor'
            % l3_sens_row_13: the boresight the product states, else the payload's
            if isfield(f, 'boresight_body'), sb = unit_(f.boresight_body); stated = true; else, sb = zeros(3, 1); stated = false; end
            e = asils.models.earthsensor.EsDesc_zero();
            e.bs = asils.models.earthsensor.es_boresight(stated, sb, dev.boresight);
            [e.noise, P] = n_(P, 'accuracy_rad'); [e.fov, P] = n_(P, 'fov_half_angle_rad'); [rate, P] = n_(P, 'rate_Hz');
            e.bias_sigma = sig_(P, 'bias_rad');
            dev.es = struct('fitted', true, 'rec', e, 'rate_hz', rate);
        case 'gnss'
            [ps, P] = n_(P, 'pos_sigma_m'); [vs, P] = n_(P, 'vel_sigma_m_s'); [rate, P] = n_(P, 'rate_Hz'); [lat, P] = nonneg_(P, 'latency_s');
            dev.gps = struct('fitted', true, 'pos_sigma', ps, 'vel_sigma', vs, 'rate_hz', rate, 'latency', lat);
        otherwise
            done = false;
    end
end

function x = add_rotor_(x, kind, a, gi, hmax, tmax, j, cou, vis, pst, tsig, flo, fhi, mis)
% one rotor added to the momentum-device set (kind: rotorset's RotorKind, 0 wheel, 1 ring, 2 CMG, 3 VSCMG)
    x.n = x.n + 1; i = x.n;
    x.kind(i) = kind; x.a0(i, :) = a; x.gi(i) = gi; x.h_max(i) = hmax; x.torque_max(i) = tmax; x.jrot(i) = j;
    x.coulomb(i) = cou; x.viscous(i) = vis; x.p_steady(i) = pst; x.tsig(i) = tsig; x.flo(i) = flo; x.fhi(i) = fhi; x.misalign(i) = mis;
    x.t_sd(i) = Inf; x.k_hv(i) = 1.0; x.ac(i) = 1.0; x.s(i) = 1.0; x.l(i) = 1.0; x.eta_lo(i) = 1.0; x.eta_hi(i) = 1.0;
end

% ---------------- what the engine holds, refused by name ----------------
function capacity_(id, part, slot, f, P, x)
    cnt = @(key) count0_(f, key);
    over = @(what, have, cap) assert(have <= cap, 'asils:product:refused', 'product %s: %d %s; the twin holds at most %d', id, have, what, cap);
    switch slot
        case 'coils', over('magnetorquer coils', cnt('axes_body'), 8);
        case {'wheels', 'rings'}, over('rotors', x.n + cnt('axes_body'), 8);
        case {'cmg', 'vscmg'}, over('rotors', x.n + cnt('spin_axes_body'), 8); over('gimbals', x.ng + cnt('gimbal_axes_body'), 4);
        case 'star_tracker', over('star-tracker heads', max(cnt('boresights_body'), cnt('boresight_body')), 2);
        case {'sun_sensors', 'coarse_sun_sensors'}, over('Sun sensor heads', cnt('normals_body'), 8);
        case 'rcs'
            t = getnum_(P.nm, 'thrusters', NaN);
            assert(t == 12, 'asils:product:refused', 'part %s: %g thrusters; the twin models 12 (six couples)', part, t);
    end
end

function selector_(id, slot, f)
    switch slot
        case 'star_tracker', key = 'model'; have = {'noise', 'quest', 'image'};
        case 'sun_sensors', key = 'level'; have = {'model', 'chain'};
        otherwise, return
    end
    if isfield(f, key) && ~any(strcmp(f.(key), have))
        error('asils:product:refused', 'product %s: %s %s %s; the twin models %s', id, slot, key, f.(key), strjoin(have, ', '));
    end
end

% ---------------- reading a part ----------------
function [v, P] = n_(P, k)
    v = getnum_(P.nm, k, NaN);
    if ~isfinite(v), P.missing{end+1} = k; end
end
function [v, P] = pos_(P, k)
    [v, P] = bounded_(P, k, @(x) x > 0, 'above 0');
end
function [v, P] = nonneg_(P, k)
    [v, P] = bounded_(P, k, @(x) x >= 0, '0 or more');
end
function [v, P] = bounded_(P, k, ok, what)
    v = getnum_(P.nm, k, NaN);
    if ~isfinite(v), P.missing{end+1} = k;
    elseif ~ok(v), P.missing{end+1} = sprintf('%s %s (it states %g)', k, what, v); end
end
function [v, P] = whole_(P, k, lo, hi)
    [v, P] = pos_(P, k);
    if isfinite(v) && (v ~= round(v) || v < lo || v > hi)
        P.missing{end+1} = sprintf('%s as a whole number from %g to %g (it states %g)', k, lo, hi, v);
    end
end
function s = sig_(P, k)
    s = 0.0;
    if isfield(P.ds, k) && isstruct(P.ds.(k)) && isfield(P.ds.(k), 'sigma') && isnumeric(P.ds.(k).sigma), s = P.ds.(k).sigma; end
end
function [lo, hi] = lohi_(ds, k, d)
    lo = d; hi = d;
    if isfield(ds, k) && isstruct(ds.(k))
        if isfield(ds.(k), 'lo') && isnumeric(ds.(k).lo), lo = ds.(k).lo; end
        if isfield(ds.(k), 'hi') && isnumeric(ds.(k).hi), hi = ds.(k).hi; end
    end
end
function v = getnum_(s, k, d)
    v = d;
    if isstruct(s) && isfield(s, k) && isnumeric(s.(k)) && isscalar(s.(k)), v = double(s.(k)); end
end
function v = stated_(s, k)
    v = getnum_(s, k, NaN);
end
function x = nan2zero_(x)
    if isnan(x), x = 0; end
end
function A = axes_(P, k)
% a list of 3-vectors ([[..],[..]] or a single [x,y,z]), each made unit: one row each
    A = zeros(0, 3);
    if ~isfield(P.f, k), return, end
    a = P.f.(k);
    if iscell(a), a = cell2mat(cellfun(@(x) reshape(x, 1, []), a, 'UniformOutput', false)); end
    if isvector(a) && numel(a) == 3, a = reshape(a, 1, 3); end
    for i = 1:size(a, 1)
        v = a(i, :); n = sqrt(v(1)*v(1) + v(2)*v(2) + v(3)*v(3));
        A(i, :) = [v(1)/n, v(2)/n, v(3)/n];
    end
end
function u = unit_(a)
    n = sqrt(a(1)*a(1) + a(2)*a(2) + a(3)*a(3)); u = [a(1)/n; a(2)/n; a(3)/n];
end
function n = count0_(f, key)
    n = 0;
    if ~isfield(f, key), return, end
    a = f.(key);
    if iscell(a), n = numel(a); elseif isvector(a) && numel(a) == 3, n = 1; else, n = size(a, 1); end
end
function n = count_(f, key)
% the length of a list the fill states (-1 where it states none), as adcs-design's budget counts it
    n = -1;
    if ~isfield(f, key), return, end
    a = f.(key);
    if iscell(a) || iscolumn(a), n = numel(a); else, n = size(a, 1); end
end
function s = slot_of_(slot)
% sizebudget's Slot (design/sizebudget.pc): the fill's word as the choice's option number
    names = {'coils', 'wheels', 'rings', 'cmg', 'vscmg', 'rcs', 'star_tracker', 'magnetometer', 'sun_sensors', 'gyro', 'gnss', ...
             'earth_sensor', 'coarse_sun_sensors'};
    s = find(strcmp(names, slot), 1) - 1;
    if isempty(s), s = 13; end
end
function c = cellstr_(a)
    if iscell(a), c = a; elseif ischar(a), c = {a}; else, c = {}; end
end
