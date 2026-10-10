function fp = fingerprint(what, arg)
%ASILS.UTIL.FINGERPRINT  What a twin run was flown from, as the engine's `adcs results stale`
%   judges it (engine/crates/adcs-sim/src/store.rs, twin_*): Adler-32 over bytes with carriage
%   returns removed, written 'a32:' and eight hex digits.
%
%   asils.util.fingerprint('source')       the twin's own code: +asils/**/*.m, each file's path
%                                          and Adler-32 in one listing, sorted by path (the
%                                          vendored POP is not its code since S7.19b: no run
%                                          calls it)
%   asils.util.fingerprint('data')         every file under data/ but the generators' test vectors
%   asils.util.fingerprint('file', F)      one file
%
%   The source and data fingerprints are computed once per session. With a design database in use
%   (asils.util.design, trinetra.run), 'data' and 'file' are over the design's inputs (the same bytes as the
%   files tools/from_design.py writes, so the same fingerprints), and 'source' takes the generated packages
%   from its generated folder (trinetra.build) in place of the install's own copies.
%   Owner: Agastya. Copyright (c) 2026 Agastya. All rights reserved.
    persistent cache
    if isempty(cache), cache = containers.Map(); end
    R = asils.util.root();
    D = asils.util.design(); G = ''; tag = '';
    if ~isempty(D)
        G = asils.util.getf(D, 'generated', '');
        tag = ['|' D.file '|' asils.util.getf(D, 'fingerprint', '') '|' G];
    end
    switch what
        case 'file'
            fp = sprintf('a32:%08x', adler(inbytes(D, arg)));
        case {'source', 'data'}
            if isKey(cache, [what tag]), fp = cache([what tag]); return; end
            if strcmp(what, 'source')
                files = listm(R, '+asils');
                roots = repmat({R}, numel(files), 1);
                if ~isempty(G)
                    for p = {'+models', '+relations', '+alg', '+pc'}
                        top = ['+asils/' p{1}];
                        if exist(fullfile(G, '+asils', p{1}), 'dir') ~= 7, continue; end
                        keep = ~strncmp(files, [top '/'], numel(top) + 1);
                        files = files(keep); roots = roots(keep);
                        g = listm(G, top);
                        files = [files; g]; roots = [roots; repmat({G}, numel(g), 1)]; %#ok<AGROW>
                    end
                end
                [files, o] = sort(files); roots = roots(o);
                lines = cellfun(@(f, r) sprintf('%s %08x\n', f, adler(readbytes(fullfile(r, f)))), files, roots, 'UniformOutput', false);
            elseif ~isempty(D)
                ks = keys(D.inputs);
                files = sort(ks(strncmp(ks, 'data/', 5)))';
                files = files(cellfun(@(f) isempty(regexp(f, '_vectors\.json$', 'once')), files));
                lines = cellfun(@(f) sprintf('%s %08x\n', f, adler(textbytes(D.inputs(f)))), files, 'UniformOutput', false);
            else
                files = listall(R, 'data');
                files = files(cellfun(@(f) isempty(regexp(f, '_vectors\.json$', 'once')), files));
                files = sort(files);
                lines = cellfun(@(f) sprintf('%s %08x\n', f, adler(readbytes(fullfile(R, f)))), files, 'UniformOutput', false);
            end
            fp = sprintf('a32:%08x', adler(uint8([lines{:}])));
            cache([what tag]) = fp;
        otherwise
            error('asils:fingerprint', 'fingerprint of %s: source, data or file', what);
    end
end

function b = inbytes(D, f)
    % an input's bytes: the design's when one is in use and the path is one of its inputs
    if ~isempty(D)
        k = asils.util.inputkey(f);
        if ~isempty(k), b = textbytes(asils.util.readtext(f)); return, end
    end
    b = readbytes(f);
end

function b = textbytes(t)
    if all(t < 128), b = uint8(t(:)); else, b = unicode2native(t, 'UTF-8'); b = b(:); end
    b = b(b ~= 13);
end

function b = readbytes(f)
    fid = fopen(f, 'r');
    if fid < 0, error('asils:fingerprint', 'cannot read %s', f); end
    b = fread(fid, Inf, 'uint8=>uint8');
    fclose(fid);
    b = b(b ~= 13);
end

function v = adler(b)
    b = double(b(:));
    n = numel(b);
    A = mod(1 + sum(b), 65521);
    B = mod(n + sum((n:-1:1)' .* b), 65521);
    v = B * 65536 + A;
end

function out = listm(R, top)
    out = listall(R, top);
    out = out(cellfun(@(f) numel(f) > 2 && strcmp(f(end-1:end), '.m'), out));
end

function out = listall(R, top)
    % every file under R/top, as a path relative to R with / separators
    out = {};
    d = dir(fullfile(R, top));
    for i = 1:numel(d)
        n = d(i).name;
        if any(strcmp(n, {'.', '..'})) || n(1) == '.', continue; end
        rel = [top '/' n];
        if d(i).isdir
            out = [out; listall(R, rel)]; %#ok<AGROW>
        else
            out{end+1, 1} = rel; %#ok<AGROW>
        end
    end
end
