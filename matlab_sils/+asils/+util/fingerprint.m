function fp = fingerprint(what, arg)
%ASILS.UTIL.FINGERPRINT  What a twin run was flown from, as the engine's `adcs results stale`
%   judges it (engine/crates/adcs-sim/src/store.rs, twin_*): Adler-32 over bytes with carriage
%   returns removed, written 'a32:' and eight hex digits.
%
%   asils.util.fingerprint('source')       the twin's own code: +asils/**/*.m and POP's code
%                                          (pop/0[1-5]_*/**/*.m), each file's path and Adler-32
%                                          in one listing, sorted by path
%   asils.util.fingerprint('data')         every file under data/ but the generators' test vectors
%   asils.util.fingerprint('file', F)      one file
%
%   The source and data fingerprints are computed once per session.
%   Owner: Agastya. Copyright (c) 2026 Agastya. All rights reserved.
    persistent cache
    if isempty(cache), cache = struct(); end
    R = asils.util.root();
    switch what
        case 'file'
            fp = sprintf('a32:%08x', adler(readbytes(arg)));
        case {'source', 'data'}
            if isfield(cache, what), fp = cache.(what); return; end
            if strcmp(what, 'source')
                files = [listm(R, '+asils'); listpop(R)];
            else
                files = listall(R, 'data');
                files = files(cellfun(@(f) isempty(regexp(f, '_vectors\.json$', 'once')), files));
            end
            files = sort(files);
            lines = cellfun(@(f) sprintf('%s %08x\n', f, adler(readbytes(fullfile(R, f)))), files, 'UniformOutput', false);
            fp = sprintf('a32:%08x', adler(uint8([lines{:}])));
            cache.(what) = fp;
        otherwise
            error('asils:fingerprint', 'fingerprint of %s: source, data or file', what);
    end
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

function out = listpop(R)
    out = {};
    d = dir(fullfile(R, 'pop', '0*'));
    for i = 1:numel(d)
        if d(i).isdir && ~isempty(regexp(d(i).name, '^0[1-5]_', 'once'))
            out = [out; listm(R, ['pop/' d(i).name])]; %#ok<AGROW>
        end
    end
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
