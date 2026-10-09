function txt = readtext(f)
%ASILS.UTIL.READTEXT  An input's text: from the design database when one is in use and the path is one it answers
%   for (asils.util.design, asils.util.inputkey), refused by name when it holds none; else the file's.
%   Owner: Agastya. Copyright (c) 2026 Agastya. All rights reserved.
    D = asils.util.design();
    if ~isempty(D)
        k = asils.util.inputkey(f);
        if ~isempty(k)
            assert(isKey(D.inputs, k), 'asils:design:missing', ...
                '%s is not in the design database %s (the twin reads its inputs from it alone)', k, D.file);
            txt = D.inputs(k);
            return
        end
    end
    fid = fopen(f, 'r');
    assert(fid > 0, 'asils:json:open', 'Cannot open %s', f);
    txt = fread(fid, '*char')'; fclose(fid);
end
