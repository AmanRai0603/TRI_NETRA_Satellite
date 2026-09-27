function s = readjson(file)
%ASILS.UTIL.READJSON  Read a JSON file into a struct (jsondecode; MATLAB and Octave 7+).
    fid = fopen(file, 'r');
    assert(fid > 0, 'asils:json:open', 'Cannot open %s', file);
    txt = fread(fid, '*char')'; fclose(fid);
    s = jsondecode(txt);
end
