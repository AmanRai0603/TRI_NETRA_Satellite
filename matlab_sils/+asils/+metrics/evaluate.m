function M = evaluate(rec)
%ASILS.METRICS.EVALUATE  Every metric the scenario asks for, measured on the run (the engine's metrics.rs evaluate, in
%   MATLAB): how each kind is measured is kpi's (kpi_metric_evaluate, _statistics, _ecss: asils.models.kpimetrics,
%   kpistats, kpiecss), the jitter pnt's (asils.models.jitter); its value, the unit it is stated in, and its verdict against
%   the case's requirement or the scenario's limit. This file reads the scenario's metric sections (the defaults kpi's where
%   a section states none) and hands the run's channels over. The scenario's words for the kinds, windows, statistics and
%   channels are the schema's lists (adcs-sim schema.rs), in the order of kpi's choices.
%   Owner: Agastya. Copyright (c) 2026 Agastya. All rights reserved.
    KINDS = {'time_to_rate', 'ape', 'ape_los', 'ake', 'ake_los', 'rate_stability', 'time_to_threshold', 'wheel_momentum_peak', ...
             'time_to_mode', 'sun_angle', 'spin_rate_error', 'mode_fraction', 'propellant', 'power_mean', 'power_peak', 'jitter', ...
             'rpe', 'rpe_los', 'mpe', 'mpe_los', 'pde', 'pde_los', 'rke', 'rke_los', 'mke', 'mke_los', 'kde', 'kde_los', ...
             'power_margin', 'battery_dod', 'soc_min'};
    WINDOWS = {'all', 'last_orbit', 'last_half_orbit', 'pointing'};
    STATS = {'max', 'rms', 'mean', 'p95', 'p99.73'};
    CHANNELS = {'ape_3ax', 'ape_los', 'ake_3ax', 'ake_los', 'rate', 'rks', 'sun_angle', 'sun_angle_geo', 'spin_z', 'rate_err'};
    UNITS = {'deg', 'min', 's', 'deg/s', 'N m s', '%', 'g', 'W', 'arcsec'};
    P = rec.P; S = P.scenario; C = P.case.v;
    ms = asils.util.getf(S, 'metrics', {}); if isstruct(ms), ms = num2cell(ms); end
    M = struct('id', {}, 'kind', {}, 'value', {}, 'unit', {}, 'req', {}, 'req_key', {}, 'pass', {});
    [dw, dst, dch, rate_thr, rate_hold, thr, thr_hold, from_s, dsense_min, dunit_min, dend] = asils.models.kpimetrics.kpi_metric_defaults();
    n = numel(rec.t);
    t = rec.t(:); mode = rec.mode(:) - 1;           % the flight software's state numbers
    prop = rec.prop_kg(:); p_mtq = rec.P_mtq(:); p_rw = rec.P_rw(:); p_rcs = rec.P_rcs(:);
    nr = P.dev.mex.rec.n;
    if nr > 0, h = reshape(rec.h_w(1:nr, :), [], 1); else, h = zeros(0, 1); end
    ch = cell(1, numel(CHANNELS));
    for c = 1:numel(CHANNELS), ch{c} = rec.(CHANNELS{c})(:); end
    w = zeros(n, 1); tmp = zeros(n, 1); tt = zeros(n, 1); xx = zeros(n, 1);
    ps = ~isempty(power_load_(C));
    MT = asils.fsw.modes();
    for i = 1:numel(ms)
        m = ms{i};
        spec = asils.util.getf(m, 'window', WINDOWS{dw + 1});
        idx = window_(P, t, mode, spec, WINDOWS);
        ni = numel(idx);
        stw = asils.util.getf(m, 'statistic', '');
        if isempty(stw), st = dst; else, st = opt_(STATS, stw, 5); end
        kind = asils.util.getf(m, 'kind', '');
        k = opt_(KINDS, kind, 31);
        unit_min = flag_(m, 'unit_min', dunit_min);
        mode_named = opt_(MT.state, asils.util.getf(m, 'mode', ''), -1);
        switch k
            case 0
                val = asils.models.kpimetrics.kpi_time_to_rate(t, ch{5}, mode, n, asils.util.getf(m, 'rate_threshold_deg_s', rate_thr), ...
                    asils.util.getf(m, 'hold_s', rate_hold), flag_(m, 'end_at_mode_exit', dend));
            case 6
                name = asils.util.getf(m, 'channel', CHANNELS{dch + 1});
                c = find(strcmp(CHANNELS, name), 1);
                if isempty(c)
                    M(end+1) = struct('id', m.id, 'kind', kind, 'value', NaN, 'unit', '', 'req', NaN, 'req_key', ['unknown channel ' name], 'pass', NaN); %#ok<AGROW>
                    continue
                end
                val = asils.models.kpimetrics.kpi_time_to_threshold(t, ch{c}, n, asils.util.getf(m, 'from_s', from_s), ...
                    asils.util.getf(m, 'threshold_deg', thr), asils.util.getf(m, 'hold_s', thr_hold), unit_min, tt, xx);
            case 7, val = asils.models.kpimetrics.kpi_wheel_peak(h, nr, idx, ni);
            case 8, val = asils.models.kpimetrics.kpi_time_to_mode(t, mode, n, mode_named);
            case 9
                by = isfield(m, 'mode');
                val = asils.models.kpimetrics.kpi_sun_angle_stat(ch{7}, mode, idx, ni, by, mode_named, st, w, tmp);
            case 10, val = asils.models.kpimetrics.kpi_spin_error_stat(ch{9}, idx, ni, P.fsw.ss_spin_dps, st, w, tmp);
            case 11, val = asils.models.kpimetrics.kpi_mode_fraction(mode, idx, ni, mode_named);
            case 12, val = asils.models.kpimetrics.kpi_propellant(prop, n);
            case 15, val = stat_(jitter_(rec, idx + 1), st);
            case 28
                lw = NaN; if ps, lw = C.power_load_w; end
                val = asils.models.kpimetrics.kpi_power_margin(ps, rec.P_gen(:), p_mtq, p_rw, p_rcs, idx, ni, lw);
            case 29, [val, ~] = asils.models.kpimetrics.kpi_battery(rec.soc(:), idx, ni);
            case 30, [~, val] = asils.models.kpimetrics.kpi_battery(rec.soc(:), idx, ni);
            case 13, [val, ~] = asils.models.kpimetrics.kpi_adcs_power(p_mtq, p_rw, p_rcs, idx, ni);
            case 14, [~, val] = asils.models.kpimetrics.kpi_adcs_power(p_mtq, p_rw, p_rcs, idx, ni);
            otherwise
                [has, c] = asils.models.kpimetrics.kpi_kind_channel(k);
                [isk, ek, know, los] = asils.models.kpimetrics.kpi_ecss_kind(k);
                if has
                    val = asils.models.kpistats.kpi_channel_stat(ch{c + 1}, idx, ni, st, w, tmp);
                elseif isk
                    val = stat_(ecss_(rec, ek, know, los, t, idx, asils.util.getf(m, 'delta_s', NaN), asils.util.getf(m, 'separation_s', NaN)), st);
                else
                    val = NaN;
                end
        end
        unit = UNITS{asils.models.kpimetrics.kpi_metric_unit(k, unit_min) + 1};
        % which requirement: the case's (a requirement key) or the scenario's own limit
        rk = asils.util.getf(m, 'requirement', ''); rv = NaN;
        if ~isempty(rk)
            f = strrep(rk, '.', '_'); if isfield(C, f), rv = C.(f); end
        elseif isfield(m, 'limit') && isnumeric(m.limit)
            rv = m.limit; rk = 'scenario';
        end
        sense_min = dsense_min;
        if isfield(m, 'sense') && ischar(m.sense), sense_min = strcmp(m.sense, 'min'); end
        pv = asils.models.kpimetrics.kpi_verdict(val, rv, sense_min);
        if pv == -1, pass = NaN; else, pass = pv; end
        M(end+1) = struct('id', m.id, 'kind', kind, 'value', val, 'unit', unit, 'req', rv, 'req_key', rk, 'pass', pass); %#ok<AGROW>
    end
end

function o = opt_(words, w, other)
    o = find(strcmp(words, w), 1) - 1;
    if isempty(o), o = other; end
end

function b = flag_(m, k, d)
    b = d;
    if isfield(m, k)
        x = m.(k);
        if islogical(x), b = x; elseif isnumeric(x), b = x ~= 0; end
    end
end

function idx = window_(P, t, mode, spec, WINDOWS)
% the samples a metric's window takes (kpistats' kpi_window): 0-based indices
    if strncmp(spec, 'after_s:', 8)
        w = 4; after = str2double(spec(9:end)); if ~isfinite(after), after = 0; end
    else
        w = opt_(WINDOWS, spec, 0); after = 0;
    end
    [ni, ~, ~, idx] = asils.models.kpistats.kpi_window(w, after, t, mode, numel(t), P.orbit.period_s, zeros(numel(t), 1));
    idx = idx(1:max(ni, 0));
end

function v = stat_(x, st)
% the statistic st of the finite values among x (kpistats' kpi_statistic)
    x = x(:); n = numel(x);
    v = asils.models.kpistats.kpi_statistic(x, n, st, zeros(n, 1), zeros(n, 1));
end

function out = ecss_(rec, k, know, los, t, idx, delta, sep)
% the ECSS-E-ST-60-10C indices over the window's samples, in blocks of delta (a drift over sep), in degrees (kpiecss)
    if know, e = rec.e_ake; else, e = rec.e_vec; end
    e = e(:);
    ni = numel(idx);
    [nb, t, idx] = asils.models.kpiecss.kpi_ecss_blocks(t, idx, ni, delta);
    nb = max(nb, 0);
    [nout, ~, ~, ~, ~, ~, out] = asils.models.kpiecss.kpi_ecss(k, los, e, rec.P.dev.boresight, t, idx, ni, delta, sep, ...
                                                              zeros(3*nb, 1), zeros(nb, 1), zeros(max(ni, nb), 1));
    out = out(1:max(nout, 0));
end

function j = jitter_(rec, idx)
% pointing jitter from rotor imbalance, frequency domain [arcsec, each sample of the window] (gp_4's rotor_jitter over the
% recorded momenta, the body's smallest principal moment and lever (jitter_body), the wheel loop's bandwidth the flight
% software flies); zero with no rotor; empty (not computed) when a rotor's part states no imbalance
    P = rec.P; x = P.dev.mex.rec; n = x.n;
    if n == 0, j = zeros(numel(idx), 1); return, end
    if any(P.dev.imbalance(1:n, 3) == 0), j = zeros(0, 1); return, end
    [jmin, d] = asils.models.jitter.jitter_body(P.sc.I, P.sc.box_m);
    jrot = zeros(8, 1); us = jrot; ud = jrot;
    jrot(1:n) = x.jrot(1:n); us(1:n) = P.dev.imbalance(1:n, 1); ud(1:n) = P.dev.imbalance(1:n, 2);
    j = zeros(numel(idx), 1);
    for k = 1:numel(idx)
        h = zeros(8, 1); h(1:n) = rec.h_w(1:n, idx(k));
        j(k) = asils.models.jitter.rotor_jitter(n, h, jrot, us, ud, jmin, d, P.fsw.rw_bandwidth);
    end
end

function L = power_load_(C)
    L = [];
    k = {'power_area_px', 'power_area_mx', 'power_area_py', 'power_area_my', 'power_area_pz', 'power_area_mz', ...
         'power_eff', 'power_batt_wh', 'power_load_w', 'power_soc0'};
    if all(cellfun(@(x) isfield(C, x) && isfinite(C.(x)), k)), L = C.power_load_w; end
end
