function P = config(scenarioId, caseFile, opts)
%ASILS.CONFIG  Build the one parameter struct P a run needs.
%   P = asils.config('nadir_hold_ais', 'cases/ais_3u.csv')
%   P = asils.config(..., struct('seed', 7, 'set', struct('orbit_step_s', 20)))
%
%   Sources, in order (a later one never restates an earlier one):
%     case CSV         mission, orbit, mass, surfaces, magnetic, requirements
%     scenario JSON    duration, steps, initial state, mode sequence, metrics
%     product JSON     fitted parts and their descriptors (asils.product.load)
%     this file        derived values (SSO geometry, gains from inertia),
%                      the few engineering defaults the case format has no
%                      key for -- each commented with its source.
    if nargin < 3, opts = struct(); end
    R = asils.util.root();
    if ~exist(caseFile, 'file'), caseFile = fullfile(R, caseFile); end
    if isstruct(scenarioId), S = scenarioId; else, S = asils.scenario.load(scenarioId); end   % a mode test builds its scenario in memory
    C = asils.case.read(caseFile);
    asils.scenario.check(S, C);             % the engine's rules: unknown keys, wrong types, missing requirements refused
    v = C.v;
    P.scenario = S; P.case = C; P.id = S.id;
    P.faults = asils.util.getf(S, 'faults', []);          % scheduled fault injection (asils.faults.apply)
    P.seed = asils.util.getf(opts, 'seed', 1);
    P.dev = asils.product.load(asils.util.getf(opts, 'product', S.product));
    P.sizing_case = asils.util.getf(opts, 'sizing_case', '');

    %% epoch: case mission.epoch = years after J2000.0
    P.epoch_utc = asils.util.jd2utc(2451545.0 + v.mission_epoch*365.25);
    P.epoch_utc(6) = round(P.epoch_utc(6));

    %% orbit (case) + in-loop propagator settings
    P.orbit.alt_km = v.orbit_alt; P.orbit.inc_deg = v.orbit_inc; P.orbit.ecc = v.orbit_ecc;
    P.orbit.ltan_h = v.orbit_ltan; P.orbit.argp_deg = 0;
    P.orbit.u0_deg = asils.util.getf(S.initial, 'arg_lat_deg', 0);
    P.orbit.step_s = 10;             % RK4 step of the POP in the loop (cm-level, see asils.orbit.state)
    P.orbit.grav_degree = 6;         % POP embedded field (J2..J6 + tesserals it carries)
    P.orbit.thirdbody = true; P.orbit.drag = true; P.orbit.srp = true;
    P.orbit.atmos = 'dtm2020';       % POP DTM2020 operational density (offline with manual indices)
    mu = 3.986004418e14; a = 6378137 + v.orbit_alt*1e3;
    P.orbit.n = sqrt(mu/a^3); P.orbit.period_s = 2*pi/P.orbit.n; P.mu = mu;

    %% space weather: epoch 2027 is past the measured record -> manual indices
    P.env.F107 = 130; P.env.F107a = 130; P.env.Kp = 2.0; P.env.ap = 7;   % declining solar cycle 25
    P.env.igrf_nmax = 13;            % truth field degree
    P.env.dt_s = 1.0;                % field / Sun / shadow / density refresh (held between)
    P.env.on = [true true true true];% [gravity-gradient aero SRP residual-dipole]

    %% spacecraft (case)
    P.sc.mass_kg = v.mass_m;
    P.sc.I = diag([v.mass_imin, v.mass_iint, v.mass_imax]);      % long axis = X_B
    P.sc.box_m = class_box(C);              % the body of the case's class (catalogue/classes.toml)
    cpa = v.surface_cpa;
    P.sc.cm_offset_m = cpa*[0.30; 0.70; -0.65]/norm([0.30; 0.70; -0.65]);   % |offset| = case CP-CM
    % a stated value nothing models is refused, never silently dropped (= the engine's CASE_UNMODELLED)
    um = {'mission_duty', 'mass_cm', 'resources_vbus', 'resources_nif'};
    for i = 1:numel(um)
        if isfield(v, um{i}) && isfinite(v.(um{i}))
            error('asils:case:refused', 'case %s states %s = %g: the models do not use it yet', C.id, strrep(um{i}, '_', '.'), v.(um{i}));
        end
    end
    % the facet model: one centre-of-mass offset for both torques, the sunlit area from the box
    if isfield(v, 'surface_cps') && isfinite(v.surface_cps) && v.surface_cps ~= cpa
        error('asils:case:refused', 'case %s: surface.cps = %g m differs from surface.cpa = %g m; the facet model has one centre-of-mass offset for both torques', C.id, v.surface_cps, cpa);
    end
    b_ = P.sc.box_m; face = max([b_(1)*b_(2), b_(2)*b_(3), b_(1)*b_(3)]);
    if isfield(v, 'surface_asun') && isfinite(v.surface_asun) && abs(v.surface_asun - face) > 1e-6*max(face, 1e-12)
        error('asils:case:refused', 'case %s: surface.asun = %g m^2, but the facet model lights the class body, largest face %g m^2', C.id, v.surface_asun, face);
    end
    P.sc.aref_m2 = v.surface_afr; P.sc.cd = v.surface_cd; P.sc.refl = v.surface_refl;
    % surface-model settings, the engine's ACCOMMODATION, VB_RATIO, SPEC_FRAC (config.rs): Moe & Moe (2005) LEO
    P.sc.sigma_n = 0.8; P.sc.sigma_t = 0.8; P.sc.vb_ratio = 0.05;
    P.sc.spec_frac = 0.5;
    P.sc.m_res = v.magnetic_dres*[1;1;1]/sqrt(3);

    %% simulation timing (scenario)
    P.sim.duration_s = S.time.duration_s;
    P.sim.dt = S.time.dt_s;
    P.sim.record_dt = S.time.record_dt_s;

    %% flight software (derived gains; tunable per scenario)
    F = struct();
    legacy = struct('nadir_rw', 'nadir_fine', 'target_rw', 'target_fine', 'slew_rw', 'slew_fine');
    F.start_mode = S.fsw.start_mode; if isfield(legacy, F.start_mode), F.start_mode = legacy.(F.start_mode); end
    F.auto_next = asils.util.getf(S.fsw, 'auto_next', ''); if isfield(legacy, F.auto_next), F.auto_next = legacy.(F.auto_next); end

    F.detumble_exit = asils.util.getf(S.fsw, 'detumble_exit_deg_s', 0.5)*pi/180;
    F.detumble_hold_s = asils.util.getf(S.fsw, 'detumble_hold_s', 60);
    F.schedule = asils.util.getf(S.fsw, 'schedule', []);           % [[fsw.schedule]] t_s, mode: commanded changes
    if isstruct(F.schedule), F.schedule = num2cell(F.schedule); end
    % thruster rate damping (detumble_rcs) and rotor Sun acquisition (sun_acq_rotor)
    F.rcsd = struct('T_damp_s', asils.util.getf(S.fsw, 'rcs_damp_s', 20), 'deadband_deg_s', 0.2, 'period_s', 1.0);
    F.sa = struct('w_max_deg_s', asils.util.getf(S.fsw, 'sun_acq_rate_deg_s', 1.0), 'kd', 0.1, ...
                  'done_deg', 10, 'done_hold_s', 60);
    F.capture_deg = asils.util.getf(S.fsw, 'capture_deg', 3.0);         % fine modes: capture manoeuvre beyond this error
    F.capture_rate_deg_s = asils.util.getf(S.fsw, 'capture_rate_deg_s', 1.0);
    F.guidance = S.fsw.guidance;
    F.guidance.q_off = asils.fsw.boresight_offset(P.dev.boresight);   % payload axis -> nadir
    F.mtq_period = 1.0; F.mtq_meas = 0.2;          % measure-then-drive duty (Standard Code bdotScheduler idea)
    Jmin = min(diag(P.sc.I));
    F.bdot_k = asils.util.getf(S.fsw, 'bdot_gain_scale', 3.0) * 2*P.orbit.n*(1 + sind(v.orbit_inc))*Jmin;
    % Sun-spin -> magnetic pointing hand-over and the Sun-state gravity-gradient feed-forward (05_control.md)
    F.ho_in = asils.util.getf(S.fsw, 'handover_in_dps', 1.0)*pi/180; F.ho_out = asils.util.getf(S.fsw, 'handover_out_dps', 0.5)*pi/180;
    F.ho_hold_s = asils.util.getf(S.fsw, 'handover_hold_s', 60.0); F.mtq_gg_ff = asils.util.getf(S.fsw, 'mtq_gg_ff', 1);
    F.gd_yaw_flip = asils.util.getf(S.fsw, 'yaw_flip', true);     % nadir family: power face towards the Sun
    I = diag(P.sc.I);
    wn = asils.util.getf(S.fsw, 'mtq_wn', 0.005); z = asils.util.getf(S.fsw, 'mtq_zeta', 2.0);   % SILS sweep (docs/RESULTS.md)
    F.mtq.Kp = I*wn^2; F.mtq.Kd = 2*z*I*wn;
    % the same bandwidth for every magnetic pointing law, so a trade compares laws, not gains
    F.mtq.err_max = 0.5; F.mtq.int_max = 0.05; th = 0.05; F.mtq.Ki = zeros(3,1);
    F.mtq.Klqr = zeros(3, 3);
    for ax = 1:3
        Q = diag([(wn/(0.5*th))^2*0, 1/th^2, 1/(wn*th)^2]) + diag([1e-12 0 0]); Rq = 1/(I(ax)*wn^2*th)^2;
        F.mtq.Klqr(ax, :) = asils.fsw.lqr_gain([0 1 0; 0 0 1; 0 0 0], [0; 0; 1/I(ax)], Q, Rq);
    end
    F.mtq.lambda = wn/(2*z)*2; F.mtq.phi = 5e-4; F.mtq.Gs = 2*z*wn*F.mtq.phi*ones(3,1);
    % Standard Code L1/L2 (cfg.control: ctl.spinup, ctl.sunSpin; theory doc sec. 4-7)
    F.ss = struct('k_l1', asils.util.getf(S.fsw, 'l1_gain', 1e6), 'spin_dps', asils.util.getf(S.fsw, 'spin_rate_dps', 6), ...
        'sigma0', 1, 'z_in_dps', 0.5, 'perp_in_dps', 0.5, 'sun_min', 0.05, 't_check_s', 60, 'omega_max_dps', 100, ...
        'dwell_in_s', 60, 'k1', 0.01, 'k2', 0.05, 'eclipse', 'E1', 'perp_out_dps', 1.0, 'omega_exit_dps', 2.0, ...
        'dwell_out_s', 30, 'detumble_exit_dps', 2.0);
    wn = asils.util.getf(S.fsw, 'rw_bandwidth', 0.9);   % SILS sweep: noise-limited optimum at the spec's 1 rad/s bound
    z = asils.util.getf(S.fsw, 'rw_damping', 2.0);   % SILS sweep: heavier damping beats 0.9 on the 3-sigma APE
    F.rw.Kp = I*wn^2; F.rw.Kd = 2*z*I*wn; F.rw.Ki = 0.15*I*wn^3;
    F.rw.err_max = 0.2; F.rw.int_max = 0.02; F.rw.dt = 1/asils.util.getf(S.fsw, 'rw_rate_hz', 10);

    % LQR on [int theta; theta; omega] per axis, Bryson weights sized to the same bandwidth
    th = 1e-3; F.rw.Klqr = zeros(3, 3);
    for ax = 1:3
        Aq = [0 1 0; 0 0 1; 0 0 0]; Bq = [0; 0; 1/I(ax)];
        Q = diag([(wn/(0.5*th))^2, 1/th^2, 1/(wn*th)^2]); Rq = 1/(I(ax)*wn^2*th)^2;
        F.rw.Klqr(ax, :) = asils.fsw.lqr_gain(Aq, Bq, Q, Rq);
    end
    % sliding mode: surface slope lambda, boundary layer phi, reaching gain
    F.rw.lambda = wn/(2*z); F.rw.phi = 2e-4; F.rw.Gs = 2*z*wn*F.rw.phi*ones(3,1);
    F.cmg = struct('lam0', 1e-9, 'mu', 10, 'k_null', 0.002);        % SR steering (Wie 2008)
    F.rcs = struct('assist', logical(asils.util.getf(S.fsw, 'rcs_assist', 1)), 'assist_frac', 0.8, ...
                   'dump', logical(asils.util.getf(S.fsw, 'rcs_dump', 1)), 'dump_hi', 4e-3, 'dump_lo', 1e-3, 'dump_k', 0.05);
    F.fdir_s = 3.0;                                  % a rotor off its command this long is isolated
    F.fdir_win_s = 120.0;                            % windowed rotor FDIR: window length
    F.fdir_h_frac = 0.005;                           % ... a commanded change under this x h_max is not judged
    F.rate_lpf_s = asils.util.getf(S.fsw, 'rate_lpf_s', 0.3);   % controller rate filter time constant
    F.dump_k = asils.util.getf(S.fsw, 'dump_gain', 2e-3); F.h_bias = asils.util.getf(S.fsw, 'wheel_bias_Nms', 2e-3);
    F.m_res_est = P.sc.m_res;       % ground-calibrated residual dipole the coils cancel (nominal case value)
    F.truth_knowledge = false;                      % debug: FSW gets the true attitude/rate
    F.mekf.meas_scale = 1; F.mekf.proc_scale = 1;
    F.mekf.sig_mag = 0.03; F.mekf.sig_sun = 0.012;  % direction 1-sigma [rad] incl. model error
    F.st_coast_s = 900;                             % gyro-only coasting allowed across a star-tracker outage
    F.gps_latency = 0;                              % age of a GNSS fix [s]: the fix is carried forward by it
    if P.dev.gps.fitted, F.gps_latency = P.dev.gps.latency; end
    F.igrf_nmax = 10;                               % onboard field model degree (truth: 13)
    F.algorithms = asils.util.getf(S.fsw, 'algorithms', struct());
    F.J = P.sc.I;                                   % the inertia the flight software was loaded with (nominal)
    P.fsw = F;

    %  The FSW's calibrated dipole (fsw.m_res_est) and inertia (fsw.J) are fixed BEFORE overrides,
    %  so a dispersed true dipole or inertia (sc.m_res, sc.I) leaves a realistic knowledge error,
    %  as on the engine (the flight software keeps the ground-calibrated values).
    if isfield(opts, 'set')
        P = asils.util.setpaths(P, opts.set);
    end
    % which algorithm fills each FSW slot, checked against the hardware
    % (a trade overrides one with set.fsw__algorithms__<slot>)
    S2 = P.scenario; S2.fsw.algorithms = P.fsw.algorithms;
    P.fsw.alg = asils.fsw.select(P.dev, S2);
    P.sc.Iinv = inv(P.sc.I);
    % the case's flexible mode on the truth body, all or none (= the engine's Config::build)
    P.sc.flex = struct('on', false);
    fk = {'flex_fmode', 'flex_mpart', 'flex_zeta', 'flex_axis'};
    st = cellfun(@(k) isfield(v, k) && isfinite(v.(k)), fk);
    if any(st) && ~all(st)
        error('asils:case:refused', 'case %s states part of its flexible mode: %s missing (all of it or none)', ...
              C.id, strjoin(strrep(fk(~st), '_', '.'), ', '));
    end
    if all(st)
        a = v.flex_axis; dl = zeros(3, 1); dl(a) = sqrt(v.flex_mpart*P.sc.I(a, a));
        P.sc.flex = struct('on', true, 'delta', dl, 'omega', 2*pi*v.flex_fmode, 'zeta', v.flex_zeta, ...
                           'Minv', inv(P.sc.I - dl*dl'));
    end
end

function b = class_box(C)
%CLASS_BOX  The body of the case's satellite class, as the engine's config::class_box: a blank or
%   unknown meta.class is refused, never assumed.
    K = asils.util.readjson(fullfile(asils.util.root(), 'data', 'classes.json'));
    L = K.class; if isstruct(L), L = num2cell(L); end
    ids = cellfun(@(x) x.id, L, 'UniformOutput', false);
    if isempty(C.class)
        error('asils:case:class', 'case %s: meta.class is blank; the twin models the body of the class it names (%s)', C.id, strjoin(ids, ', '));
    end
    k = find(strcmp(ids, C.class), 1);
    if isempty(k)
        error('asils:case:class', 'case %s: meta.class = "%s" is no class in catalogue/classes.toml (%s)', C.id, C.class, strjoin(ids, ', '));
    end
    b = reshape(L{k}.box_m, 1, 3);
end
