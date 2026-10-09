function res = campaign(tn, id, varargin)
%TRINETRA.CAMPAIGN  Fly a campaign of an opened design (trinetra.open; Monte Carlo or edge cases, data/campaigns in the
%   design) with its generated code (trinetra.build): asils.campaign.run over the design's inputs and code (trinetra.use),
%   its runs and summary in the user's copy, results/<campaign>/ (run_NNNN.mat, summary.json, runs.csv, figures).
%
%   res = trinetra.campaign(tn, 'mc_nadir_ais')
%   res = trinetra.campaign(tn, 'mc_nadir_ais', 'runs', 1:4, 'figures', false, 'quiet', true)
%   res = trinetra.campaign(tn, 'mc_nadir_ais', 'set', struct('engine__duration_s', 600))   % on top of every run's draws
%   A run already filed is not flown again (a cut campaign goes on where it stopped).
%   Owner: Agastya. Copyright (c) 2026 Agastya. All rights reserved.
    o = struct('runs', [], 'quiet', false, 'figures', true, 'out', '', 'set', struct());
    for k = 1:2:numel(varargin)
        assert(isfield(o, varargin{k}), 'trinetra:campaign', 'trinetra.campaign has no option %s', varargin{k});
        o.(varargin{k}) = varargin{k+1};
    end
    assert(any(strcmp(tn.campaigns, id)), 'trinetra:campaign', 'the design %s holds no campaign %s', tn.file, id);
    c = trinetra.use(tn); %#ok<NASGU>
    out = o.out;
    if isempty(out), out = fullfile(tn.results, id); end
    res = asils.campaign.run(id, 'runs', o.runs, 'out', out, 'set', o.set);
    if ~o.quiet, asils.campaign.print(res); end
    asils.campaign.write(res, out);
    if o.figures && res.n > 0, asils.viz.campaign(res, out); end
end
