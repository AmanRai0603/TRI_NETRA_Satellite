function res = run(campaignId, varargin)
%ASILS.CAMPAIGN.RUN  Monte Carlo campaign (data/campaigns/<id>.json).
%   res = asils.campaign.run('mc_nadir_ais')                 % every run, here
%   res = asils.campaign.run('mc_nadir_ais', 'runs', 1:10)   % a subset (a worker)
%   res = asils.campaign.run(..., 'out', dir)                % per-run files
%   Uses parfor when the Parallel Computing Toolbox is present. In Octave,
%   tools/run_campaign.sh starts one process per core over run subsets and
%   asils.campaign.collect gathers them.
    o = struct('runs', [], 'out', '', 'quiet', true);
    for i = 1:2:numel(varargin), o.(varargin{i}) = varargin{i+1}; end
    R = asils.util.root();
    C = asils.util.caseid(asils.util.readjson(fullfile(R, 'data', 'campaigns', [campaignId '.json'])));
    if isempty(o.runs), o.runs = 1:C.runs; end
    if isempty(o.out), o.out = fullfile(R, 'store', 'results', campaignId); end
    if ~exist(o.out, 'dir'), mkdir(o.out); end
    caseFile = fullfile(R, 'cases', [C.case_id '.csv']);
    P0 = asils.config(C.scenario, caseFile);
    runs = o.runs;
    usePar = ~exist('OCTAVE_VERSION', 'builtin') && license('test', 'Distrib_Computing_Toolbox') && numel(runs) > 1;
    if usePar
        parfor i = 1:numel(runs)
            one_(C, P0, runs(i), caseFile, o.out);
        end
    else
        for i = 1:numel(runs)
            one_(C, P0, runs(i), caseFile, o.out);
        end
    end
    res = asils.campaign.collect(campaignId, o.out);
end

function one_(C, P0, k, caseFile, out)
    f = fullfile(out, sprintf('run_%04d.mat', k));
    if exist(f, 'file'), return, end
    [set, d] = asils.campaign.draw(C, P0, k);
    t0 = tic;
    rec = asils.run(C.scenario, caseFile, 'seed', C.seed + 7919*k, 'set', set, 'quiet', true);
    s = asils.campaign.summarise(rec);
    s.k = k; s.draws = d; s.wall_s = toc(t0);
    save_(f, s);
    fprintf('[campaign %s] run %d done (%.0f s)\n', C.id, k, s.wall_s);
end
function save_(f, s) %#ok<INUSD>
    save(f, 's', '-v7');
end
