function check(S, C)
%ASILS.SCENARIO.CHECK  Refuse a scenario the engine would refuse, by the engine's own rules.
%   Every key must be one the engine reads, every value of its type, every name in its list;
%   a metric's requirement must be a key of the case, and one that judges is not a diagnostic;
%   the schedule fits the flight software.
%   The rules are data/scenario_schema.json, written from the engine's schema
%   (engine/crates/adcs-sim/src/schema.rs), so the twin and the engine refuse the same things.
%   S: a scenario struct (asils.scenario.load or built in memory); C: the case (asils.case.read).
    persistent K
    if isempty(K)
        K = asils.util.readjson(fullfile(asils.util.root(), 'data', 'scenario_schema.json'));
        if isstruct(K.keys), K.keys = num2cell(K.keys); end
        K.paths = cellfun(@(e) e.path, K.keys, 'UniformOutput', false);
    end
    bad = walk(S, '', K, C, {});
    if isfield(S, 'fsw') && isfield(S.fsw, 'schedule') && numel(S.fsw.schedule) > K.max_schedule
        bad{end+1} = sprintf('fsw.schedule: %d entries; the flight software holds at most %d', numel(S.fsw.schedule), K.max_schedule);
    end
    if isfield(S, 'metrics')
        M = S.metrics; if isstruct(M), M = num2cell(M); end
        for i = 1:numel(M)
            m = M{i};
            if isfield(m, 'requirement') && ~isempty(m.requirement)
                k = strrep(m.requirement, '.', '_');
                if ~strncmp(m.requirement, 'req.', 4) || ~isfield(C.v, k)
                    bad{end+1} = sprintf('metrics[%d] (%s).requirement = "%s": the case %s has no such requirement', i - 1, m.id, m.requirement, C.id);
                end
            end
            judges = (isfield(m, 'requirement') && ~isempty(m.requirement)) || isfield(m, 'limit');
            if judges && isfield(m, 'diagnostic')
                bad{end+1} = sprintf('metrics[%d] (%s): judged, so not a diagnostic', i - 1, m.id);
            end
            if isfield(m, 'window') && ~any(strcmp(m.window, K.metric_windows))
                x = regexp(m.window, '^after_s:([-+0-9.eE]+)$', 'tokens', 'once');
                if isempty(x) || ~isfinite(str2double(x{1}))
                    bad{end+1} = sprintf('metrics[%d] (%s).window = "%s": one of %s or after_s:<seconds>', i - 1, m.id, m.window, strjoin(K.metric_windows(:)', ', '));
                end
            end
        end
    end
    if ~isempty(bad)
        error('asils:scenario:refused', 'scenario %s: %s', S.id, strjoin(bad, '; '));
    end
end

function bad = walk(s, path, K, C, bad)
    f = fieldnames(s);
    for i = 1:numel(f)
        k = f{i};
        if any(strcmp(k, {'xCase', 'x_case', 'case_'})), k = 'case'; end
        if strcmp(k, 'case_id'), continue; end          % added by asils.util.caseid, not in the file
        v = s.(f{i});
        if isempty(path), p = k; else, p = [path '.' k]; end
        if strcmp(p, 'fsw.algorithms')
            if ~isstruct(v), bad{end+1} = 'fsw.algorithms: must be a section of slot = algorithm id'; continue; end
            g = fieldnames(v);
            for j = 1:numel(g)
                if ~any(strcmp(g{j}, K.algorithm_slots)), bad{end+1} = sprintf('fsw.algorithms.%s: no such slot', g{j}); end
            end
            continue
        end
        n = find(strcmp(p, K.paths), 1);
        if ~isempty(n)
            e = K.keys{n};
            if strcmp(e.type, 'list')
                if iscell(v), L = v; elseif isstruct(v), L = num2cell(v); else, bad{end+1} = sprintf('%s: must be a list of sections', p); continue; end
                for j = 1:numel(L), bad = walk(L{j}, [p '[]'], K, C, bad); end
            else
                bad = typed(p, e, v, C, bad);
            end
        elseif isstruct(v) && any(strncmp([p '.'], K.paths, numel(p) + 1))
            bad = walk(v, p, K, C, bad);
        else
            bad{end+1} = sprintf('%s: the engine does not read this key (misspelt, or from another tool)', p);
        end
    end
end

function bad = typed(p, e, v, C, bad)
    num = @(x) isnumeric(x) && isscalar(x) && isfinite(x);
    vec = @(x, n) isnumeric(x) && numel(x) == n && all(isfinite(x(:)));
    switch e.type
        case 'num',       ok = num(v);
        case 'numorcase', ok = num(v) || (ischar(v) && strncmp(v, 'case:', 5));
            if ok && ischar(v) && ~(isfield(C.v, strrep(v(6:end), '.', '_')) && isfinite(C.v.(strrep(v(6:end), '.', '_'))))
                bad{end+1} = sprintf('%s = %s: the case does not state %s', p, v, v(6:end));
            end
        case 'str',       ok = ischar(v);
        case 'bool',      ok = islogical(v) && isscalar(v);
        case 'flag',      ok = (islogical(v) && isscalar(v)) || (num(v) && (v == 0 || v == 1));
        case 'vec3',      ok = vec(v, 3);
        case 'numorvec3', ok = num(v) || vec(v, 3);
        case 'quat',      ok = vec(v, 4);
        case 'one_of',    ok = ischar(v) && any(strcmp(v, e.values));
        otherwise,        ok = false;
    end
    if ~ok, bad{end+1} = sprintf('%s: a value of the wrong kind (must be %s)', p, e.type); end
end
