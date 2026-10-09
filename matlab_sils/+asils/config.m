function P = config(scenarioId, caseFile, opts)
%ASILS.CONFIG  Everything a run needs: case + scenario + product -> the plant's truth parameters and the flight
%   software's adcs-fswcfg/1 blob (the engine's adcs-sim config.rs Config::build, in MATLAB).
%   P = asils.config('nadir_hold_ais', 'cases/ais_3u.csv')
%   P = asils.config(..., struct('seed', 7, 'set', struct('engine__f107', 180, 'fsw__rw_bandwidth', 0.5)))
%
%   Reading only. The values come from: the case CSV (mission, orbit, mass, surfaces, magnetic, requirements); the
%   scenario JSON (time, initial state, flight-software choices, metrics, faults); the product and its parts
%   (asils.product.load); the design's stated values where the case and the scenario state none (data/stated.json:
%   dyn's, env's, vv's); the set-up's rules, generated from the design (+asils/+models: caseorbit's mission_epoch,
%   case_mean_motion, dispersed_period and shifted_epoch, truthplant's principal_inertia, residual_dipole_axes,
%   scale_inertia, inertia_products and flexible_mode, cmoffset's cm_offset, facets_box).
%
%   The flight software's parameters are the blob the engine builds for the same scenario, case and overrides
%   (`adcs params`, the engine's Config::build and its checks: a configuration the engine refuses is refused here, with
%   its words), decoded by asils.fsw.params_decode: never recomputed by the twin.
%
%   Overrides (opts.set, a struct; '__' for '.'): the engine's words, `engine__<name>` (engine.f107, engine.kp,
%   engine.mass_kg, engine.inertia_scale, engine.m_res, engine.duration_s, engine.epoch_days, ...: the truth, the flight
%   software keeps its nominal values) and a scenario path (fsw__algorithms__pointing, fsw__rw_bandwidth,
%   initial__rate__magnitude_deg_s, time__dt_s, ...). The twin's older words are read as the engine's (asils.util.overrides).
%   Owner: Agastya. Copyright (c) 2026 Agastya. All rights reserved.
    if nargin < 3, opts = struct(); end
    R = asils.util.root();
    if ~exist(caseFile, 'file'), caseFile = fullfile(R, caseFile); end
    if isstruct(scenarioId), S = scenarioId; sfile = ''; else, S = asils.scenario.load(scenarioId); sfile = scenarioId; end
    [sc_set, eng_set, P.hal] = asils.util.overrides(asils.util.getf(opts, 'set', struct()));
    for i = 1:size(sc_set, 1), S = asils.util.set_override(S, sc_set{i, 1}, sc_set{i, 2}); end
    C = asils.case.read(caseFile);
    asils.scenario.check(S, C);             % the engine's rules: unknown keys, wrong types, missing requirements refused
    v = C.v; st = asils.util.stated();
    P.scenario = S; P.case = C; P.id = S.id;
    P.faults = asils.util.getf(S, 'faults', []);
    P.seed = asils.util.getf(opts, 'seed', 1);
    P.dev = asils.product.load(asils.util.getf(opts, 'product', S.product));
    P.sizing_case = asils.util.getf(opts, 'sizing_case', '');
    P.overrides = [sc_set; eng_set];
    tm = asils.util.getf(S, 'time', struct()); ini = asils.util.getf(S, 'initial', struct());

    %% the set-up: env_case_orbit (the epoch, its seconds rounded; the mean motion and period), dyn_truth_plant, s1_4
    [P.epoch_utc, P.jd0] = asils.models.caseorbit.mission_epoch(v.mission_epoch);
    P.epoch_utc = reshape(P.epoch_utc, 1, 6);
    [n, period_s] = asils.models.caseorbit.case_mean_motion(v.orbit_alt*1e3);
    P.orbit = struct('alt_km', v.orbit_alt, 'inc_deg', v.orbit_inc, 'ecc', v.orbit_ecc, 'ltan_h', v.orbit_ltan, 'argp_deg', 0, ...
        'u0_deg', asils.util.getf(ini, 'arg_lat_deg', st.get('env_start_arg_lat_default')), 'step_s', st.get('env_orbit_step'), ...
        'n', n, 'period_s', period_s, 'atmos', 'dtm2020', ...
        'model', ifelse_(st.flag('env_orbit_precision_default'), 'pop', 'fast'));
    P.env = struct('F107', st.get('env_f107_default'), 'F107a', st.get('env_f107a_default'), 'Kp', st.get('env_kp_default'), ...
        'ap', st.get('env_ap_default'), 'igrf_nmax', st.whole('env_field_degree', 1, 13), 'dt_s', st.get('env_refresh_step'), ...
        'on', [st.flag('env_torque_gravity_gradient'), st.flag('env_torque_aero'), st.flag('env_torque_radiation'), st.flag('env_torque_magnetic')], ...
        'density_scale', st.get('env_density_scale_default'));
    P.sc = struct();
    P.sc.mass_kg = v.mass_m;
    P.sc.I = asils.models.truthplant.principal_inertia(v.mass_imin, v.mass_iint, v.mass_imax);
    P.sc.box_m = class_box_(C);
    P.sc.cm_offset_m = asils.models.cmoffset.cm_offset(v.surface_cpa, st.v3('dyn_cm_direction'));
    P.sc.m_res = asils.models.truthplant.residual_dipole_axes(v.magnetic_dres);
    P.sc.aref_m2 = v.surface_afr; P.sc.cd = v.surface_cd; P.sc.refl = v.surface_refl;
    P.sc.sigma_n = st.get('dyn_surface_accommodation'); P.sc.sigma_t = P.sc.sigma_n;
    P.sc.vb_ratio = st.get('dyn_surface_vb_ratio'); P.sc.spec_frac = st.get('dyn_surface_specular_share');
    P.sim.duration_s = asils.util.getf(tm, 'duration_s', st.get('vv_run_duration_default'));
    P.sim.dt = asils.util.getf(tm, 'dt_s', st.get('fsw_param_dt'));
    P.sim.record_dt = asils.util.getf(tm, 'record_dt_s', st.get('vv_record_step_default'));
    % how a run starts where its scenario's initial section states nothing (dyn's stated values)
    P.start = struct('att_kind', st.whole('dyn_initial_attitude_kind_default', 0, 3), 'axis', st.list('dyn_initial_error_axis', 3), ...
        'angle_deg', st.get('dyn_initial_error_angle'), 'rate_kind', st.whole('dyn_initial_rate_kind_default', 0, 3), ...
        'rate_deg_s', st.list('dyn_initial_rate_value', 3), 'magnitude_deg_s', st.get('dyn_initial_rate_magnitude'), ...
        'extra_deg_s', st.get('dyn_initial_rate_extra'));

    %% the flight software: the algorithm each slot flies, and the blob the engine builds
    P.blob = blob_(R, S, sfile, caseFile, P.overrides);
    P.fsw = asils.fsw.params_decode(P.blob);
    P.mu = P.fsw.mu;                        % the constants' MU_E, as the blob carries it (fswbody's fsw_epoch_earth)
    P.fsw.alg = asils.fsw.select(P.dev, S);
    P.fsw.rw_bandwidth = asils.util.getf(asils.util.getf(S, 'fsw', struct()), 'rw_bandwidth', st.get('fsw_tune_rw_bandwidth'));
    MT = asils.fsw.modes();
    P.gd_kind0 = MT.guid(P.fsw.start_mode + 1);
    P.h_t_rot = h_targets_(P.fsw, P.dev);

    %% the engine's settings (engine.<name>): the truth changes, the flight software keeps what it was loaded with
    P = apply_engine_(P, eng_set);
    % the case's flexible mode on the truth body (dyn_truth_plant): |delta|^2 is mpart of the coupled axis's inertia
    P.sc.flex = asils.models.flexmode.Flex_zero();
    if isfield(v, 'flex_fmode') && isfinite(v.flex_fmode)
        [d, w, z] = asils.models.truthplant.flexible_mode(v.flex_fmode, v.flex_mpart, v.flex_zeta, v.flex_axis, P.sc.I);
        P.sc.flex = struct('on', true, 'delta', d, 'omega', w, 'zeta', z);
    end
    P.sc.Iinv = asils.la.inv(P.sc.I);
end

function P = apply_engine_(P, eng)
% the engine's apply_engine: truth dispersions and model settings
    for i = 1:size(eng, 1)
        k = eng{i, 1}; x = eng{i, 2};
        switch k
            case 'engine.orbit', P.orbit.model = x;
            case 'engine.inertia_scale', P.sc.I = asils.models.truthplant.scale_inertia(P.sc.I, x(:));
            case 'engine.inertia_products'
                [m, pd] = asils.models.truthplant.inertia_products(P.sc.I, x(:));
                assert(pd, 'asils:config:refused', '%s: the inertia is not positive definite', k);
                P.sc.I = m;
            case 'engine.cm_offset_m', P.sc.cm_offset_m = x(:);
            case 'engine.m_res', P.sc.m_res = x(:);
            case 'engine.density_scale', P.env.density_scale = x;
            case 'engine.duration_s', P.sim.duration_s = x;
            case 'engine.orbit_step_s', P.orbit.step_s = x;
            case 'engine.zonal_max', P.orbit.zonal_max = x;
            case 'engine.igrf_nmax', P.env.igrf_nmax = x;
            case 'engine.f107', P.env.F107 = x;
            case 'engine.f107a', P.env.F107a = x;
            case 'engine.kp', P.env.Kp = x;
            case 'engine.ap', P.env.ap = x;
            case 'engine.accommodation', P.sc.sigma_n = x; P.sc.sigma_t = x;
            case 'engine.refl', P.sc.refl = x;
            case 'engine.vb_ratio', P.sc.vb_ratio = x;
            case 'engine.spec_frac', P.sc.spec_frac = x;
            case 'engine.mass_kg', P.sc.mass_kg = x;
            case 'engine.epoch_days'
                % env_case_orbit: the epoch moved, its seconds rounded (the blob's jd0 moved with it: `adcs params`)
                [e, P.jd0] = asils.models.caseorbit.shifted_epoch(P.jd0, x); P.epoch_utc = reshape(e, 1, 6);
            case 'engine.ltan_h', P.orbit.ltan_h = x;
            case 'engine.alt_km'
                P.orbit.alt_km = x; P.orbit.period_s = asils.models.caseorbit.dispersed_period(1e3*x);
            otherwise
                error('asils:config:refused', 'unknown engine override %s', k);
        end
    end
end

function h = h_targets_(p, dev)
% each rotor's momentum at the start when the scenario states none: the flight software's targets (fswrotor's
% fsw_rotor_targets, as the blob's h_bias, the rotors' kinds and capacities give them)
    x = dev.mex.rec;
    h = asils.models.fswrotor.fsw_rotor_targets(x.n, x.kind, x.h_max, x.h0, p.h_bias);
end

function b = blob_(R, S, sfile, caseFile, ov)
% the adcs-fswcfg/1 blob the engine builds (`adcs params`), read from its file
    exe = asils.util.engine();
    tmp = tempname(); out = [tmp '.fswcfg'];
    if isempty(sfile) || ~ischar(sfile)
        sfile = [tmp '.json']; fid = fopen(sfile, 'w'); fprintf(fid, '%s', scenario_json_(S)); fclose(fid);
        ov = ov(strncmp(ov(:, 1), 'engine.', 7), :);       % the scenario's own overrides are in the file
    end
    args = sprintf(' --set %s', strjoin(cellfun(@(k, x) shellq_([k '=' setval_(x)]), ov(:, 1)', ov(:, 2)', 'UniformOutput', false), ' --set '));
    if isempty(ov), args = ''; end
    cmd = sprintf('ADCS_ROOT=%s %s params %s --case %s --out %s%s 2>&1', shellq_(R), shellq_(exe), shellq_(sfile), shellq_(caseFile), shellq_(out), args);
    [rc, txt] = system(cmd);
    if rc ~= 0
        error('asils:config:refused', 'the engine refuses this configuration (adcs params): %s', strtrim(txt));
    end
    fid = fopen(out, 'r'); b = fread(fid, Inf, 'uint8=>uint8'); fclose(fid);
    delete(out); if exist([tmp '.json'], 'file'), delete([tmp '.json']); end
end

function t = scenario_json_(S)
% a scenario held in memory as the JSON the engine reads ('case' is a MATLAB keyword: jsondecode renamed it)
    f = fieldnames(S);
    for k = {'xCase', 'x_case', 'case_'}
        if any(strcmp(f, k{1})), S = rmfield(S, k{1}); end
    end
    if isfield(S, 'case_id'), S.case_zz9 = S.case_id; S = rmfield(S, 'case_id'); end
    t = strrep(jsonencode(S), '"case_zz9"', '"case"');
end

function s = setval_(x)
% an override's value as the engine reads it: JSON (a number, a list, true or false) or text
    if ischar(x), s = x;
    elseif islogical(x) && isscalar(x), s = ifelse_(x, 'true', 'false');
    elseif isscalar(x), s = sprintf('%.17g', x);
    else, s = ['[' strjoin(arrayfun(@(y) sprintf('%.17g', y), x(:)', 'UniformOutput', false), ',') ']'];
    end
end

function q = shellq_(s)
    q = ['''' strrep(s, '''', '''\''''') ''''];
end

function y = ifelse_(c, a, b)
    if c, y = a; else, y = b; end
end

function b = class_box_(C)
%CLASS_BOX_  The body of the case's satellite class (the engine's config::class_box): a blank or unknown meta.class is
%   refused, never assumed.
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
    b = reshape(L{k}.box_m, 3, 1);
end
