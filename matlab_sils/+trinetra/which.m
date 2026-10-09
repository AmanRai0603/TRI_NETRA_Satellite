function e = which(name, tn)
%TRINETRA.WHICH  Name the node a generated function came from: its node, its group's release, the revision its text
%   carries, and the design text it was translated from (the index trinetra.build writes beside the code).
%
%   trinetra.which('aero_torque')                         every generated function of that name
%   trinetra.which('asils.models.facets.aero_torque')     one, by its full name (or 'facets.aero_torque')
%   e = trinetra.which('aero_torque', tn)                 in tn's generated code; else the design in use
%                                                          (trinetra.use), else every generated folder on the path
%   Owner: Agastya. Copyright (c) 2026 Agastya. All rights reserved.
    if nargin >= 2
        folders = {tn.generated};
    else
        D = asils.util.design();
        if ~isempty(D) && ~isempty(asils.util.getf(D, 'generated', ''))
            folders = {D.generated};
        else
            folders = strsplit(path(), pathsep);
        end
    end
    e = [];
    for i = 1:numel(folders)
        f = fullfile(folders{i}, 'index.json');
        if exist(f, 'file') ~= 2, continue, end
        I = jsondecode(fileread(f));
        if ~isfield(I, 'generated_by') || ~strcmp(I.generated_by, 'tndb build-matlab'), continue, end
        F = I.files; if ~iscell(F), F = num2cell(F); end
        for k = 1:numel(F)
            x = F{k};
            full = [x.package '.' x.name];
            if strcmp(x.name, name) || strcmp(full, name) || (numel(full) > numel(name) && strcmp(full(end-numel(name):end), ['.' name]))
                x.folder = folders{i};
                e = [e, {x}]; %#ok<AGROW>
            end
        end
        if ~isempty(e), break, end
    end
    if isempty(e)
        fprintf('%s: no generated function of that name (trinetra.build writes them, with their index)\n', name);
        e = {};
        return
    end
    if nargout == 0
        for k = 1:numel(e)
            x = e{k};
            fprintf('%s.%s  %s\n', x.package, x.name, from_(x));
        end
        clear e
    end
end

function t = from_(x)
    if isfield(x, 'node') && ~isempty(x.node)
        t = sprintf('node %s', x.node);
        if isfield(x, 'release') && ~isempty(x.release), t = sprintf('%s (%s', t, x.release); else, t = [t ' (']; end
        if isfield(x, 'revision') && ~isempty(x.revision), t = sprintf('%s, revision %s)', t, x.revision); else, t = [t ')']; end
        if isfield(x, 'source'), t = sprintf('%s, from %s', t, x.source); end
        if isfield(x, 'from') && ~isempty(x.from), t = sprintf('%s; %s', t, x.from); end
    elseif isfield(x, 'from')
        t = x.from;
        if isfield(x, 'source'), t = sprintf('%s (%s)', t, x.source); end
    else
        t = '?';
    end
end
