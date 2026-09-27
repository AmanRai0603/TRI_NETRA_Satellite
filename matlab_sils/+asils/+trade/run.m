function T = run(tradeId, varargin)
%ASILS.TRADE.RUN  Run a trade study (data/trades/<id>.json): every candidate
%   x every seed, then rank (asils.trade.collect).
%
%   T = asils.trade.run('trade_mtq_pointing_ais')             % all jobs, here
%   T = asils.trade.run('trade_mtq_pointing_ais', 'jobs', 3)  % one job (a worker)
%
%   HOW ONE JOB, SEVERAL ALGORITHMS, SEVERAL HARDWARE SETS ARE MANAGED
%   A candidate is either
%     - an ALGORITHM for a slot on fixed hardware:  algorithms = {slot = id}
%       (the registry, catalogue/algorithms, checks it can fly on the product)
%     - a HARDWARE set: another scenario whose product carries other devices
%       (its own selected algorithms)
%     - a TUNING of one algorithm: set = {path = value}
%   and every candidate is flown on the SAME seeds (the same sensor noise
%   draws and initial conditions), so the ranking compares designs, not luck.
%   The winner is PROPOSED for the product's [selected] table; a person
%   confirms it (docs/SELECTION.md).
    o = struct('jobs', [], 'quiet', true);
    for i = 1:2:numel(varargin), o.(varargin{i}) = varargin{i+1}; end
    R = asils.util.root();
    D = asils.trade.spec(tradeId);
    out = fullfile(R, 'store', 'trades', tradeId); if ~exist(out, 'dir'), mkdir(out); end
    J = asils.trade.jobs(D);
    if isempty(o.jobs), o.jobs = 1:numel(J); end
    for k = o.jobs
        j = J(k);
        f = fullfile(out, sprintf('%s_s%d.mat', j.cand, j.seed));
        if exist(f, 'file'), continue, end
        c = D.candidates{j.ci};
        S = asils.scenario.load(j.scenario);
        set = struct();
        if isfield(c, 'set'), set = c.set; end
        if isfield(c, 'algorithms')
            a = fieldnames(c.algorithms);
            for i = 1:numel(a), set.(['fsw__algorithms__' a{i}]) = c.algorithms.(a{i}); end
        end
        if isfield(D, 'duration_s'), set.sim__duration_s = D.duration_s; end
        t0 = tic;
        try
            rec = asils.run(j.scenario, fullfile(R, 'cases', [S.case_id '.csv']), 'seed', j.seed, 'set', set, 'quiet', o.quiet);
            s = asils.campaign.summarise(rec);
            s.alg = rmfield(rec.P.fsw.alg, 'caps'); s.product = rec.P.dev.id;
            s.prop_g = 1e3*max([0, rec.prop_kg(isfinite(rec.prop_kg))]);
            s.error = '';
        catch e
            s = struct('metrics', [], 'error', e.message);
            fprintf(2, '[trade %s] %s seed %d failed: %s\n', tradeId, j.cand, j.seed, e.message);
        end
        s.cand = j.cand; s.seed = j.seed; s.wall_s = toc(t0);
        save(f, 's', '-v7');
        fprintf('[trade %s] %s seed %d done (%.0f s)\n', tradeId, j.cand, j.seed, s.wall_s);
    end
    T = asils.trade.collect(tradeId);
end
