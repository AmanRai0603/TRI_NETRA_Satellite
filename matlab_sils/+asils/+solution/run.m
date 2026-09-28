function run(caseId, varargin)
%ASILS.SOLUTION.RUN  Fly jobs of a case's solution matrix (asils.solution.jobs).
%   asils.solution.run('ais_3u')                 % every job, here
%   asils.solution.run('ais_3u', 'jobs', 5)      % one job (a worker)
%   Each run is filed as store/solutions/<case>/<mode>/<option>_s<seed>.mat
%   (metrics + decimated channels); an existing file is not re-run.
%   The case must be sized first (asils.sizing.size_all).
    o = struct('jobs', [], 'seeds', [1 2], 'quiet', true);
    for i = 1:2:numel(varargin), o.(varargin{i}) = varargin{i+1}; end
    R = asils.util.root();
    J = asils.solution.jobs(caseId, o.seeds);
    if isempty(o.jobs), o.jobs = 1:numel(J); end
    for k = o.jobs
        j = J(k);
        d = fullfile(R, 'store', 'solutions', caseId, j.mode); if ~exist(d, 'dir'), mkdir(d); end
        f = fullfile(d, sprintf('%s_s%d.mat', strrep(j.option, '+', '_'), j.seed));
        if exist(f, 'file'), continue, end
        S = asils.solution.scenario(caseId, j.mode, j.option);
        t0 = tic;
        try
            rec = asils.run(S, fullfile(R, 'cases', [caseId '.csv']), 'seed', j.seed, 'quiet', o.quiet);
            s = asils.campaign.summarise(rec);
            s.sun_angle = rec.sun_angle_geo(1:max(1, round(10/rec.P.sim.record_dt)):end);
            s.mode_log = rec.mode_log; s.product = rec.P.dev.id; s.alg = rmfield(rec.P.fsw.alg, 'caps');
            s.budget = rec.P.dev.budget; s.error = '';
        catch e
            s = struct('metrics', [], 'error', e.message);
            fprintf(2, '[solution %s] %s/%s seed %d failed: %s\n', caseId, j.mode, j.option, j.seed, e.message);
        end
        s.mode = j.mode; s.option = j.option; s.seed = j.seed; s.wall_s = toc(t0);
        save(f, 's', '-v7');
        fprintf('[solution %s] %s / %s seed %d done (%.0f s)\n', caseId, j.mode, j.option, j.seed, s.wall_s);
    end
end
