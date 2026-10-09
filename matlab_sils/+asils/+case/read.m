function c = read(file)
%ASILS.CASE.READ  Read a case CSV (adcs-case/1): the user's only input.
%   c = asils.case.read('cases/ais_3u.csv')
%   c.id, c.title, c.v.<section>_<name> numeric value (NaN when not stated),
%   c.rows (all rows with unit, lo, hi, level, note) for reports.
%   Refuses a file whose meta.schema is not adcs-case/1, a row with fewer than five
%   fields, a key given twice, and a value that is neither blank (not stated) nor a
%   finite number -- the same refusals as the engine's case reader (adcs-sim/src/case.rs).
%   With a design database in use (asils.util.design), a case under cases/ is the design's.
    if ~isempty(asils.util.design()) && ~isempty(asils.util.inputkey(file))
        txt = asils.util.readtext(file);
    else
        fid = fopen(file, 'r');
        assert(fid > 0, 'asils:case:open', 'Cannot open case file %s', file);
        txt = fread(fid, '*char')'; fclose(fid);
    end
    lines = regexp(txt, '\r?\n', 'split');
    hdr = splitcsv_(lines{1});
    need = {'section','key','label','unit','value','lo','hi','level','note'};
    assert(isequal(hdr(1:9), need), 'asils:case:header', ...
        'Case %s: header must be %s', file, strjoin(need, ','));
    c = struct('file', file, 'id', '', 'title', '', 'class', '', 'v', struct(), 'rows', []);
    rows = struct('key', {}, 'label', {}, 'unit', {}, 'value', {}, 'lo', {}, 'hi', {}, 'level', {}, 'note', {});
    seen = {};
    for i = 2:numel(lines)
        if isempty(strtrim(lines{i})), continue, end
        f = splitcsv_(lines{i});
        assert(numel(f) >= 5, 'asils:case:row', ...
            'Case %s line %d: %d field(s); a row has at least section,key,label,unit,value', file, i, numel(f));
        f(end+1:9) = {''};
        key = strtrim(f{2});
        assert(~isempty(key), 'asils:case:key', 'Case %s line %d: the key is empty', file, i);
        assert(~any(strcmp(seen, key)), 'asils:case:twice', 'Case %s line %d: %s is given twice', file, i, key);
        seen{end+1} = key; %#ok<AGROW>
        r = struct('key', f{2}, 'label', f{3}, 'unit', f{4}, 'value', f{5}, ...
            'lo', num_(f{6}), 'hi', num_(f{7}), 'level', num_(f{8}), 'note', f{9});
        rows(end+1) = r; %#ok<AGROW>
        switch r.key
            case 'meta.schema'
                assert(strcmp(r.value, 'adcs-case/1'), 'asils:case:schema', ...
                    'Case %s: meta.schema must be adcs-case/1, got %s', file, r.value);
            case 'meta.case_id', c.id = r.value;
            case 'meta.title',   c.title = r.value;
            case 'meta.class',   c.class = strtrim(r.value);
            otherwise
                if ~strncmp(r.key, 'meta.', 5)
                    c.v.(strrep(r.key, '.', '_')) = value_(r.value, file, i, r.key);
                    c.lo.(strrep(r.key, '.', '_')) = r.lo;
                    c.hi.(strrep(r.key, '.', '_')) = r.hi;
                    c.level.(strrep(r.key, '.', '_')) = r.level;
                end
        end
    end
    assert(~isempty(c.id), 'asils:case:id', 'Case %s has no meta.case_id', file);
    c.rows = rows;
end
function x = value_(s, file, line, key)
%VALUE_  Blank is "not stated" (NaN); anything else must be a finite number.
    s = strtrim(s);
    if isempty(s), x = NaN; return, end
    x = str2double(s);
    assert(isfinite(x), 'asils:case:value', ...
        'Case %s line %d: %s = "%s": not a finite number (leave it blank if it is not stated)', file, line, key, s);
end
function x = num_(s)
    s = strtrim(s);
    if isempty(s), x = NaN; else, x = str2double(s); end
end
function f = splitcsv_(line)
    f = {}; cur = ''; inq = false; i = 1; n = numel(line);
    while i <= n
        ch = line(i);
        if inq
            if ch == '"'
                if i < n && line(i+1) == '"', cur(end+1) = '"'; i = i + 1; %#ok<AGROW>
                else, inq = false; end
            else
                cur(end+1) = ch; %#ok<AGROW>
            end
        else
            if ch == '"', inq = true;
            elseif ch == ',', f{end+1} = cur; cur = ''; %#ok<AGROW>
            else, cur(end+1) = ch; %#ok<AGROW>
            end
        end
        i = i + 1;
    end
    f{end+1} = cur;
end
