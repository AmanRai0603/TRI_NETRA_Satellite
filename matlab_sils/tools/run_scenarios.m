function run_scenarios(ids)
%RUN_SCENARIOS  Run scenarios and file each result (channels, figures, rec.mat, HTML).
%   run_scenarios                         % every scenario in data/scenarios
%   run_scenarios({'slew_img'})           % a subset
%   Results: store/results/<scenario>/
%   Copyright (c) 2026 Agastya. All rights reserved.
    R = asils.util.root();
    if nargin < 1 || isempty(ids)
        d = asils.util.listinputs(fullfile(R, 'data', 'scenarios'), '*.json');
        ids = cellfun(@(n) n(1:end-5), {d.name}, 'UniformOutput', false);
    end
    if ischar(ids), ids = {ids}; end
    if exist('OCTAVE_VERSION', 'builtin'), graphics_toolkit('gnuplot'); end
    for i = 1:numel(ids)
        S = asils.scenario.load(ids{i});
        rec = asils.run(ids{i}, fullfile('cases', [S.case_id '.csv']));
        asils.result.save(rec);
    end
end
