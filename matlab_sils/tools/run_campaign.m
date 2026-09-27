function res = run_campaign(id, runs)
%RUN_CAMPAIGN  Run (part of) a Monte Carlo campaign, then collect and plot it.
%   run_campaign('mc_nadir_ais')          % all runs here (parfor in MATLAB when available)
%   run_campaign('mc_nadir_ais', 1:6)     % one worker's share (see run_campaign.sh)
%   Copyright (c) 2026 Agastya. All rights reserved.
    if nargin < 2, runs = []; end
    if exist('OCTAVE_VERSION', 'builtin'), graphics_toolkit('gnuplot'); end
    res = asils.campaign.run(id, 'runs', runs);
    asils.campaign.print(res);
    if isempty(runs) || res.n >= res.campaign.runs
        asils.viz.campaign(res);
        out = fullfile(asils.util.root(), 'store', 'results', id);
        asils.campaign.write(res, out);
    end
end
