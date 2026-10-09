function m = test_trinetra_open()
%TEST_TRINETRA_OPEN  The twin opens the design database itself (docs/S7_INVENTORY.md S7.18). In a temporary workspace,
%   the regression copy (tests/regression/design.tndb) is opened (trinetra.open: tndb health, tndb read), its code
%   generated (trinetra.build: tndb build-matlab), and:
%   (a) every generated file is the committed one byte for byte (+asils/+models, +relations, +alg, +pc: what
%       tools/engine_build.py and tools/flight_build.py write), the same files, read-only; trinetra.which names a node;
%   (b) every input is the file tools/from_design.py exports for the twin (matlab_sils/data, matlab_sils/cases) byte for
%       byte, and the design holds no other;
%   (c) trinetra.run of short scenarios gives the recording asils.run gives on the committed tree, bit for bit, with the
%       design's code on the path while it flies and the committed tree's after; the run filed names its design.
%   Part of the twin's suite (run_all_tests); tests/test_trinetra_open.py runs it on its own.
%   Owner: Agastya. Copyright (c) 2026 Agastya. All rights reserved.
    R = asils.util.root(); repo = fileparts(R);
    src = fullfile(repo, 'tests', 'regression', 'design.tndb');
    assert(exist(src, 'file') == 2, 'no regression copy at %s', src);
    ws = tempname(); mkdir(ws);
    p0 = path(); d0 = asils.util.design(); w0 = pwd();
    c = onCleanup(@() cleanup_(ws, p0, d0, w0)); %#ok<NASGU>
    copyfile(src, fullfile(ws, 'design.tndb'));
    tn = trinetra.open(fullfile(ws, 'design.tndb'), 'quiet', true);
    assert(tn.health.built_in.count == 0 && isempty(tn.health.inputs.not_their_fingerprint), 'the design''s health');
    info = trinetra.build(tn, 'quiet', true);
    G = tn.generated;
    path(p0);
    u = trinetra.use(tn);
    D = asils.util.design();
    assert(strncmp(which('asils.models.facets.aero_torque'), G, numel(G)) && strcmp(D.file, tn.file), ...
        'trinetra.use flies the design''s code and inputs');
    clear u
    assert(~strncmp(which('asils.models.facets.aero_torque'), G, numel(G)) && isempty(asils.util.design()), ...
        'the committed tree''s code and files once it is put back');

    %% (a) the generated files are the committed ones, byte for byte
    nf = 0;
    for pk = {'+models', '+relations', '+alg', '+pc'}
        g = list_(fullfile(G, '+asils', pk{1}), ''); h = list_(fullfile(R, '+asils', pk{1}), '');
        only = setxor(g, h);
        assert(isempty(only), '+asils/%s: %d file(s) in one and not the other (%s)', pk{1}, numel(only), strjoin(only(1:min(3, end)), ', '));
        for i = 1:numel(g)
            a = bytes_(fullfile(G, '+asils', pk{1}, g{i})); b = bytes_(fullfile(R, '+asils', pk{1}, g{i}));
            assert(isequal(a, b), '+asils/%s/%s differs from the committed file', pk{1}, g{i});
            nf = nf + 1;
        end
    end
    assert(nf == info.files, 'the build made %d files, %d compared', info.files, nf);
    [~, at] = fileattrib(fullfile(G, '+asils', '+pc', 'clamp.m'));
    assert(~at.UserWrite, 'the generated files are read-only');
    w = trinetra.which('asils.models.facets.aero_torque', tn);
    assert(numel(w) == 1 && strcmp(w{1}.node, 'l3_dist_row_02') && ~isempty(w{1}.revision), 'trinetra.which names the node and its revision');

    %% (b) the inputs are the files from_design writes for the twin, byte for byte
    ks = keys(tn.inputs);
    for i = 1:numel(ks)
        f = fullfile(R, ks{i});
        assert(exist(f, 'file') == 2, '%s: in the design, not among the exported files', ks{i});
        assert(isequal(bytes_(f), reshape(unicode2native(tn.inputs(ks{i}), 'UTF-8'), [], 1)), '%s differs from matlab_sils/%s', ks{i}, ks{i});
    end
    on_disk = [strcat('data/', list_(fullfile(R, 'data'), '')), strcat('cases/', list_(fullfile(R, 'cases'), ''))];
    on_disk = on_disk(cellfun(@(f) isempty(regexp(f, '(_vectors\.json|\.gitkeep)$', 'once')), on_disk));
    extra = setdiff(on_disk, ks);
    assert(isempty(extra), '%d exported file(s) the design does not hold (%s)', numel(extra), strjoin(extra(1:min(3, end)), ', '));

    %% (c) trinetra.run = asils.run on the committed tree, bit for bit
    runs = {'nadir_hold_ais', 20; 'fine_hold_img', 10; 'detumble_ais', 20; 'slew_cmg', 10};
    nch = 0;
    for i = 1:size(runs, 1)
        id = runs{i, 1}; set = struct('engine__duration_s', runs{i, 2});
        S = asils.scenario.load(id);
        a = asils.run(id, fullfile(R, 'cases', [S.case_id '.csv']), 'set', set, 'quiet', true);
        b = trinetra.run(tn, id, 'set', set, 'quiet', true, 'save', i == 1, 'figures', false, 'report', false);
        assert(isempty(asils.util.design()) && strcmp(path(), p0), 'trinetra.run puts back the path and the inputs');
        nch = nch + same_(a, b, id);
    end
    man = jsondecode(fileread(fullfile(tn.results, runs{1, 1}, 'manifest.json')));
    assert(strcmp(man.inputs.design.fingerprint, tn.health.inputs_fingerprint) && strcmp(man.inputs.design.generated, G), ...
        'the run filed names the design and the code it flew');
    assert(strcmp(man.inputs.data_fingerprint, asils.util.fingerprint('data')), 'the design''s inputs fingerprint as the files do');

    %% (d) a campaign: trinetra.campaign = asils.campaign.run on the committed tree (two short runs of the Monte Carlo)
    set = struct('engine__duration_s', 10);
    A = asils.campaign.run('mc_nadir_ais', 'runs', 1:2, 'out', fullfile(ws, 'committed_mc'), 'set', set);
    B = trinetra.campaign(tn, 'mc_nadir_ais', 'runs', 1:2, 'set', set, 'quiet', true, 'figures', false);
    assert(A.n == 2 && B.n == 2 && exist(fullfile(tn.results, 'mc_nadir_ais', 'summary.json'), 'file') == 2, 'the campaign is filed');
    for k = 1:2
        assert(bits_(rmfield(A.runs{k}, 'wall_s'), rmfield(B.runs{k}, 'wall_s')), 'mc_nadir_ais run %d differs', k);
    end
    m = sprintf(['%d generated files = committed (+models, +relations, +alg, +pc), read-only; %d inputs = from_design''s; ' ...
                 '%d scenarios x %d channels and 2 campaign runs bit for bit'], nf, numel(ks), size(runs, 1), round(nch/size(runs, 1)));
end

function ok = bits_(x, y)
% the same values, bit for bit, through structs and cells
    ok = isequal(class(x), class(y)) && isequal(size(x), size(y));
    if ~ok, return, end
    if isstruct(x)
        f = fieldnames(x);
        ok = isequal(sort(f), sort(fieldnames(y)));
        for i = 1:numel(x)
            for j = 1:numel(f)
                if ok, ok = bits_(x(i).(f{j}), y(i).(f{j})); end
            end
        end
    elseif iscell(x)
        for i = 1:numel(x), if ok, ok = bits_(x{i}, y{i}); end, end
    elseif isfloat(x)
        ok = isequal(typecast(double(x(:)), 'uint64'), typecast(double(y(:)), 'uint64'));
    else
        ok = isequal(x, y);
    end
end

function n = same_(a, b, id)
% every channel of the two recordings, bit for bit (the wall time and the set-up's file paths aside)
    f = fieldnames(a); n = 0;
    assert(isequal(sort(f), sort(fieldnames(b))), '%s: the recordings hold other channels', id);
    for i = 1:numel(f)
        x = a.(f{i}); y = b.(f{i});
        if any(strcmp(f{i}, {'wall_s', 'P'})), continue, end
        if isnumeric(x) || islogical(x)
            assert(isequal(size(x), size(y)) && isequal(class(x), class(y)), '%s: %s has another shape', id, f{i});
            if isfloat(x)
                assert(isequal(typecast(double(x(:)), 'uint64'), typecast(double(y(:)), 'uint64')), '%s: %s differs', id, f{i});
            else
                assert(isequal(x, y), '%s: %s differs', id, f{i});
            end
            n = n + 1;
        else
            assert(isequaln(x, y), '%s: %s differs', id, f{i});
        end
    end
    assert(isequal(a.P.blob, b.P.blob), '%s: the flight software''s blob differs', id);
end

function b = bytes_(f)
    fid = fopen(f, 'r'); b = fread(fid, Inf, 'uint8=>uint8'); fclose(fid);
end

function out = list_(d, rel)
    out = {};
    L = dir(d);
    for i = 1:numel(L)
        n = L(i).name;
        if any(strcmp(n, {'.', '..'})), continue, end
        if L(i).isdir
            out = [out, list_(fullfile(d, n), [rel n '/'])]; %#ok<AGROW>
        else
            out{end+1} = [rel n]; %#ok<AGROW>
        end
    end
end

function cleanup_(ws, p0, d0, w0)
    cd(w0); path(p0); asils.util.design(d0);
    if ~ispc, system(sprintf('chmod -R u+w %s', asils.util.shellq(ws))); end
    try, confirm_recursive_rmdir(false); catch, end %#ok<CTCH>
    try, rmdir(ws, 's'); catch, end %#ok<CTCH>
end
