function info = build(tn, varargin)
%TRINETRA.BUILD  Generate every MATLAB function the twin flies from an opened design (trinetra.open), into its generated
%   folder beside the user's copy (tn.generated), read-only, and put it on the path ahead of the install's own copies.
%   The library's command does it (tndb build-matlab: the library's translator, byte for byte what tools/engine_build.py
%   and tools/flight_build.py write for the twin), so nothing but the bundled programs is needed:
%     +asils/+models      the design's models: the plant, the environment and the published models, the units and
%                         their chains, the emulators' codecs, the set-up, the flight software's parameters' laws, the
%                         metrics, the sizing (every method block an engine target names, and what it uses)
%     +asils/+relations   the design's relations library and the groups' computing rows, each relation once (needs the
%                         groups' wiring, design/groups: taken from the repository the twin sits in when it is there)
%     +asils/+alg         the flight software's algorithms (the toolbox and fsw/pseudocode 02-09), and their identity
%     +asils/+pc          the language's runtime every package shares
%     index.json          where each file came from: its node, its group's release, the revision (trinetra.which)
%
%   trinetra.build(tn)
%   info = trinetra.build(tn, 'groups', 'path/design/groups', 'quiet', true)
%   The twin's runner (the tick, the bus, the units' states, the recording, the campaigns) is the install's code; it
%   flies what is generated here (trinetra.run, trinetra.campaign, trinetra.use).
%   Owner: Agastya. Copyright (c) 2026 Agastya. All rights reserved.
    o = struct('groups', default_groups_(), 'quiet', false);
    for k = 1:2:numel(varargin), o.(varargin{k}) = varargin{k+1}; end
    args = {'build-matlab', tn.file, tn.generated};
    if ~isempty(o.groups), args = [args, {'--groups', o.groups}]; end
    info = jsondecode(asils.util.tndb(args{:}));
    addpath(tn.generated);                  % first on the path: it is flown in place of the install's copies
    rehash();
    if ~o.quiet
        p = info.packages; f = fieldnames(p);
        fprintf('trinetra.build: %d files into %s (%s), algorithms %s; read-only, on the path\n', info.files, tn.generated, ...
            strjoin(cellfun(@(x) sprintf('%s %d', strrep(x, '_', '.'), p.(x)), f', 'UniformOutput', false), ', '), info.alg_id);
        s = info.skipped;
        if iscell(s), s = [s{:}]; end
        for i = 1:numel(s), fprintf('  not built: %s (%s)\n', s(i).package, s(i).why); end
    end
end

function g = default_groups_()
% the groups' wiring of the repository the twin sits in (design/groups, tools/groupcode.py wire), when it is there
    g = fullfile(fileparts(asils.util.root()), 'design', 'groups');
    if exist(g, 'dir') ~= 7, g = ''; end
end
