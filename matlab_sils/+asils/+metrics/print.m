function print(rec)
%ASILS.METRICS.PRINT  The run's metrics against the case requirements.
    M = rec.metrics;
    fprintf('  %-26s %14s %10s  %s\n', 'metric', 'value', 'required', 'verdict');
    for i = 1:numel(M)
        v = 'no requirement';
        if isfinite(M(i).pass), if M(i).pass, v = 'PASS'; else, v = 'FAIL'; end, end
        fprintf('  %-26s %10.4g %-4s %10.4g  %s\n', M(i).id, M(i).value, M(i).unit, M(i).req, v);
    end
end
