function tn = open(file, varargin)
%TRINETRA.OPEN  Open a design database (.tndb) for the MATLAB twin, and show its health (docs/PLAN_2_0.md S7, "the twin
%   opens the database itself"; docs/S7_INVENTORY.md S7.18). The design is read through the library's own command
%   (tndb read, tndb health: JSON over system()), so it works in base MATLAB and GNU Octave >= 8 with no toolbox.
%
%   tn = trinetra.open('path/design.tndb')
%   tn = trinetra.open('path/design.tndb', 'quiet', true)     % no health shown
%
%   tn.file         the design database (absolute path)
%   tn.folder       the folder it is in: the user's copy; trinetra.build writes tn.generated (folder/generated) and
%                   trinetra.run and trinetra.campaign write tn.results (folder/results) there
%   tn.health       what the design is and how it stands (tndb health): its id, version, toolbox, nodes by behaviour,
%                   the built-in count, its groups' releases, its inputs, its signatures; trinetra.health(tn) shows it
%   tn.fingerprint  the design's content hash (its inputs and every node): what trinetra.build's code is held to
%   tn.inputs       the twin's inputs, by their path: 'data/...' and 'cases/<id>.csv' -> text (containers.Map), the
%                   bytes tools/from_design.py exports to matlab_sils/data and matlab_sils/cases
%   tn.scenarios, tn.campaigns, tn.trades, tn.cases   the ids it holds
%   Owner: Agastya. Copyright (c) 2026 Agastya. All rights reserved.
    o = struct('quiet', false);
    for k = 1:2:numel(varargin), o.(varargin{k}) = varargin{k+1}; end
    assert(ischar(file) && ~isempty(file), 'trinetra:open', 'trinetra.open takes the path of a design database (.tndb)');
    f = absolute_(file);
    assert(exist(f, 'file') == 2, 'trinetra:open', 'no design database at %s', f);
    tn = struct();
    tn.file = f;
    tn.folder = fileparts(f);
    tn.generated = fullfile(tn.folder, 'generated');
    tn.results = fullfile(tn.folder, 'results');
    tn.health = jsondecode(asils.util.tndb('health', f));
    tn.fingerprint = tn.health.fingerprint;
    tn.version = tn.health.version;
    J = jsondecode(asils.util.tndb('read', f));
    if isempty(J)
        tn.inputs = containers.Map('KeyType', 'char', 'ValueType', 'any');
    else
        tn.inputs = containers.Map({J.path}, {J.body});
    end
    ks = keys(tn.inputs);
    tn.scenarios = ids_(ks, 'data/scenarios/', '.json');
    tn.campaigns = ids_(ks, 'data/campaigns/', '.json');
    tn.trades = ids_(ks, 'data/trades/', '.json');
    tn.cases = ids_(ks, 'cases/', '.csv');
    if ~o.quiet, trinetra.health(tn); end
end

function ids = ids_(ks, pre, ext)
    ids = {};
    for i = 1:numel(ks)
        k = ks{i};
        if strncmp(k, pre, numel(pre)) && numel(k) > numel(pre) + numel(ext) && strcmp(k(end-numel(ext)+1:end), ext) ...
                && isempty(strfind(k(numel(pre)+1:end), '/'))
            ids{end+1} = k(numel(pre)+1:end-numel(ext)); %#ok<AGROW>
        end
    end
end

function f = absolute_(f)
    g = strrep(f, '\', '/');
    if ~(g(1) == '/' || (numel(g) > 2 && g(2) == ':' && g(3) == '/')), f = fullfile(pwd, f); end
end
