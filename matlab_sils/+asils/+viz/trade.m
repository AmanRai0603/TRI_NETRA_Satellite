function trade(T, file, visible)
%ASILS.VIZ.TRADE  One trade on one figure: every candidate's objective on every
%   seed (dots), its worst value (bar), feasible bars filled, the winner first.
    if nargin < 3, visible = false; end
    vis = 'off'; if visible, vis = 'on'; end
    n = numel(T.candidates);
    f = figure('visible', vis, 'position', [50 50 900 420]);
    hold on; grid on;
    for i = 1:n
        c = T.candidates(i);
        col = [0.75 0.75 0.75]; if c.feasible, col = [0.25 0.5 0.8]; end
        if i == 1 && ~isempty(T.selected), col = [0.1 0.6 0.35]; end
        if isfinite(c.obj), bar(i, c.obj, 0.6, 'facecolor', col, 'edgecolor', 'none'); end
        v = c.obj_seeds; v = v(isfinite(v));
        if ~isempty(v), plot(i + zeros(size(v)), v, 'k.', 'markersize', 14); end
    end
    set(gca, 'xtick', 1:n, 'xticklabel', {T.candidates.id}, 'ticklabelinterpreter', 'none');
    xlim([0.4 n + 0.6]);
    u = ''; c1 = T.candidates(1); if isfield(c1.metrics, T.objective.metric), u = c1.metrics.(T.objective.metric).unit; end
    ylabel(sprintf('%s [%s]', T.objective.metric, u), 'interpreter', 'none');
    title(sprintf('%s  (green: proposed, blue: feasible, grey: fails a requirement)', T.id), 'interpreter', 'none');
    print(f, file, '-dpng', '-r110'); if ~visible, close(f); end
end
