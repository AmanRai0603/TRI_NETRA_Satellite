function dev = load(productId)
%ASILS.PRODUCT.LOAD  A catalogue product -> the device parameter block P.dev.
%   Reads data/products/<id>.json and every part it fills from data/parts/.
%   A slot the product does not fill is marked fitted = false.
%
%   Momentum-exchange devices of every kind (reaction wheels, fluid rings,
%   CMG and VSCMG rotors) are gathered into ONE block, dev.mex, one entry per
%   rotor, because the plant treats them alike (asils.plant.deriv):
%     dev.mex.kind{i}  'rw' | 'fmr' | 'cmg' | 'vscmg'
%     dev.mex.A0       spin axes at zero gimbal angle (3 x nr)
%     dev.mex.G, gi    gimbal axes (3 x ng) and each rotor's gimbal index
    R = asils.util.root();
    pr = asils.util.readjson(find_(R, 'products', productId));
    fills = pr.fill; if ~iscell(fills), fills = num2cell(fills); end
    dev = struct('id', pr.id, 'label', pr.label, 'family', pr.family, 'algorithms', {pr.algorithms});
    if isfield(pr, 'selected'), dev.selected = pr.selected; end     % promoted outcome of a trade
    dev.boresight = [0;1;0];
    if isfield(pr, 'payload_boresight_body'), dev.boresight = pr.payload_boresight_body(:)/norm(pr.payload_boresight_body); end
    dev.sun_axis = [0;0;-1];                  % power face (Sun referencing / acquisition), Standard Code -Z
    if isfield(pr, 'sun_axis_body'), dev.sun_axis = pr.sun_axis_body(:)/norm(pr.sun_axis_body); end
    dev.budget = struct('mass_kg', 0, 'power_W', 0, 'volume_L', 0, 'items', {{}});
    dev.es.fitted = false;
    dev.gyro.fitted = false; dev.mag.fitted = false; dev.sun.fitted = false; dev.css.fitted = false;
    dev.st.fitted = false; dev.mtq.fitted = false; dev.gps.fitted = false; dev.rcs.fitted = false;
    X = mex_empty_();
    for i = 1:numel(fills)
        f = fills{i};
        p = asils.util.readjson(find_(R, 'parts', f.part));
        nm = p.nominal; ds = asils.util.getf(p, 'dispersion', struct());
        sig = @(key, d) sigma_(ds, key, d);
        dev.budget = budget_(dev.budget, f, nm);
        switch f.slot
            case 'coils'
                dev.mtq = struct('fitted', true, 'part', f.part, 'axes', axes_(f.axes_body), ...
                    'm_max', nm.dipole_max_Am2, 'p_max', nm.power_at_max_W, ...
                    'scale_sigma', sig('dipole_scale', 0), 'misalign_rad', sig('axis_misalignment_rad', 0));
            case 'wheels'
                A = axes_(f.axes_body); n = size(A, 2);
                for k = 1:n
                    X = add_(X, 'rw', f.part, A(:,k), 0, nm.h_max_Nms, nm.torque_max_Nm, nm.rotor_inertia_kgm2, ...
                        nm.friction_coulomb_Nm, nm.friction_viscous_Nms, nm.power_steady_W, sig('torque_scale', 0), ...
                        ds.friction_scale.lo, ds.friction_scale.hi, sig('axis_misalignment_rad', 0));
                end
            case 'rings'                      % fluid momentum rings (IDMAS magneto-fluidic panel)
                A = axes_(f.axes_body); n = size(A, 2);
                Ac = pi*nm.bore_m^2/4; S = nm.enclosed_area_m2; rho = nm.fluid_density_kg_m3;
                k_hv = rho*Ac*2*S;                               % h = k_hv * v   [N m s per m/s]
                Tsd = rho*nm.bore_m^2/(32*nm.fluid_viscosity_Pa_s);   % laminar spin-down time
                hmax = k_hv*nm.v_max_m_s;
                for k = 1:n
                    X = add_(X, 'fmr', f.part, A(:,k), 0, hmax, 2*hmax/Tsd, k_hv, 0, 0, 0, 0, ...
                        ds.friction_scale.lo, ds.friction_scale.hi, sig('axis_misalignment_rad', 0));
                    X.T_sd(end) = Tsd; X.k_hv(end) = k_hv; X.Ac(end) = Ac; X.S(end) = S; X.l(end) = nm.channel_length_m;
                    X.flow_noise_h(end) = k_hv*sig('flow_sensor_noise_m_s', 0);
                    fp = asils.util.getf(nm, 'field_power_W', 0); if isnan(fp), fp = 0; end
                    X.field_power(end) = fp;
                    X.eta_lo(end) = ds.pump_efficiency.lo; X.eta_hi(end) = ds.pump_efficiency.hi;
                end
                dev.fmr_cruise_h = k_hv*nm.v_cruise_m_s;
            case {'cmg', 'vscmg'}
                G = axes_(f.gimbal_axes_body); A = axes_(f.spin_axes_body); n = size(A, 2);
                g0 = size(X.G, 2);
                X.G = [X.G, G];
                for k = 1:n
                    if strcmp(f.slot, 'cmg')
                        X = add_(X, 'cmg', f.part, A(:,k), g0 + k, nm.rotor_momentum_Nms, nm.rotor_torque_max_Nm, ...
                            nm.rotor_inertia_kgm2, 0, 0, nm.power_steady_W, sig('torque_scale', 0), 1, 1, sig('axis_misalignment_rad', 0));
                    else
                        X = add_(X, 'vscmg', f.part, A(:,k), g0 + k, nm.h_max_Nms, nm.rotor_torque_max_Nm, ...
                            nm.rotor_inertia_kgm2, nm.friction_coulomb_Nm, nm.friction_viscous_Nms, nm.power_steady_W, ...
                            sig('torque_scale', 0), ds.friction_scale.lo, ds.friction_scale.hi, sig('axis_misalignment_rad', 0));
                    end
                    X.h0(end) = nm.rotor_momentum_Nms;
                end
                X.gimbal_rate_max = nm.gimbal_rate_max_rad_s; X.gimbal_power = nm.gimbal_power_W;
            case 'rcs'
                F = nm.thrust_N; a = [nm.arm_short_m, nm.arm_long_m, nm.arm_long_m];
                T = zeros(3, 6);
                for ax = 1:3, T(ax, 2*ax-1) = 2*F*a(ax); T(ax, 2*ax) = -2*F*a(ax); end
                dev.rcs = struct('fitted', true, 'part', f.part, 'tau_couple', T, 'thrust_N', F, ...
                    'isp_s', nm.isp_s, 'mib_s', nm.mib_s, 'valve_res_s', nm.valve_res_s, ...
                    'propellant_kg', nm.propellant_kg, 'valve_power_W', nm.valve_power_W, ...
                    'isp_lo', lohi_(ds, 'isp_s', nm.isp_s, 1), 'isp_hi', lohi_(ds, 'isp_s', nm.isp_s, 2), ...
                    'thrust_sigma', sig('thrust_scale', 0), 'misalign_rad', sig('axis_misalignment_rad', 0));
            case 'star_tracker'
                if isfield(f, 'boresights_body'), bs = axes_(f.boresights_body); else, bs = axes_(f.boresight_body(:)'); end
                dev.st = struct('fitted', true, 'part', f.part, 'boresight', bs, ...
                    'noise_cross', nm.noise_cross_rad, 'noise_roll', nm.noise_roll_rad, ...
                    'rate_hz', nm.rate_Hz, 'latency', nm.latency_s, 'max_rate', nm.max_rate_rad_s, ...
                    'sun_excl', nm.sun_exclusion_rad, 'earth_excl', nm.earth_exclusion_rad, ...
                    'fov', nm.fov_half_angle_rad, 'model', asils.util.getf(f, 'model', 'quest'), ...
                    'bias_sigma', sig('bias_rad', 0), 'misalign_sigma', sig('axis_misalignment_rad', 0), ...
                    'power', nm.power_W);
                if isfield(f, 'calibrated_residual_rad')     % in-orbit alignment calibration
                    dev.st.bias_sigma = f.calibrated_residual_rad; dev.st.misalign_sigma = 0;
                end
            case 'magnetometer'
                dev.mag = struct('fitted', true, 'part', f.part, 'noise', nm.noise_T_rms, ...
                    'bias_T', nm.bias_T, 'bias_sigma', sig('bias_T', 0), 'range', nm.range_T, ...
                    'rate_hz', nm.rate_Hz, 'sf_sigma', sig('scale_factor', 0), ...
                    'misalign_rad', sig('axis_misalignment_rad', 0), 'k_coil', 5e-6);
            case 'sun_sensors'
                dev.sun = struct('fitted', true, 'part', f.part, 'normals', axes_(f.normals_body), ...
                    'noise', nm.accuracy_rad, 'fov_rad', nm.fov_half_angle_rad, 'rate_hz', nm.rate_Hz, ...
                    'bias_sigma', sig('bias_rad', 0));
            case 'coarse_sun_sensors'
                dev.css = struct('fitted', true, 'part', f.part, 'normals', axes_(f.normals_body), ...
                    'noise', nm.noise_frac, 'albedo', nm.albedo, 'fov_rad', nm.fov_half_angle_rad, ...
                    'scale_sigma', sig('scale', 0), 'misalign_rad', sig('axis_misalignment_rad', 0));
            case 'gyro'
                dev.gyro = struct('fitted', true, 'part', f.part, 'arw', nm.arw_rad_per_sqrt_s, ...
                    'rrw', nm.rrw_rad_per_s_sqrt_s, 'range', nm.range_rad_s, 'rate_hz', nm.rate_Hz, ...
                    'bias_sigma', sig('bias_rad_s', 0), 'sf_sigma', sig('scale_factor', 0), ...
                    'misalign_rad', sig('axis_misalignment_rad', 0));
            case 'earth_sensor'
                bsv = dev.boresight; if isfield(f, 'boresight_body'), bsv = f.boresight_body(:)/norm(f.boresight_body); end
                dev.es = struct('fitted', true, 'part', f.part, 'boresight', bsv, 'noise', nm.accuracy_rad, ...
                    'fov_rad', nm.fov_half_angle_rad, 'rate_hz', nm.rate_Hz, 'bias_sigma', sig('bias_rad', 0));
            case 'gnss'
                dev.gps = struct('fitted', true, 'part', f.part, 'pos_sigma', nm.pos_sigma_m, ...
                    'vel_sigma', nm.vel_sigma_m_s, 'rate_hz', nm.rate_Hz);
        end
    end
    X.fitted = ~isempty(X.kind);
    X.Us = zeros(1, numel(X.kind)); X.Ud = X.Us;       % rotor imbalance (jitter, asils.sizing.jitter)
    k = 0;
    for i = 1:numel(fills)
        f = fills{i};
        if ~any(strcmp(f.slot, {'wheels', 'rings', 'cmg', 'vscmg'})), continue, end
        pj = asils.util.readjson(find_(R, 'parts', f.part)); nm = pj.nominal;
        if isfield(f, 'spin_axes_body'), a = f.spin_axes_body; else, a = f.axes_body; end
        if iscell(a), n = numel(a); elseif isvector(a), n = 1; else, n = size(a, 1); end
        X.Us(k+1:k+n) = asils.util.getf(nm, 'static_imbalance_kgm', 0);
        X.Ud(k+1:k+n) = asils.util.getf(nm, 'dynamic_imbalance_kgm2', 0);
        k = k + n;
    end
    dev.mex = X;
    dev.rw.fitted = X.fitted;          % legacy name: "has momentum-exchange devices"
end

function f = find_(R, kind, id)
%FIND_  A catalogue record (data/<kind>/<id>.json), or a case-sized one written
%   by asils.sizing (store/sized/<case>/<kind>/<id>.json).
    f = fullfile(R, 'data', kind, [id '.json']);
    if exist(f, 'file') == 2, return, end
    d = dir(fullfile(R, 'store', 'sized'));
    for i = 1:numel(d)
        g = fullfile(R, 'store', 'sized', d(i).name, kind, [id '.json']);
        if d(i).isdir && exist(g, 'file') == 2, f = g; return, end
    end
    error('asils:product:missing', 'No %s record %s (data/%s or store/sized/*/%s)', kind(1:end-1), id, kind, kind);
end

function X = mex_empty_()
    X = struct('kind', {{}}, 'part', {{}}, 'A0', zeros(3,0), 'G', zeros(3,0), 'gi', zeros(1,0), ...
        'h_max', [], 'torque_max', [], 'J', [], 'coulomb', [], 'viscous', [], 'p_steady', [], ...
        'torque_scale_sigma', [], 'friction_scale_lo', [], 'friction_scale_hi', [], 'misalign_rad', [], ...
        'T_sd', [], 'k_hv', [], 'Ac', [], 'S', [], 'l', [], 'flow_noise_h', [], 'field_power', [], 'eta_lo', [], 'eta_hi', [], 'h0', [], ...
        'torque_noise', 0.001, 'friction_comp', 0.95, 'eta', 0.8, 'k_speed', 1.0, 'k_flow', 2.0, 'flow_tau', 0.3, ...
        'gimbal_rate_max', 0, 'gimbal_power', 0);
end

function X = add_(X, kind, part, a, gi, hmax, tmax, J, cou, vis, pst, tsig, flo, fhi, mis)
    X.kind{end+1} = kind; X.part{end+1} = part; X.A0(:, end+1) = a; X.gi(end+1) = gi;
    X.h_max(end+1) = hmax; X.torque_max(end+1) = tmax; X.J(end+1) = J; X.coulomb(end+1) = cou;
    X.viscous(end+1) = vis; X.p_steady(end+1) = pst; X.torque_scale_sigma(end+1) = tsig;
    X.friction_scale_lo(end+1) = flo; X.friction_scale_hi(end+1) = fhi; X.misalign_rad(end+1) = mis;
    X.T_sd(end+1) = Inf; X.k_hv(end+1) = 1; X.Ac(end+1) = 1; X.S(end+1) = 1; X.l(end+1) = 1;
    X.flow_noise_h(end+1) = 0; X.field_power(end+1) = 0; X.eta_lo(end+1) = 1; X.eta_hi(end+1) = 1; X.h0(end+1) = 0;
end

function A = axes_(a)
    if iscell(a), a = cell2mat(cellfun(@(x) x(:)', a, 'UniformOutput', false)); end
    A = a';                                   % 3 x n
    for j = 1:size(A,2), A(:,j) = A(:,j)/norm(A(:,j)); end
end

function B = budget_(B, f, nm)
%BUDGET_  ADCS mass / nominal power / volume, per fill (unit count from its axes).
    n = 1;
    for key = {'axes_body', 'spin_axes_body', 'boresights_body', 'normals_body'}
        if isfield(f, key{1})
            a = f.(key{1}); if iscell(a), n = numel(a); elseif isvector(a), n = numel(a)/3; else, n = size(a, 1); end
            if strcmp(f.slot, 'coarse_sun_sensors'), n = 1; end   % one board of cosine cells
        end
    end
    m = asils.util.getf(nm, 'mass_kg', 0);
    p = asils.util.getf(nm, 'power_steady_W', asils.util.getf(nm, 'power_W', asils.util.getf(nm, 'power_at_max_W', 0)));
    v = asils.util.getf(nm, 'volume_L', 0);
    B.mass_kg = B.mass_kg + n*m; B.power_W = B.power_W + n*p; B.volume_L = B.volume_L + n*v;
    B.items{end+1} = struct('slot', f.slot, 'part', f.part, 'n', n, 'mass_kg', n*m, 'power_W', n*p, 'volume_L', n*v);
end

function x = lohi_(ds, key, d, k)
    x = d;
    if isfield(ds, key)
        if k == 1 && isfield(ds.(key), 'lo'), x = ds.(key).lo; end
        if k == 2 && isfield(ds.(key), 'hi'), x = ds.(key).hi; end
    end
end

function s = sigma_(ds, key, d)
    s = d;
    if isfield(ds, key) && isfield(ds.(key), 'sigma'), s = ds.(key).sigma; end
end
