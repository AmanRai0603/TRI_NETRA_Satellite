function T = collect(tradeId)
%ASILS.TRADE.COLLECT  Gather a trade's runs, rank the candidates, propose one.
%
%   Per candidate: every metric's mean and worst over the seeds, and the share
%   of (seed, requirement) checks that pass. Ranking:
%     1 FEASIBLE first: every requirement met on every seed
%       (the trade's `gate` metrics, or every metric with a requirement)
%     2 then the OBJECTIVE, its worst value over the seeds (robust choice)
%     3 then the TIE-BREAK (power, propellant, ...) when within `tie_tol`
%   Writes store/trades/<id>/trade.json and trade.png.
    R = asils.util.root();
    D = asils.trade.spec(tradeId);
    dirn = fullfile(R, 'store', 'trades', tradeId);
    J = asils.trade.jobs(D);
    nc = numel(D.candidates);
    ob = D.objective; tb = asils.util.getf(D, 'tie_break', struct('metric', '', 'sense', 'min'));
    tol = asils.util.getf(D, 'tie_tol', 0.05);
    C = struct('id', {}, 'label', {}, 'scenario', {}, 'product', {}, 'algorithms', {}, 'n', {}, 'n_err', {}, ...
               'obj', {}, 'obj_mean', {}, 'obj_seeds', {}, 'tie', {}, 'pass_rate', {}, 'feasible', {}, 'metrics', {}, 'errors', {});
    for i = 1:nc
        c = D.candidates{i};
        js = J([J.ci] == i);
        runs = {};
        errs = {};
        for j = js
            f = fullfile(dirn, sprintf('%s_s%d.mat', j.cand, j.seed));
            if exist(f, 'file') ~= 2, continue, end
            L = load(f); s = L.s;
            if ~isempty(s.error), errs{end+1} = s.error; end %#ok<AGROW>
            runs{end+1} = s; %#ok<AGROW>
        end
        e = struct('id', c.id, 'label', asils.util.getf(c, 'label', c.id), 'scenario', js(1).scenario, ...
                   'product', '', 'algorithms', struct(), 'n', numel(runs), 'n_err', numel(errs), ...
                   'obj', NaN, 'obj_mean', NaN, 'obj_seeds', [], 'tie', NaN, 'pass_rate', NaN, ...
                   'feasible', false, 'metrics', struct(), 'errors', {errs});
        ok = cellfun(@(s) isempty(s.error), runs);
        good = runs(ok);
        if ~isempty(good)
            e.product = good{1}.product; e.algorithms = good{1}.alg;
            ids = {good{1}.metrics.id};
            npass = 0; nreq = 0;
            gate = asils.util.getf(D, 'gate', {});
            if ischar(gate), gate = {gate}; end
            for m = 1:numel(ids)
                v = cellfun(@(s) metric_(s, ids{m}, 'value'), good);
                p = cellfun(@(s) metric_(s, ids{m}, 'pass'), good);
                u = metric_str_(good{1}, ids{m});
                e.metrics.(ids{m}) = struct('mean', mean(v), 'worst', worst_(v, sense_(D, ids{m}, ob, tb)), ...
                    'values', v, 'unit', u, 'pass', p);
                gated = isempty(gate) || any(strcmp(gate, ids{m}));
                if gated && any(isfinite(p))
                    nreq = nreq + numel(p); npass = npass + sum(p == 1);
                end
            end
            if isfield(e.metrics, ob.metric)
                x = e.metrics.(ob.metric);
                e.obj = x.worst; e.obj_mean = x.mean; e.obj_seeds = x.values;
            end
            if ~isempty(tb.metric) && isfield(e.metrics, tb.metric), e.tie = e.metrics.(tb.metric).mean; end
            if nreq > 0, e.pass_rate = 100*npass/nreq; else, e.pass_rate = 100; end
            e.feasible = e.pass_rate == 100 && isfinite(e.obj) && isempty(errs);
        end
        e.n_err = numel(errs);
        C(end+1) = e; %#ok<AGROW>
    end
    % ---- rank
    sgn = 1; if strcmp(asils.util.getf(ob, 'sense', 'min'), 'max'), sgn = -1; end
    key = zeros(nc, 1);
    for i = 1:nc
        o = sgn*C(i).obj; if ~isfinite(o), o = Inf; end
        key(i) = o;
    end
    order = 1:nc;
    feas = [C.feasible];
    order = [sortby_(find(feas), key), sortby_(find(~feas), key)];
    % tie-break among the leaders within tol of the best feasible objective
    if ~isempty(tb.metric) && any(feas)
        b = order(1); lead = order(feas(order) & abs(key(order)' - key(b)) <= tol*max(abs(key(b)), 1e-12));
        if numel(lead) > 1
            ts = arrayfun(@(i) C(i).tie, lead); if strcmp(asils.util.getf(tb, 'sense', 'min'), 'max'), ts = -ts; end
            [~, k] = sort(ts); lead = lead(k);
            order = [lead, setdiff(order, lead, 'stable')];
        end
    end
    T = struct('id', D.id, 'label', D.label, 'question', asils.util.getf(D, 'question', ''), ...
        'kind', asils.util.getf(D, 'kind', 'algorithm'), 'slot', asils.util.getf(D, 'slot', ''), ...
        'promote_to', asils.util.getf(D, 'promote_to', ''), 'objective', ob, 'tie_break', tb, ...
        'seeds', D.seeds, 'ranking', {arrayfun(@(i) C(i).id, order, 'UniformOutput', false)}, ...
        'selected', '', 'rationale', '', 'status', 'PROPOSED · awaiting a person');
    T.candidates = C(order);
    if any(feas)
        w = C(order(1));
        T.selected = w.id;
        T.rationale = sprintf('%s: feasible on all %d seeds, worst %s %.4g %s', w.id, w.n, ob.metric, w.obj, unit_(w, ob.metric));
        if ~isempty(tb.metric) && isfinite(w.tie), T.rationale = [T.rationale sprintf(', %s %.3g', tb.metric, w.tie)]; end
    else
        T.rationale = 'no candidate meets every requirement on every seed';
        k = order(1); if isfinite(C(k).obj), T.selected = ''; T.rationale = [T.rationale sprintf('; best effort %s', C(k).id)]; end
    end
    fid = fopen(fullfile(dirn, 'trade.json'), 'w'); fprintf(fid, '%s', jsonencode(T)); fclose(fid);
    try, asils.viz.trade(T, fullfile(dirn, 'trade.png')); catch err, fprintf(2, 'trade plot: %s\n', err.message); end
    asils.trade.print(T);
end

function v = metric_(s, id, f)
    k = find(strcmp({s.metrics.id}, id), 1);
    if isempty(k), v = NaN; else, v = s.metrics(k).(f); if isempty(v), v = NaN; end, end
end
function u = metric_str_(s, id)
    k = find(strcmp({s.metrics.id}, id), 1); u = s.metrics(k).unit;
end
function u = unit_(c, id)
    u = ''; if isfield(c.metrics, id), u = c.metrics.(id).unit; end
end
function sn = sense_(D, id, ob, tb)
    sn = 'min';
    if strcmp(id, ob.metric), sn = asils.util.getf(ob, 'sense', 'min'); end
    if ~isempty(tb.metric) && strcmp(id, tb.metric), sn = asils.util.getf(tb, 'sense', 'min'); end
end
function w = worst_(v, sn)
    v = v(isfinite(v)); if isempty(v), w = NaN; return, end
    if strcmp(sn, 'max'), w = min(v); else, w = max(v); end
end
function o = sortby_(ix, key)
    [~, k] = sort(key(ix)); o = ix(k); o = o(:)';
end
