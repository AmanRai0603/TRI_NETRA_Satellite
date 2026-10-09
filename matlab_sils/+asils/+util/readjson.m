function s = readjson(file)
%ASILS.UTIL.READJSON  Read a JSON input into a struct (jsondecode; MATLAB and Octave 7+): the design database's when one
%   is in use and the path is one of its inputs (asils.util.readtext), else the file's.
    s = jsondecode(asils.util.readtext(file));
end
