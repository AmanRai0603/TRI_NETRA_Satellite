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
    S = asils.scenario.load(scenarioId);
    C = asils.case.read(caseFile);
    v = C.v;
    P.scenario = S; P.case = C; P.id = S.id;
    P.seed = asils.util.getf(opts, 'seed', 1);
    P.dev = asils.product.load(asils.util.getf(opts, 'product', S.product));

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
    P.sc.box_m = [0.34 0.10 0.10];
    cpa = v.surface_cpa;
    P.sc.cm_offset_m = cpa*[0.30; 0.70; -0.65]/norm([0.30; 0.70; -0.65]);   % |offset| = case CP-CM
    P.sc.aref_m2 = v.surface_afr; P.sc.cd = v.surface_cd; P.sc.refl = v.surface_refl;
    P.sc.sigma_n = 0.8; P.sc.sigma_t = 0.8; P.sc.vb_ratio = 0.05;  % Moe & Moe (2005) LEO accommodation
    P.sc.spec_frac = 0.5;
    P.sc.m_res = v.magnetic_dres*[1;1;1]/sqrt(3);

    %% simulation timing (scenario)
    P.sim.duration_s = S.time.duration_s;
    P.sim.dt = S.time.dt_s;
    P.sim.record_dt = S.time.record_dt_s;

    %% flight software (derived gains; tunable per scenario)
    F = struct();
    F.start_mode = S.fsw.start_mode;
    F.auto_next = asils.util.getf(S.fsw, 'auto_next', '');
    F.detumble_exit = asils.util.getf(S.fsw, 'detumble_exit_deg_s', 0.5)*pi/180;
    F.detumble_hold_s = asils.util.getf(S.fsw, 'detumble_hold_s', 60);
    F.guidance = S.fsw.guidance;
    F.guidance.q_off = asils.fsw.boresight_offset(P.dev.boresight);   % payload axis -> nadir
    F.mtq_period = 1.0; F.mtq_meas = 0.2;          % measure-then-drive duty (Standard Code bdotScheduler idea)
    Jmin = min(diag(P.sc.I));
    F.bdot_k = asils.util.getf(S.fsw, 'bdot_gain_scale', 3.0) * 2*P.orbit.n*(1 + sind(v.orbit_inc))*Jmin;
    I = diag(P.sc.I);
    wn = asils.util.getf(S.fsw, 'mtq_wn', 0.005); z = asils.util.getf(S.fsw, 'mtq_zeta', 2.0);   % SILS sweep (docs/RESULTS.md)
    F.mtq.Kp = I*wn^2; F.mtq.Kd = 2*z*I*wn;
    wn = asils.util.getf(S.fsw, 'rw_bandwidth', 0.9);   % SILS sweep: noise-limited optimum at the spec's 1 rad/s bound z = asils.util.getf(S.fsw, 'rw_damping', 0.9);
    F.rw.Kp = I*wn^2; F.rw.Kd = 2*z*I*wn; F.rw.Ki = 0.15*I*wn^3;
    F.rw.err_max = 0.2; F.rw.int_max = 0.02; F.rw.dt = 1/asils.util.getf(S.fsw, 'rw_rate_hz', 10);
    F.rate_lpf_s = asils.util.getf(S.fsw, 'rate_lpf_s', 0.3);   % controller rate filter time constant
    F.dump_k = asils.util.getf(S.fsw, 'dump_gain', 2e-3); F.h_bias = asils.util.getf(S.fsw, 'wheel_bias_Nms', 2e-3);
    F.m_res_est = P.sc.m_res;       % ground-calibrated residual dipole the coils cancel (nominal case value)
    F.truth_knowledge = false;                      % debug: FSW gets the true attitude/rate
    F.mekf.meas_scale = 1; F.mekf.proc_scale = 1;
    F.mekf.sig_mag = 0.03; F.mekf.sig_sun = 0.012;  % direction 1-sigma [rad] incl. model error
    F.st_coast_s = 900;                             % gyro-only coasting allowed across a star-tracker outage
    F.igrf_nmax = 10;                               % onboard field model degree (truth: 13)
    P.fsw = F;

    %% overrides (Monte Carlo dispersions and user tweaks): opts.set.<path_with_underscores>
    %  The FSW's calibrated dipole (fsw.m_res_est) is fixed BEFORE overrides, so a
    %  dispersed true dipole (sc.m_res) leaves a realistic calibration error.
    if isfield(opts, 'set')
        P = asils.util.setpaths(P, opts.set);
    end
    P.sc.Iinv = inv(P.sc.I);
end
