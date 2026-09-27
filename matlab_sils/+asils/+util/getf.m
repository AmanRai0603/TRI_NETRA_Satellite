function v = getf(s, f, d)
%ASILS.UTIL.GETF  s.(f) if present and non-empty, else the default d.
    if isstruct(s) && isfield(s, f) && ~isempty(s.(f)), v = s.(f); else, v = d; end
end
