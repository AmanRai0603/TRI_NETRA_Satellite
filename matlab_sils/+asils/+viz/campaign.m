function files = campaign(res, outdir, visible)
%ASILS.VIZ.CAMPAIGN  Monte Carlo figures: per metric histogram + empirical CDF
%   with the requirement line and the ensemble percentile at the case level;
%   envelope of the pointing error across runs; metric vs each dispersion.
    if nargin < 2 || isempty(outdir), outdir = fullfile(asils.util.root(), 'store', 'results', res.id); end
    if nargin < 3, visible = false; end
    vis = 'off'; if visible, vis = 'on'; end
    files = {};
    for i = 1:numel(res.stats)
        s = res.stats(i); v = s.values(isfinite(s.values));
        if isempty(v), continue, end
        f = figure('visible', vis, 'position', [50 50 1000 420]);
        subplot(1,2,1); hist(v, max(5, round(sqrt(numel(v))))); hold on; grid on;
        yl = ylim; if isfinite(s.req), plot(s.req*[1 1], yl, 'k--', 'linewidth', 1.5); end
        plot(s.pct*[1 1], yl, 'r-', 'linewidth', 1.5);
        xlabel(sprintf('%s [%s]', strrep(s.id, '_', ' '), s.unit), 'interpreter', 'none'); ylabel('runs');
        title(sprintf('%s: %d runs', res.id, numel(v)), 'interpreter', 'none');
        subplot(1,2,2); vs = sort(v); stairs(vs, (1:numel(vs))/numel(vs)); hold on; grid on;
        if isfinite(s.req), plot(s.req*[1 1], [0 1], 'k--', 'linewidth', 1.5); end
        plot(s.pct*[1 1], [0 1], 'r-'); xlabel(s.unit); ylabel('empirical CDF');
        title(sprintf('p%.2f = %.4g  (req %.4g)', s.level, s.pct, s.req), 'interpreter', 'none');
        files{end+1} = fullfile(outdir, sprintf('mc_%s.png', s.id)); %#ok<AGROW>
        print(f, files{end}, '-dpng', '-r110'); if ~visible, close(f); end
    end
    f = figure('visible', vis, 'position', [50 50 1000 420]);
    for k = 1:res.n
        r = res.runs{k}; semilogy(r.t/60, r.ape_los, 'color', [0.3 0.5 0.8]); hold on;
    end
    grid on; xlabel('time [min]'); ylabel('payload LOS APE [deg]'); title(sprintf('%s: all runs', res.id), 'interpreter', 'none');
    files{end+1} = fullfile(outdir, 'mc_envelope.png'); print(f, files{end}, '-dpng', '-r110'); if ~visible, close(f); end
end
