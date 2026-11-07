function run_draws(drawsFile, outDir, runs)
%RUN_DRAWS  Fly the twin on draws it is given (tools/engine_twosample.py writes them): the same dispersions
%   the engine flew, run for run. One file per run, run_NNNN.json: {k, metrics}.
%   run_draws('store/twosample/mc_nadir_ais/draws.json', outDir, 1:4)
%   Copyright (c) 2026 Agastya. All rights reserved.
    R = asils.util.root();
    D = jsondecode(fileread(drawsFile));
    caseFile = fullfile(R, 'cases', [D.case_id '.csv']);
    if ~exist(outDir, 'dir'), mkdir(outDir); end
    draws = D.draws; if ~iscell(draws), draws = num2cell(draws); end
    for i = 1:numel(draws)
        d = draws{i};
        if nargin >= 3 && ~isempty(runs) && ~ismember(d.k, runs), continue, end
        f = fullfile(outDir, sprintf('run_%04d.json', d.k));
        if exist(f, 'file'), continue, end
        set = struct();
        sets = d.set; if ~iscell(sets), sets = num2cell(sets); end
        for j = 1:numel(sets)
            set.(strrep(sets{j}.name, '.', '__')) = sets{j}.value;
        end
        rec = asils.run(D.scenario, caseFile, 'seed', d.seed, 'set', set, 'quiet', true);
        fid = fopen(f, 'w'); fprintf(fid, '%s\n', jsonencode(struct('k', d.k, 'metrics', rec.metrics))); fclose(fid);
        fprintf('[twosample %s] run %d done\n', D.campaign, d.k);
    end
end
