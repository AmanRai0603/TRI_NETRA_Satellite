function Sol = collect(caseId)
%ASILS.SOLUTION.COLLECT  A case's solution: the mode x option matrix, each
%   family's verdict, the recommended solution and its comparison with the
%   benchmarks. Writes store/solutions/<case>/solution.json.
%
%   OPTION (per mode): every metric's mean and worst over the seeds; FEASIBLE
%   when every metric with a requirement (a case key, or the mode's limit)
%   passes on every seed.
%   FAMILY (catalogue/families.toml): for each mode, the options its actuators
%   allow (an option needs 'mtq' + its actuator + its dump); the best of them
%   -- feasible first, then the mode's objective (worst over seeds), then the
%   mean power -- is the family's method for that mode. A family PASSES when
%   every mode has a feasible method.
%   RECOMMENDATION: among role = "solution" families, the simplest that
%   passes (mtq -> mtq_fmr -> mtq_fmr_rcs); ties by mass, then power. If none
%   passes, the one passing most modes, with the gaps named. Benchmarks are
%   scored the same way and only compared.
    R = asils.util.root();
    Z = asils.util.readjson(fullfile(R, 'store', 'sized', caseId, 'sizing.json'));
    Fam = asils.util.readjson(fullfile(R, 'data', 'families.json'));
    fams = Fam.family; if ~iscell(fams), fams = num2cell(fams); end
    ids = asils.solution.mode();
    Sol = struct('case', caseId, 'class', Z.class, 'modes', struct(), 'families', struct());
    for m = 1:numel(ids)
        M = asils.solution.mode(ids{m});
        ob = M.objective; sense = 'min';
        opts = struct('id', {}, 'uses', {}, 'family', {}, 'n', {}, 'feasible', {}, 'obj', {}, 'obj_mean', {}, ...
                      'power', {}, 'metrics', {}, 'failed', {}, 'errors', {});
        for i = 1:numel(M.options)
            o = M.options{i};
            f = dir(fullfile(R, 'store', 'solutions', caseId, ids{m}, sprintf('%s_s*.mat', strrep(o.id, '+', '_'))));
            runs = {}; errs = {};
            for k = 1:numel(f)
                L = load(fullfile(f(k).folder, f(k).name)); s = L.s;
                if isempty(s.error), runs{end+1} = s; else, errs{end+1} = s.error; end %#ok<AGROW>
            end
            e = struct('id', o.id, 'uses', {o.uses}, 'family', o.family, 'n', numel(runs), 'feasible', false, ...
                       'obj', NaN, 'obj_mean', NaN, 'power', NaN, 'metrics', struct(), 'failed', {{}}, 'errors', {errs});
            if ~isempty(runs)
                mids = {runs{1}.metrics.id}; ok = true;
                for q = 1:numel(mids)
                    v = cellfun(@(s) val_(s, mids{q}, 'value'), runs);
                    p = cellfun(@(s) val_(s, mids{q}, 'pass'), runs);
                    rq = val_(runs{1}, mids{q}, 'req');
                    un = runs{1}.metrics(q).unit;
                    e.metrics.(mids{q}) = struct('mean', mean(v), 'worst', worst_(v, mids{q}), 'unit', un, 'req', rq, ...
                                                 'pass', all(p == 1 | isnan(p)), 'bound', any(isfinite(p)));
                    if any(p == 0), ok = false; e.failed{end+1} = mids{q}; end
                end
                e.feasible = ok && isempty(errs);
                if isfield(e.metrics, ob), e.obj = e.metrics.(ob).worst; e.obj_mean = e.metrics.(ob).mean; end
                if isfield(e.metrics, 'power_mean'), e.power = e.metrics.power_mean.mean; end
            end
            opts(end+1) = e; %#ok<AGROW>
        end
        Sol.modes.(ids{m}) = struct('label', M.label, 'objective', ob, 'options', opts);
    end
    % ---- families
    rank_all = {};
    for i = 1:numel(fams)
        fa = fams{i}; acts = fa.actuators; if ~iscell(acts), acts = cellstr(acts); end
        E = struct('id', fa.id, 'role', fa.role, 'label', fa.label, 'simplicity', fa.simplicity, ...
                   'mass_kg', Z.families.(fa.id).mass_kg, 'power_nominal_W', Z.families.(fa.id).power_W, ...
                   'volume_L', Z.families.(fa.id).volume_L, 'methods', struct(), 'passes', 0, 'gaps', {{}});
        for m = 1:numel(ids)
            O = Sol.modes.(ids{m}).options;
            allowed = arrayfun(@(o) all(ismember(o.uses, acts)), O);
            best = pick_(O(allowed));
            if isempty(best)
                E.methods.(ids{m}) = struct('option', '', 'feasible', false, 'obj', NaN, 'power', NaN);
                E.gaps{end+1} = sprintf('%s: no option for this hardware', ids{m});
            else
                E.methods.(ids{m}) = struct('option', best.id, 'feasible', best.feasible, 'obj', best.obj, 'power', best.power);
                if best.feasible, E.passes = E.passes + 1;
                else, E.gaps{end+1} = sprintf('%s: %s fails %s', ids{m}, best.id, strjoin(best.failed, ', ')); end
            end
        end
        E.pass = E.passes == numel(ids);
        Sol.families.(fa.id) = E;
        rank_all{end+1} = E; %#ok<AGROW>
    end
    % ---- recommendation among our solutions
    sol = rank_all(cellfun(@(e) strcmp(e.role, 'solution'), rank_all));
    key = cellfun(@(e) [~e.pass, (numel(ids) - e.passes), e.simplicity, e.mass_kg, e.power_nominal_W], sol, 'UniformOutput', false);
    K = cell2mat(key(:)); [~, o] = sortrows(K);
    rec = sol{o(1)};
    Sol.recommended = rec.id;
    if rec.pass
        Sol.verdict = sprintf('%s passes every mode', rec.id);
    else
        Sol.verdict = sprintf('no solution family passes every mode; best %s (%d/%d modes): %s', ...
            rec.id, rec.passes, numel(ids), strjoin(rec.gaps, '; '));
    end
    Sol.ranking = cellfun(@(e) e.id, sol(o), 'UniformOutput', false);
    Sol.mode_ids = ids;
    fid = fopen(fullfile(R, 'store', 'solutions', caseId, 'solution.json'), 'w'); fprintf(fid, '%s', jsonencode(Sol)); fclose(fid);
    asils.solution.print(Sol);
end

function b = pick_(O)
%PICK_  Best option: feasible first, then the objective (worst over seeds), then power.
    b = [];
    if isempty(O), return, end
    k = zeros(numel(O), 3);
    for i = 1:numel(O)
        ob = O(i).obj; if ~isfinite(ob), ob = Inf; end
        pw = O(i).power; if ~isfinite(pw), pw = Inf; end
        k(i, :) = [~O(i).feasible, ob, pw];
    end
    [~, j] = sortrows(k); b = O(j(1));
end

function v = val_(s, id, f)
    v = NaN;
    k = find(strcmp({s.metrics.id}, id), 1);
    if ~isempty(k) && ~isempty(s.metrics(k).(f)), v = double(s.metrics(k).(f)); end
end

function w = worst_(v, id)
    v = v(isfinite(v)); if isempty(v), w = NaN; return, end
    if any(strcmp(id, {'sun_spin_share_last_orbit'})), w = min(v); else, w = max(v); end
end
