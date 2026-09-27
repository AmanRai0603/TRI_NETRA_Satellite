function TT = get_f107(startDate, endDate, opts)
%GET_F107  F10.7 (+81-day centered mean) from NASA OMNI2 as a UTC timetable.
%
% Input  : start/end date, opts (cache dir)
% Process: downloads the covering OMNI2 yearly files via get_omni2 -> extracts daily F10.7 ->
%          computes the 81-day centered mean
% Output : UTC timetable TT with F107 and F107_bar
%
%
% SOURCE: NASA/SPDF OMNI2 hourly files (omni2_YYYY.dat). The F10.7 in OMNI2 is
% the daily Penticton/DRAO 10.7 cm flux -- the same ultimate measurement other
% archives redistribute, but NASA-curated. (For F10.7 there is no meaningful
% accuracy difference between OMNI and CelesTrak; both come from DRAO.)
%
% Because the CENTERED 81-day mean needs ~40 days on each side, this fetches the
% OMNI2 year file(s) spanning [start-41d, end+41d] (so it may pull 2 files near a
% year boundary), collapses the hourly F10.7 to daily, computes the mean, and
% subsets to the requested range.
%
% Used as the LAST-RESORT solar driver for the DTM2020 research model (via
% f30_from_f107). Prefer real F30 (get_f30 / get_f30_cls) when available.
%
% USAGE
%   f107TT  = get_f107('2009-11-01','2009-11-30');
%   F30     = f30_from_f107(f107TT.F107);
%   F30_bar = f30_from_f107(f107TT.F107_bar);
%
% INPUTS
%   startDate,endDate - 'YYYY-MM-DD' or datetime (UTC)
%   opts (optional):
%     .meanWindowDays (default 81)
%     .localFiles (default {}) - cell of local omni2_YYYY.dat paths, in the same
%                                order as the years needed (offline use)
%
% OUTPUT  TT - timetable (UTC daily): Time, F107 [sfu], F107_bar [sfu]
%
% REQUIRES get_omni2.m on the path.

if nargin < 3, opts = struct(); end
if ~isfield(opts,'meanWindowDays'), opts.meanWindowDays = 81; end
if ~isfield(opts,'localFiles'),     opts.localFiles     = {}; end
% cacheDir/downloadDir must reach get_omni2 or nothing is cached at all. This was
% never forwarded: get_omni2 was called bare, so it downloaded to a temp file every
% time. data.spaceweather passes .cacheDir; accept .downloadDir as an alias.
if ~isfield(opts,'cacheDir'),       opts.cacheDir = '';      end
if isempty(opts.cacheDir) && isfield(opts,'downloadDir'),  opts.cacheDir = opts.downloadDir; end

s = datetime(startDate,'TimeZone','UTC');
e = datetime(endDate,  'TimeZone','UTC');
half  = ceil(opts.meanWindowDays/2);
years = year(s - days(half+1)) : year(e + days(half+1));
fprintf('[F107/OMNI2] years needed: %s\n', mat2str(years));

allT = datetime.empty(0,1);  allT.TimeZone = 'UTC';  allF = [];
for k = 1:numel(years)
    yr = years(k);
    if numel(opts.localFiles) >= k && ~isempty(opts.localFiles{k})
        omni = get_omni2(yr, struct('localFile', opts.localFiles{k}, 'cacheDir', opts.cacheDir));
    else
        omni = get_omni2(yr, struct('cacheDir', opts.cacheDir));
    end
    daily = retime(omni(:,'f107'), 'daily', 'mean');   % F10.7 is daily -> collapse hourly
    allT  = [allT; daily.Time];                          %#ok<AGROW>
    allF  = [allF; daily.f107];                          %#ok<AGROW>
end

[allT, ix] = unique(allT);  allF = allF(ix);             % sort + dedup

% -------------------------------------------------------------------------
% THE CENTRED MEAN AND ITS EDGES -- read this before trusting F107_bar near
% the present day.
%
% 'Endpoints','shrink' does NOT return NaN at the edges: it silently shrinks the
% window. Mid-history that is harmless, because this function deliberately pulls
% whole OMNI2 year files spanning [start-41d, end+41d], so the window is full.
%
% But at the END of the available OMNI2 data -- i.e. TODAY -- there is no future
% to average over. The last ~40 days therefore get a progressively shrinking
% window, and at the newest sample the "81-day CENTRED mean" is really a ~41-day
% TRAILING mean, returned under the same name with no flag. For a hindcast that
% never bites. For a run at or near the present epoch -- exactly the case where
% you are trying to predict -- F107_bar is quietly a different quantity than the
% models were fitted on.
%
% So: carry the actual sample count per day. F107_bar_n < meanWindowDays means the
% window was clipped and that day's mean is NOT a true centred 81-day value.
% Downstream can then decide, instead of being lied to.
% -------------------------------------------------------------------------
f107bar = movmean(allF, opts.meanWindowDays, 'omitnan', 'Endpoints','shrink');
nbar    = movsum(double(~isnan(allF)), opts.meanWindowDays, 'Endpoints','shrink');

TT = timetable(allT, allF, f107bar, nbar, 'VariableNames', {'F107','F107_bar','F107_bar_n'});
TT.Properties.DimensionNames{1} = 'Time';
TT.Properties.VariableUnits = {'sfu','sfu','days'};
TT = TT(TT.Time >= s & TT.Time <= e, :);
if isempty(TT), error('get_f107: no F10.7 data in [%s,%s].', char(s), char(e)); end
TT.Properties.Description = 'NASA OMNI2 F10.7 (daily) + centered 81-day mean. UTC.';
fprintf('[F107/OMNI2] Done: %d days. F107(1)=%.1f  F107_bar(1)=%.1f sfu.\n', ...
        height(TT), TT.F107(1), TT.F107_bar(1));
clipped = TT.F107_bar_n < opts.meanWindowDays;
if any(clipped)
    warning('get_f107:centredMeanClipped', ...
      ['%d of %d days have a CLIPPED 81-day window (min %d days) -- their F107_bar is ' ...
       'NOT a true centred mean. This happens at the edge of the OMNI2 archive, i.e. ' ...
       'near the present epoch. See TT.F107_bar_n.'], ...
       sum(clipped), height(TT), min(TT.F107_bar_n));
end
end
