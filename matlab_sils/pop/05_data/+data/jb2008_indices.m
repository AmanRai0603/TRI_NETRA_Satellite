function idx = jb2008_indices(startDate, endDate, opts)
%DATA.JB2008_INDICES  JB2008 driver files (SOLFSMY + DTCFILE) via your fetcher.
%   idx = data.jb2008_indices([startDate, endDate, opts])
%   Wraps data_sources/spaceweather/get_jb2008_indices, which downloads the SET
%   files and returns idx.sol (daily F10/S10/M10/Y10 + 81-day means) and idx.dtc
%   (hourly DSTDTC), ready for jb2008_density. Cached in <data.root>/spaceweather.
%   SOURCE  Space Environment Technologies  https://sol.spacenvironment.net/jb2008/indices/
    if nargin<1, startDate=''; end
    if nargin<2, endDate=''; end
    if nargin<3, opts=struct(); end
    cacheDir = fullfile(data.root(),'spaceweather');
    if ~exist(cacheDir,'dir'), mkdir(cacheDir); end
    % ---------------------------------------------------------------------
    % PARAMETER NAMES. This used to pass 'force' and 'startDate'/'endDate'.
    % get_jb2008_indices takes NONE of those -- its inputParser knows only
    % cacheDir, forceDownload, solfsmyFile, dtcFile, maxAgeDays -- and it does not
    % set KeepUnmatched. So every call died with "the name 'force' is not a valid
    % parameter name", even once the fetcher was on the path. The wrapper had
    % never been run against the thing it wraps.
    % ---------------------------------------------------------------------
    args = {'cacheDir', cacheDir, 'forceDownload', logical(getf(opts,'force',false))};

    % ---------------------------------------------------------------------
    % OFFLINE FALLBACK. The toolbox already SHIPS SOLFSMY.TXT and DTCFILE.TXT
    % under 04_atmosphere/density_models/jb2008/ (release 8_1_0, covering to
    % 2026-137). get_jb2008_indices takes 'solfsmyFile'/'dtcFile' to read local
    % copies instead of downloading -- so point it at the bundled files when the
    % caller has not asked to force a refresh and no fresher copy is cached.
    % That makes JB2008 work with no network at all, which matters because this
    % gets called from inside a propagation.
    %
    % The bundled files are STATIC: for any epoch past their coverage, or for
    % 2024-2025 work (SET re-derived S10 in May 2025), pass force=true and get the
    % live files. jb2008_density interpolates with 'extrap' and will NOT tell you.
    % ---------------------------------------------------------------------
    if ~getf(opts,'force',false)
        bundled = fileparts(which('jb2008_density'));
        fSol = fullfile(bundled,'SOLFSMY.TXT'); fDtc = fullfile(bundled,'DTCFILE.TXT');
        cSol = fullfile(cacheDir,'SOLFSMY.TXT');
        if ~exist(cSol,'file') && exist(fSol,'file') && exist(fDtc,'file')
            args = [args, {'solfsmyFile', fSol, 'dtcFile', fDtc}];
        end
    end
    if isfield(opts,'maxAgeDays') && ~isempty(opts.maxAgeDays)
        args = [args, {'maxAgeDays', opts.maxAgeDays}];
    end
    idx = get_jb2008_indices(args{:});

    % get_jb2008_indices has no date-range arguments: it returns the whole SET
    % archive and jb2008_density interpolates into it. Cropping here would only
    % throw away the lag margin the model needs (F10/S10 at t-24h, M10 at t-48h,
    % Y10 at t-120h), so the requested range is used as a COVERAGE CHECK instead.
    if ~isempty(startDate)
        need0 = datenum(startDate) - 6;            % 5-day Y10 lag + 1 day of slack
        if isempty(endDate), need1 = datenum(startDate); else, need1 = datenum(endDate); end
        have0 = datenum(idx.sol.Time(1)); have1 = datenum(idx.sol.Time(end));
        if need0 < have0 || need1 > have1
            warning('data:jb2008_indices:coverage', ...
              ['requested %s..%s (needs data from %s for the Y10 t-120h lag) but SOLFSMY ' ...
               'covers %s..%s. jb2008_density interpolates with ''extrap'', so it will ' ...
               'EXTRAPOLATE silently rather than error. Re-fetch with force=true.'], ...
               datestr(datenum(startDate),'yyyy-mm-dd'), datestr(need1,'yyyy-mm-dd'), ...
               datestr(need0,'yyyy-mm-dd'), datestr(have0,'yyyy-mm-dd'), datestr(have1,'yyyy-mm-dd'));
        end
    end
end
function val=getf(s,f,d), if isfield(s,f)&&~isempty(s.(f)), val=s.(f); else, val=d; end, end
