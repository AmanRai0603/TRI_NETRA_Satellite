function P = setpaths(P, s)
%ASILS.UTIL.SETPATHS  Apply overrides given as a struct whose field names are
%   dotted paths with '__' for '.', e.g. s.sc__mass_kg = 4.2, s.env__F107 = 160.
%   A path must name a value the configuration has: an override the twin would never read
%   (a misspelling) is refused by name, not quietly added as a new field. An algorithm slot
%   may be chosen (fsw__algorithms__<slot>) even when the scenario did not name one.
    f = fieldnames(s);
    slots = {'detumble', 'attitude', 'pointing', 'mtq_pointing', 'sun_acquisition', 'allocation', 'thrusters', 'sun_spin'};
    for i = 1:numel(f)
        parts = strsplit(f{i}, '__');
        if numel(parts) == 3 && strcmp(parts{1}, 'fsw') && strcmp(parts{2}, 'algorithms')
            if ~any(strcmp(parts{3}, slots))
                error('asils:set:unknown', 'set %s: no algorithm slot %s', strjoin(parts, '.'), parts{3});
            end
        else
            x = P;
            for j = 1:numel(parts)
                if ~isstruct(x) || ~isfield(x, parts{j})
                    error('asils:set:unknown', 'set %s: the configuration has no %s (misspelt?)', strjoin(parts, '.'), strjoin(parts(1:j), '.'));
                end
                x = x.(parts{j});
            end
        end
        P = setfield(P, parts{:}, s.(f{i}));
    end
end
