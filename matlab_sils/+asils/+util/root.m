function r = root()
%ASILS.UTIL.ROOT  The matlab_sils folder (parent of +asils).
    r = fileparts(fileparts(fileparts(mfilename('fullpath'))));
end
