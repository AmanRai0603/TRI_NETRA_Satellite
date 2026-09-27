function sats = tle(norad, startDate, endDate, opts)
%DATA.TLE  Fetch TLEs from OPEN sources (historical-capable), via your chain.
%   sats = data.tle(norad[, startDate, endDate, opts])
%   Wraps data_sources/satellite/fetch_tle_auto, which tries, in order:
%     1) SatChecker (IAU CPS)  -- HISTORICAL elsets by NORAD id + date range (open)
%     2) planet4589.org (J. McDowell) -- historical archive (open)
%     3) CelesTrak            -- CURRENT elements only (open)
%   No login/paywall anywhere. Historical support is what lets you validate
%   DECAYED satellites (GOCE, GRACE, CHAMP, SLATS) that CelesTrak no longer lists
%   -- pass the mission date range and SatChecker returns the elset from then.
%
%   norad : catalog number.  startDate/endDate : 'yyyy-mm-dd' (optional; if given
%           the historical sources are used, else current CelesTrak).
%   Returns a struct array with .name .l1 .l2 (feed validation.parseTLE) and the
%   raw text is cached under <data.root>/tle/<norad>_<range>.tle.
%
%   Runs in real MATLAB (web + datetime).
    if nargin<2, startDate=''; end
    if nargin<3, endDate=''; end
    if nargin<4, opts=struct(); end
    if isfield(opts,'cacheDir')&&~isempty(opts.cacheDir), cacheDir=opts.cacheDir; else, cacheDir=fullfile(data.root(),'tle'); end
    if ~exist(cacheDir,'dir'), mkdir(cacheDir); end
    key = sprintf('%d_%s_%s', norad, san(startDate), san(endDate));
    localfile = fullfile(cacheDir, [key '.tle']);
    if exist(localfile,'file') && ~getf(opts,'force',false)
        sats = validation.read_tle_file(localfile); return
    end
    % fetch_tle_auto DOES NOT EXIST. It never has: 05_data/data_sources/satellite/
    % is not in this tree (only density_reference and spaceweather are), so this
    % line has been an "Undefined function" waiting for the first caller. The real
    % fetcher is validation.fetch_tle -- the same DO_PLOTS class of error, found by
    % check_undefined.py rather than by a user losing a 20 s run to it.
    if exist('fetch_tle_auto','file') == 2
        [txt, srcUsed, urlUsed] = fetch_tle_auto(norad, startDate, endDate);
    else
        [txt, srcUsed, urlUsed] = validation.fetch_tle(norad, startDate, endDate);
    end
    fid=fopen(localfile,'w'); fwrite(fid, txt); fclose(fid);
    if getf(opts,'verbose',true)
        fprintf('[data.tle] NORAD %d via %s\n           %s\n', norad, srcUsed, urlUsed);
    end
    sats = validation.read_tle_file(localfile);
end
function s=san(x), if isempty(x), s='current'; else, s=regexprep(x,'[^0-9]',''); end, end
function val=getf(s,f,d), if isfield(s,f)&&~isempty(s.(f)), val=s.(f); else, val=d; end, end
