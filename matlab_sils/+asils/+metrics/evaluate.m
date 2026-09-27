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
            case 'wheel_momentum_peak'
                val = max(max(abs(rec.h_w(:, idx)))); unit = 'N m s';
            case 'power_mean'
                val = mean(rec.P_mtq(idx) + rec.P_rw(idx)); unit = 'W';
            case 'power_peak'
                val = max(rec.P_mtq(idx) + rec.P_rw(idx)); unit = 'W';
            otherwise
                error('asils:metrics:kind', 'unknown metric kind %s', m.kind);
        end
        rk = asils.util.getf(m, 'requirement', '');
        rv = NaN; pass = NaN;
        if ~isempty(rk)
            rv = C.(strrep(rk, '.', '_'));
            if isfinite(rv), pass = double(val <= rv); end
        end
        M(end+1) = struct('id', m.id, 'kind', m.kind, 'value', val, 'unit', unit, ...
                          'req', rv, 'req_key', rk, 'pass', pass); %#ok<AGROW>
    end
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
