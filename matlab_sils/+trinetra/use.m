function c = use(tn)
%TRINETRA.USE  Fly the twin from an opened design until the returned object is cleared: its generated code (trinetra.build)
%   first on the path, in place of the install's own copies, and its inputs (tn.inputs) in place of the folder's
%   (asils.util.design: the case, scenarios, products, parts, algorithms, catalogue, stated values; the engine builds the
%   flight software's blob from the same design). trinetra.run and trinetra.campaign use it; it lets any of the twin's
%   own functions run on the design too:
%
%   c = trinetra.use(tn);  Z = asils.sizing.size_all('ais_3u');  clear c
%
%   Refused, by name: a design whose code is not built, or was built from another design; an install whose own copies of
%   the generated packages hold a function the design's do not (it would be flown in their place).
%   Owner: Agastya. Copyright (c) 2026 Agastya. All rights reserved.
    G = tn.generated;
    idx = fullfile(G, 'index.json');
    if exist(idx, 'file') ~= 2
        error('trinetra:build', 'the design''s code is not built (%s): trinetra.build(tn)', G);
    end
    I = jsondecode(fileread(idx));
    if ~strcmp(I.fingerprint, tn.fingerprint)
        error('trinetra:build', '%s was built from another design (%s): trinetra.build(tn)', G, I.design);
    end
    prevD = asils.util.design(); prevP = path(); prevW = pwd();
    c = onCleanup(@() restore_(prevD, prevP, prevW));
    % the current folder comes before the path: in the install's own folder its packages would be flown, so the run is
    % made from the user's copy (and the folder put back after)
    if exist(fullfile(prevW, '+asils'), 'dir') == 7
        parts = strsplit(prevP, pathsep);
        rel = cellfun(@(p) ~isempty(p) && ~strcmp(p, '.') && ~any(p(1) == '/\') && ~(numel(p) > 1 && p(2) == ':'), parts);
        if any(rel)                             % a folder named relative to here stays on the path from there
            parts(rel) = cellfun(@(p) fullfile(prevW, p), parts(rel), 'UniformOutput', false);
            path(strjoin(parts, pathsep));
        end
        cd(tn.folder);
    end
    addpath(G);
    rehash();
    shadowed_(G);
    w = which('asils.pc.clamp');
    if ~strncmp(strrep(w, '\', '/'), strrep(G, '\', '/'), numel(G))
        error('trinetra:path', 'the design''s code (%s) is not the one on the path (asils.pc.clamp is %s)', G, w);
    end
    H = tn.health;
    asils.util.design(struct('file', tn.file, 'inputs', tn.inputs, 'fingerprint', tn.fingerprint, 'generated', G, ...
        'version', tn.version, 'design_version', H.design_version, 'inputs_fingerprint', H.inputs_fingerprint, ...
        'toolbox', H.toolbox));
end

function restore_(D, P, W)
    asils.util.design(D);
    if exist(W, 'dir') == 7, cd(W); end        % the folder first: the path may name folders relative to it
    path(P);
    rehash();
end

function shadowed_(G)
% a function the install's own copy of a generated package holds and the design's does not would be flown in its place
    R = asils.util.root();
    for p = {'+models', '+relations', '+alg', '+pc'}
        g = fullfile(G, '+asils', p{1});
        if exist(g, 'dir') ~= 7, continue, end
        mine = list_(fullfile(R, '+asils', p{1}), '');
        theirs = list_(g, '');
        extra = setdiff(mine, theirs);
        if ~isempty(extra)
            error('trinetra:shadow', ['the install''s own +asils/%s holds %d function(s) the design does not give (%s): they would be ' ...
                'flown in its place. Remove them from %s, or fly the install''s own copy (asils.run)'], p{1}, numel(extra), ...
                strjoin(extra(1:min(3, end)), ', '), fullfile(R, '+asils', p{1}));
        end
    end
end

function out = list_(d, rel)
    out = {};
    L = dir(d);
    for i = 1:numel(L)
        n = L(i).name;
        if n(1) == '.', continue, end
        if L(i).isdir
            out = [out, list_(fullfile(d, n), [rel n '/'])]; %#ok<AGROW>
        elseif numel(n) > 2 && strcmp(n(end-1:end), '.m')
            out{end+1} = [rel n]; %#ok<AGROW>
        end
    end
end
