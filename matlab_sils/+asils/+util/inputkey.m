function k = inputkey(f)
%ASILS.UTIL.INPUTKEY  A path's place in the twin's folder when it is an input the design database answers for
%   ('cases/...' or 'data/...', with / separators), else '' (as the engine's source.rs `key`). A relative path is
%   taken from the current folder, as the engine takes it.
%   Owner: Agastya. Copyright (c) 2026 Agastya. All rights reserved.
    k = '';
    f = strrep(f, '\', '/');
    if ~is_absolute_(f), f = [strrep(pwd, '\', '/') '/' f]; end
    f = regexprep(f, '/(\./)+', '/');
    r = [strrep(asils.util.root(), '\', '/') '/'];
    if numel(f) > numel(r) && strncmp(f, r, numel(r))
        rel = f(numel(r)+1:end);
        if strncmp(rel, 'data/', 5) || strncmp(rel, 'cases/', 6), k = rel; end
    end
end

function a = is_absolute_(f)
    a = ~isempty(f) && (f(1) == '/' || (numel(f) > 2 && f(2) == ':' && f(3) == '/'));
end
