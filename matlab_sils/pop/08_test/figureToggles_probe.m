function names = figureToggles_probe()
%FIGURETOGGLES_PROBE  The figure names show_OD declares.
%
%   show_OD's figureToggles() is local to that file. Read the ONE list it declares
%   rather than keeping a second copy here: a duplicated list is a second thing to
%   forget to update, which is the single most common bug this audit has found (five
%   copies of the ratio, two definitions of along-track, a catalog that disagreed
%   with the report built from it).
    src = fileread(fullfile('06_validation','realsat','show_OD.m'));
    i0  = strfind(src, 'names = { ');
    if isempty(i0), error('test:figList','cannot find the figure list in show_OD'); end
    tail = src(i0(1):end);
    i1  = strfind(tail, '};');
    blk = tail(1:i1(1));
    t   = regexp(blk, '''(\w+)''', 'tokens');
    names = {};
    for k = 1:numel(t), names{end+1} = t{k}{1}; end %#ok<AGROW>
end
