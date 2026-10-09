function out = tndb(varargin)
%ASILS.UTIL.TNDB  Run the design library's command (tndb: asils.util.program) over system() and give what it printed:
%   the twin's way into a design database, with no toolbox (docs/S7_INVENTORY.md S7.18). Its answer is read from a file,
%   not from system()'s capture, so it is the same bytes in MATLAB and Octave on every platform; what it says on error
%   is the error's text.
%   out = asils.util.tndb('read', 'design.tndb')
%   Owner: Agastya. Copyright (c) 2026 Agastya. All rights reserved.
    exe = asils.util.program('tndb');
    if isempty(exe) || exist(exe, 'file') ~= 2
        error('asils:tndb:missing', ['the design library''s command (tndb) is not here: set TNDB_BIN, or build it ' ...
            '(cargo build --release -p trinetra-design in engine/)']);
    end
    tmp = tempname(); fo = [tmp '.out']; fe = [tmp '.err'];
    c = onCleanup(@() cleanup_(fo, fe)); %#ok<NASGU>
    args = cellfun(@asils.util.shellq, [{exe}, varargin], 'UniformOutput', false);
    [rc, ~] = system(sprintf('%s > %s 2> %s', strjoin(args, ' '), asils.util.shellq(fo), asils.util.shellq(fe)));
    out = read_(fo);
    if rc ~= 0
        error('asils:tndb:refused', 'tndb %s: %s', varargin{1}, strtrim([read_(fe) ' ' out]));
    end
end

function t = read_(f)
    t = '';
    fid = fopen(f, 'r', 'n', 'UTF-8');
    if fid < 0, return, end
    t = fread(fid, Inf, '*char')'; fclose(fid);
end

function cleanup_(varargin)
    for i = 1:numel(varargin)
        if exist(varargin{i}, 'file') == 2, delete(varargin{i}); end
    end
end
