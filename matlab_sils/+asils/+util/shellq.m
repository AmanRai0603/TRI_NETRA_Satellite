function q = shellq(s)
%ASILS.UTIL.SHELLQ  A word quoted for the system() shell: single quotes on POSIX, double quotes on Windows' cmd.
%   Owner: Agastya. Copyright (c) 2026 Agastya. All rights reserved.
    if ispc
        q = ['"' strrep(s, '"', '""') '"'];
    else
        q = ['''' strrep(s, '''', '''\''''') ''''];
    end
end
