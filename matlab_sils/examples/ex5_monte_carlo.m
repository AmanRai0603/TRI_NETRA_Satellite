% EX5  Monte Carlo campaign (uses parfor when the Parallel Computing Toolbox is present).
%      In GNU Octave use tools/run_campaign.sh <id> <workers> for parallel workers.
% Copyright (c) 2026 Agastya. All rights reserved.
startup_asils
addpath tools
res = run_campaign('mc_slew_img');      % 40 runs of the target slew (short scenario)
asils.viz.campaign(res, '', true);
