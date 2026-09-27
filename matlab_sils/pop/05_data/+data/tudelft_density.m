function T = tudelft_density(satellite, startDate, endDate, opts)
%DATA.TUDELFT_DENSITY  Measured thermosphere density (validation truth) via TU Delft.
%   T = data.tudelft_density(satellite, startDate, endDate[, opts])
%   Wraps data_sources/density_reference/fetch_tudelft_density: downloads TU
%   Delft's measured neutral-density files for GOCE / GRACE / GRACE-FO / CHAMP /
%   SWARM, parses them, and returns a UTC timetable of measured density (10 s
%   native). This is the reference to validate the propagator's *modelled* drag
%   against -- compare our along-track density / decay to these measurements.
%   SOURCE  http(s)://thermosphere.tudelft.nl/data/  (open Apache directory)
    if nargin<4, opts=struct(); end
    dl = fullfile(data.root(),'density_reference');
    if ~exist(dl,'dir'), mkdir(dl); end
    if ~isfield(opts,'downloadDir'), opts.downloadDir = dl; end
    if ~isfield(opts,'outDir'),      opts.outDir     = dl; end   % keep .mat in the cache (not pwd)
    if ~isfield(opts,'version'),     opts.version = 2; end
    force = isfield(opts,'force') && opts.force;

    % ---- CACHE-FIRST: reuse a saved TU Delft .mat for this sat+range, so a
    %      re-run needs NO web directory listing and NO network at all. ----
    d0 = datestr(datenum(startDate),'yyyymmdd'); d1 = datestr(datenum(endDate),'yyyymmdd');
    pat = sprintf('%s_*_%s_%s.mat', upper(satellite), d0, d1);
    hits = dir(fullfile(dl, pat));
    if isempty(hits), hits = dir(fullfile(pwd, pat)); end        % also honor legacy root cache
    if ~isempty(hits) && ~force
        fprintf('[TUD] cache OK: %s (no listing/network)\n', hits(1).name);
        S = load(fullfile(hits(1).folder, hits(1).name));
        if isfield(S,'T'), T = S.T; return; end
    end
    T = fetch_tudelft_density(satellite, startDate, endDate, opts);
end
