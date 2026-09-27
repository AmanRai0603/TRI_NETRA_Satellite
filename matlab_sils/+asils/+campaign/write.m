function write(res, out)
%ASILS.CAMPAIGN.WRITE  Campaign summary: summary.json (stats) and runs.csv
%   (one row per run: every metric and every drawn dispersion).
    if ~exist(out, 'dir'), mkdir(out); end
    S = struct('schema', 'adcs-campaign-result/1', 'owner', 'Agastya', 'id', res.id, ...
        'scenario', res.campaign.scenario, 'case', res.campaign.case_id, 'runs', res.n, 'stats', res.stats);
    fid = fopen(fullfile(out, 'summary.json'), 'w'); fprintf(fid, '%s\n', jsonencode(S)); fclose(fid);
    d0 = res.runs{1}.draws; dn = fieldnames(d0);
    mn = arrayfun(@(m) m.id, res.runs{1}.metrics, 'UniformOutput', false);
    fid = fopen(fullfile(out, 'runs.csv'), 'w');
    fprintf(fid, 'run,%s,%s\n', strjoin(mn, ','), strjoin(dn', ','));
    for k = 1:res.n
        r = res.runs{k};
        fprintf(fid, '%d', r.k);
        for m = 1:numel(r.metrics), fprintf(fid, ',%.9g', r.metrics(m).value); end
        for j = 1:numel(dn)
            v = NaN; if isfield(r.draws, dn{j}), v = r.draws.(dn{j}); end
            fprintf(fid, ',%.9g', v);
        end
        fprintf(fid, '\n');
    end
    fclose(fid);
end
