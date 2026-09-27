function localfile = ensure(category, filename, urls, maxAgeDays, opts)
%DATA.ENSURE  Return a local data file, downloading + caching it if needed.
%   localfile = data.ensure(category, filename, urls, maxAgeDays, opts)
%
%   The one caching primitive used by every fetcher:
%     * builds <data.root>/<category>/<filename>;
%     * if that file exists and is younger than maxAgeDays (Inf = never expires),
%       returns it WITHOUT downloading;
%     * otherwise tries each URL in 'urls' (cell array) with websave, writing to
%       a temp file first and moving it into place on success;
%     * if every URL fails but a (stale) local copy exists, returns it with a
%       warning; if there is no local copy at all, errors.
%
%   opts (optional): .force (re-download even if fresh), .verbose, .timeout [s].
%
%   PSEUDOCODE
%     dir  <- root/category ; mkdir if absent
%     path <- dir/filename
%     if exist(path) and age(path) <= maxAgeDays and not force: return path
%     for u in urls:
%         try: websave(tmp,u); move tmp->path; return path
%     if exist(path): warn "using stale cache"; return path
%     error "no data and all downloads failed"
    if nargin<4||isempty(maxAgeDays), maxAgeDays=Inf; end
    if nargin<5, opts=struct(); end
    force   = getf(opts,'force',false);
    verbose = getf(opts,'verbose',true);
    timeout = getf(opts,'timeout',60);
    if ischar(urls), urls={urls}; end

    dir = fullfile(data.root(), category);
    if ~exist(dir,'dir'), mkdir(dir); end
    localfile = fullfile(dir, filename);

    if exist(localfile,'file') && ~force
        ageDays = (now - filedate(localfile));
        if ageDays <= maxAgeDays
            if verbose, fprintf('[data] cache OK  : %s (%.1f d old)\n', filename, ageDays); end
            return
        end
    end

    wo = weboptions('Timeout',timeout);
    for i=1:numel(urls)
        try
            tmp = [localfile '.tmp'];
            websave(tmp, urls{i}, wo);
            if exist(localfile,'file'), delete(localfile); end
            movefile(tmp, localfile);
            if verbose, fprintf('[data] downloaded : %s  <- %s\n', filename, urls{i}); end
            return
        catch e
            if verbose, fprintf('[data] source failed (%s): %s\n', urls{i}, e.message); end
        end
    end

    if exist(localfile,'file')
        warning('data:ensure:stale', ...
            'all downloads failed; using stale cached %s', filename);
        return
    end
    error('data:ensure:unavailable', ...
        ['could not obtain "%s" (category %s). Tried %d URL(s). ' ...
         'Check internet, or place the file manually at:\n  %s'], ...
        filename, category, numel(urls), localfile);
end

function d = filedate(f)
    s = dir(f); d = s.datenum;
end
function val=getf(s,f,d), if isfield(s,f)&&~isempty(s.(f)), val=s.(f); else, val=d; end, end
