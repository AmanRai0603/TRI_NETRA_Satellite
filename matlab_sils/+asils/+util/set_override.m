function S = set_override(S, k, x)
%ASILS.UTIL.SET_OVERRIDE  Set a dotted path of the scenario (the engine's config.rs set_override): the key is dotted names
%   inside a section the scenario has; the value keeps the type of what it replaces ("case:<key>" may take a number).
%   Owner: Agastya. Copyright (c) 2026 Agastya. All rights reserved.
    sections = {'case', 'fsw', 'id', 'initial', 'label', 'metrics', 'product', 'schema', 'time', 'faults'};
    parts = strsplit(k, '.');
    if ~any(strcmp(parts{1}, sections))
        error('asils:set:unknown', 'set %s: a scenario has no section %s; it has %s; engine settings are engine.<name>', k, parts{1}, strjoin(sections, ', '));
    end
    cur = S; have = true;
    for j = 1:numel(parts)
        if isstruct(cur) && isfield(cur, parts{j}), cur = cur.(parts{j}); else, have = false; break, end
    end
    if have && ~isempty(cur)
        ref = ischar(cur) && strncmp(cur, 'case:', 5) && isnumeric(x);
        kind = @(y) ifelse_(ischar(y), 'text', ifelse_(islogical(y), 'bool', ifelse_(isstruct(y), 'section', ifelse_(isscalar(y), 'number', 'list'))));
        if ~ref && ~strcmp(kind(cur), kind(x)) && ~(strcmp(kind(cur), 'list') && strcmp(kind(x), 'number')) ...
                && ~(strcmp(kind(cur), 'number') && strcmp(kind(x), 'list'))
            error('asils:set:type', 'set %s: the scenario has %s here, and the value is %s', k, kind(cur), kind(x));
        end
    end
    S = setfield(S, parts{:}, x);
end

function y = ifelse_(c, a, b)
    if c, y = a; else, y = b; end
end
