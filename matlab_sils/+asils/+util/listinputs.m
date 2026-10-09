function L = listinputs(folder, pattern)
%ASILS.UTIL.LISTINPUTS  The inputs in one folder of the twin's (data/scenarios, data/catalogue), as dir() lists files
%   (.name, .folder), sorted by name: the design database's when one is in use and the folder is one it answers for,
%   else the folder's files. pattern: '*' or '*.json' (a dir() wildcard).
%   L = asils.util.listinputs(fullfile(asils.util.root(), 'data', 'scenarios'), '*.json')
%   Owner: Agastya. Copyright (c) 2026 Agastya. All rights reserved.
    if nargin < 2, pattern = '*'; end
    D = asils.util.design();
    k = '';
    if ~isempty(D), k = asils.util.inputkey(fullfile(folder, 'x')); end
    if isempty(k)
        L = dir(fullfile(folder, pattern));
        L = L(~[L.isdir]);
        [~, o] = sort({L.name}); L = L(o);
        return
    end
    pre = k(1:end-1);                                        % 'data/scenarios/'
    re = ['^' regexptranslate('wildcard', pattern) '$'];
    keys_ = sort(keys(D.inputs));
    names = {};
    for i = 1:numel(keys_)
        q = keys_{i};
        if strncmp(q, pre, numel(pre))
            n = q(numel(pre)+1:end);
            if isempty(strfind(n, '/')) && ~isempty(regexp(n, re, 'once')), names{end+1} = n; end %#ok<AGROW>
        end
    end
    L = struct('name', names, 'folder', folder, 'isdir', false);
    L = L(:);
end
