function P = setpaths(P, s)
%ASILS.UTIL.SETPATHS  Apply overrides given as a struct whose field names are
%   dotted paths with '__' for '.', e.g. s.sc__mass_kg = 4.2, s.env__F107 = 160.
    f = fieldnames(s);
    for i = 1:numel(f)
        parts = strsplit(f{i}, '__');
        P = setfield(P, parts{:}, s.(f{i}));
    end
end
