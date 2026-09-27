function v = subsref_default(s, f, d)
%SUBSREF_DEFAULT  s.(f) if present and non-empty, else the default d.
%   Tiny helper so scripts can read optional struct fields without isfield clutter.
    if isstruct(s) && isfield(s,f) && ~isempty(s.(f)), v = s.(f); else, v = d; end
end
