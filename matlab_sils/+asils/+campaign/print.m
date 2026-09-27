function print(res)
%ASILS.CAMPAIGN.PRINT  Ensemble table of a campaign.
    fprintf('Campaign %s: %d runs\n', res.id, res.n);
    fprintf('  %-28s %10s %10s %10s %10s %8s %s\n', 'metric', 'mean', 'std', 'pct@lvl', 'required', 'pass%', 'verdict');
    for i = 1:numel(res.stats)
        s = res.stats(i); v = 'no requirement';
        if isfinite(s.pass), if s.pass, v = 'PASS'; else, v = 'FAIL'; end, end
        fprintf('  %-28s %10.4g %10.4g %10.4g %10.4g %7.1f%% %s (%s, level %.2f%%)\n', s.id, s.mean, s.std, s.pct, s.req, 100*s.pass_rate, v, s.unit, s.level);
    end
end
