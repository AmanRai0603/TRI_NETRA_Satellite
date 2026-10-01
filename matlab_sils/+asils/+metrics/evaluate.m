function M = evaluate(rec)
%ASILS.METRICS.EVALUATE  Every metric the scenario declares, judged against the case.
%   Each metric: kind, window, statistic ('max' | 'p99.73' | 'rms' | 'mean'),
%   requirement (a case key, e.g. 'req.ape'). Returns a struct array with
%   value, unit, requirement value and pass (NaN when no requirement).
    S = rec.P.scenario; C = rec.P.case.v;
    ms = S.metrics; M = struct('id', {}, 'kind', {}, 'value', {}, 'unit', {}, 'req', {}, 'req_key', {}, 'pass', {});
    for i = 1:numel(ms)
        m = ms{i};
        win = asils.util.getf(m, 'window', 'all');
        idx = asils.metrics.window(rec, win);
        stat = asils.util.getf(m, 'statistic', 'max');
        unit = 'deg';
        switch m.kind
            case 'time_to_rate'
                thr = m.rate_threshold_deg_s; hold = m.hold_s;
                val = asils.metrics.time_to(rec.t, rec.rate, thr, hold)/60; unit = 'min';
            case {'rpe', 'rpe_los', 'mpe', 'mpe_los', 'pde', 'pde_los', 'rke', 'rke_los', 'mke', 'mke_los', 'kde', 'kde_los'}
                val = stat_(asils.metrics.ecss(m.kind, rec, idx, asils.util.getf(m, 'delta_s', NaN), asils.util.getf(m, 'separation_s', NaN)), stat);
            case 'ape',      val = stat_(rec.ape_3ax(idx), stat);
            case 'ape_los',  val = stat_(rec.ape_los(idx), stat);
            case 'ake',      val = stat_(rec.ake_3ax(idx), stat);
            case 'ake_los',  val = stat_(rec.ake_los(idx), stat);
            case 'rate_stability'
                val = stat_(rec.rks(idx), stat); unit = 'deg/s';
            case 'time_to_threshold'
                t0 = asils.util.getf(m, 'from_s', 0);
                k = find(rec.t >= t0);
                e = rec.(asils.util.getf(m, 'channel', 'ape_los'));
                val = asils.metrics.time_to(rec.t(k) - t0, e(k), m.threshold_deg, asils.util.getf(m, 'hold_s', 10)); unit = 's';
                if asils.util.getf(m, 'unit_min', false), val = val/60; unit = 'min'; end
            case 'wheel_momentum_peak'
                val = max(max(abs(rec.h_w(:, idx)))); unit = 'N m s';
            case 'time_to_mode'        % first entry into a mode [min] (NaN: never)
                k = find(rec.mode == find(strcmp(rec.modes, m.mode)), 1);
                val = NaN; if ~isempty(k), val = rec.t(k)/60; end
                unit = 'min';
            case 'sun_angle'           % -Z_B to Sun, sunlit samples of the window
                sel = idx; if isfield(m, 'mode'), sel = idx(rec.mode(idx) == find(strcmp(rec.modes, m.mode))); end
                val = stat_(rec.sun_angle(sel), stat);
            case 'spin_rate_error'
                val = stat_(abs(abs(rec.spin_z(idx)) - rec.P.fsw.ss.spin_dps), stat); unit = 'deg/s';
            case 'mode_fraction'       % share of the window spent in a mode [%]
                val = 100*mean(rec.mode(idx) == find(strcmp(rec.modes, m.mode))); unit = '%';
            case 'propellant'            % N2O used [g]
                pk = rec.prop_kg(isfinite(rec.prop_kg)); val = 0; if ~isempty(pk), val = 1e3*max(pk); end
                unit = 'g';
            case 'jitter'                % rotor-imbalance pointing jitter, frequency domain [arcsec]
                val = stat_(asils.sizing.jitter(rec, idx), stat); unit = 'arcsec';
            case 'power_margin'          % array power less every load, over the window [W]
                if all(isnan(rec.P_gen)), val = NaN;
                else
                    val = mean(rec.P_gen(idx) - rec.P.case.v.power_load_w - rec.P_mtq(idx) - rec.P_rw(idx) - p_rcs_(rec, idx));
                end
                unit = 'W';
            case 'battery_dod'           % the deepest discharge in the window [%]
                val = 100*(1 - min(rec.soc(idx))); unit = '%';
            case 'soc_min'               % the lowest state of charge in the window [%]
                val = 100*min(rec.soc(idx)); unit = '%';
            case 'power_mean'
                val = mean(rec.P_mtq(idx) + rec.P_rw(idx) + p_rcs_(rec, idx)); unit = 'W';
            case 'power_peak'
                val = max(rec.P_mtq(idx) + rec.P_rw(idx) + p_rcs_(rec, idx)); unit = 'W';
            otherwise
                error('asils:metrics:kind', 'unknown metric kind %s', m.kind);
        end
        rk = asils.util.getf(m, 'requirement', '');
        rv = NaN; pass = NaN;
        if ~isempty(rk)
            rv = C.(strrep(rk, '.', '_'));
        elseif isfield(m, 'limit')             % scenario-local bound (no case key yet)
            rv = m.limit; rk = 'scenario';
        end
        if isfinite(rv)
            if strcmp(asils.util.getf(m, 'sense', 'max'), 'min'), pass = double(val >= rv);
            else, pass = double(val <= rv); end
            if isnan(val), pass = 0; end
        end
        M(end+1) = struct('id', m.id, 'kind', m.kind, 'value', val, 'unit', unit, ...
                          'req', rv, 'req_key', rk, 'pass', pass); %#ok<AGROW>
    end
end
function p = p_rcs_(rec, idx)
    p = 0; if isfield(rec, 'P_rcs'), p = rec.P_rcs(idx); p(~isfinite(p)) = 0; end
end

function v = stat_(x, s)
    x = x(isfinite(x));
    if isempty(x), v = NaN; return, end
    switch s
        case 'max',    v = max(x);
        case 'rms',    v = sqrt(mean(x.^2));
        case 'mean',   v = mean(x);
        case 'p99.73', xs = sort(x); v = xs(max(1, ceil(0.9973*numel(xs))));
        case 'p95',    xs = sort(x); v = xs(max(1, ceil(0.95*numel(xs))));
        otherwise, error('asils:metrics:stat', 'unknown statistic %s', s);
    end
end
