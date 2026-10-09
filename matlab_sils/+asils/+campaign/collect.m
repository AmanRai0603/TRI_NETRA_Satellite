function res = collect(campaignId, out)
%ASILS.CAMPAIGN.COLLECT  Gather per-run files into the campaign result and its statistics.
%   For each metric: every run's value, mean, std, the ensemble percentile at
%   the case's level (99.73 % when stated, else 95 %), and the pass rate
%   (ECSS-E-ST-60-10C two-level statistics: temporal statistic per run inside
%   the scenario metric, ensemble statistic here).
    R = asils.util.root();
    if nargin < 2, out = fullfile(R, 'store', 'results', campaignId); end
    C = asils.util.caseid(asils.util.readjson(fullfile(R, 'data', 'campaigns', [campaignId '.json'])));
    fl = dir(fullfile(out, 'run_*.mat'));
    res.id = campaignId; res.campaign = C; res.runs = {};
    for i = 1:numel(fl)
        L = load(fullfile(out, fl(i).name)); res.runs{end+1} = L.s; %#ok<AGROW>
    end
    n = numel(res.runs);
    res.n = n;
    if n == 0, res.stats = []; return, end
    M0 = res.runs{1}.metrics;
    caseV = asils.case.read(fullfile(R, 'cases', [C.case_id '.csv']));
    for m = 1:numel(M0)
        vals = cellfun(@(r) r.metrics(m).value, res.runs);
        lvl = 95;
        % the case's level for a case requirement; a scenario's own limit (req_key 'scenario') has none
        k = strrep(M0(m).req_key, '.', '_');
        if ~isempty(k) && isfield(caseV, 'level') && isfield(caseV.level, k)
            L = caseV.level.(k);
            if isfinite(L), lvl = L; end
        end
        v = sort(vals(isfinite(vals)));
        st.id = M0(m).id; st.unit = M0(m).unit; st.req = M0(m).req; st.values = vals;
        st.mean = mean(v); st.std = std(v); st.min = min(v); st.max = max(v); st.level = lvl;
        if isempty(v), st.pct = NaN; else, st.pct = v(max(1, ceil(lvl/100*numel(v)))); end
        st.n_valid = numel(v);
        if isfinite(st.req), st.pass_rate = mean(vals <= st.req); st.pass = st.pct <= st.req;
        else, st.pass_rate = NaN; st.pass = NaN; end
        res.stats(m) = st; %#ok<AGROW>
    end
end
