function h = health(tn)
%TRINETRA.HEALTH  Show what an opened design is and how it stands (trinetra.open shows it as it opens): its version and
%   toolbox, its nodes by behaviour and the built-in count, its groups' releases and signatures, its inputs (each held to
%   its fingerprint), and whether its generated code (trinetra.build) is there and is this design's.
%   trinetra.health(tn);  h = trinetra.health(tn)
%   Owner: Agastya. Copyright (c) 2026 Agastya. All rights reserved.
    H = tn.health;
    fprintf('TRI-NETRA design %s\n', tn.file);
    fprintf('  %s (%s), toolbox %s, needs TRI-NETRA %s or later; library %s\n', s_(H.version), s_(H.kind), s_(H.toolbox), ...
        s_(H.needs_application), s_(H.library));
    b = H.behaviours; f = fieldnames(b); n = cellfun(@(x) b.(x), f); [n, o] = sort(n, 'descend'); f = f(o);
    fprintf('  %d nodes: %s\n', H.nodes, strjoin(cellfun(@(x, y) sprintf('%s %d', x, y), f', num2cell(n'), 'UniformOutput', false), ', '));
    bi = H.built_in;
    if bi.count == 0
        fprintf('  built-in: 0 (no relation of the design is computed in code)\n');
    else
        g = fieldnames(bi.by_group);
        fprintf('  built-in: %d, relations still in code (%s)\n', bi.count, strjoin(g', ', '));
    end
    g = fieldnames(H.groups);
    fprintf('  %d groups at their releases (%s); %d signature(s)%s\n', numel(g), ...
        strjoin(cellfun(@(x) sprintf('%s %s', x, H.groups.(x)), g', 'UniformOutput', false), ', '), H.signatures, ...
        ifelse_(H.signatures == 0, ': the developer''s, not yet signed by a person', ''));
    I = H.inputs; d = fieldnames(I.by_folder);
    bad = I.not_their_fingerprint;
    fprintf('  inputs: %d (%s), %s; %d case(s): %s\n', I.count, ...
        strjoin(cellfun(@(x) sprintf('%d %s', I.by_folder.(x), x), d', 'UniformOutput', false), ', '), ...
        ifelse_(isempty(bad), 'each what its fingerprint says', sprintf('%d NOT what their fingerprints say', numel(bad))), ...
        numel(tn.cases), strjoin(tn.cases, ', '));
    fprintf('  generated code: %s\n', generated_(tn));
    if nargout, h = H; end
end

function t = generated_(tn)
    idx = fullfile(tn.generated, 'index.json');
    if exist(idx, 'file') ~= 2
        t = sprintf('not built (trinetra.build(tn) writes it into %s)', tn.generated);
        return
    end
    I = jsondecode(fileread(idx));
    if ~strcmp(I.fingerprint, tn.fingerprint)
        t = sprintf('%s was built from another design: trinetra.build(tn)', tn.generated);
        return
    end
    t = sprintf('%d files in %s, this design''s (algorithms %s)', numel(I.files), tn.generated, I.alg_id);
end

function s = s_(x)
    if isempty(x), s = '?'; elseif ischar(x), s = x; else, s = num2str(x); end
end

function y = ifelse_(c, a, b)
    if c, y = a; else, y = b; end
end
