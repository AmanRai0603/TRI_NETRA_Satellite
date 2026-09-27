function print(T)
%ASILS.TRADE.PRINT  Console table of a ranked trade.
    fprintf('\nTrade %s — %s\n', T.id, T.label);
    if ~isempty(T.question), fprintf('  %s\n', T.question); end
    fprintf('  objective %s (%s, worst over seeds)', T.objective.metric, asils.util.getf(T.objective, 'sense', 'min'));
    if ~isempty(T.tie_break.metric), fprintf(', tie-break %s', T.tie_break.metric); end
    fprintf('\n  %-4s %-22s %-10s %12s %12s %8s %s\n', 'rank', 'candidate', 'feasible', 'worst', 'mean', 'pass %', 'tie');
    for i = 1:numel(T.candidates)
        c = T.candidates(i);
        fprintf('  %-4d %-22s %-10s %12.4g %12.4g %8.0f %.3g\n', i, c.id, tf_(c.feasible), c.obj, c.obj_mean, c.pass_rate, c.tie);
    end
    fprintf('  -> %s  [%s]\n', T.rationale, T.status);
end
function s = tf_(b)
    if b, s = 'yes'; else, s = 'no'; end
end
