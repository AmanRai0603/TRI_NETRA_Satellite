function tf = hasinput(f)
%ASILS.UTIL.HASINPUT  Whether an input exists where the twin would read it: in the design database in use when the
%   path is one it answers for (asils.util.inputkey), else on disk (exist(f, 'file') == 2).
%   Owner: Agastya. Copyright (c) 2026 Agastya. All rights reserved.
    D = asils.util.design();
    if ~isempty(D)
        k = asils.util.inputkey(f);
        if ~isempty(k), tf = isKey(D.inputs, k); return, end
    end
    tf = exist(f, 'file') == 2;
end
