function TT = get_f30_cls(startDate, endDate, opts)
%GET_F30_CLS  Download the CLS/CNES F30 index (the one DTM2020_Res is built on)
%
% Input  : start/end date, opts (adjusted 1-AU vs observed)
% Process: queries the LISIRD JSON service for the CLS F30 series -> cleans gaps -> computes the
%          81-day centered mean
% Output : UTC timetable TT with native F30 and F30_bar
%
%             as a UTC timetable, with the 81-day centered mean (F30_bar).
%
% CANONICAL SOURCE & CONVENTION (CLS radio-flux service, Yaya et al. 2017,
% spaceweather.cls.fr -- the CNES/CLS product Bruinsma uses for DTM):
%   The Nobeyama 30 cm flux is distributed in TWO forms:
%     'adjusted' (1 AU)            <- USE THIS for density modelling (default)
%     'absolute' (real Sun-Earth   <- NOT for modelling; ~ a few % high near
%                 distance)            perihelion and on a different baseline
%   DTM2020_Res's F30->F10.7 regression (SWAMI eq. 2) was fitted on the
%   ADJUSTED series, so feed the adjusted product. The driver applies eq. 2
%   (f30_to_f107scale) before dtm5 -- do NOT rescale here.
%
% LITERATURE CONSISTENCY CHECK (use this to verify the product is right):
%   After eq. 2, rescaled F30 must TRACK F10.7 (they are tied by construction;
%   SWAMI even says "you can use F10.7 too"). For Nov 2009 the adjusted F30
%   rescales to ~73 == F10.7. If the rescaled F30 lands well above F10.7, the workflow
%   are on the wrong product (likely 'absolute') or a different calibration
%   vintage -- switch product, or use F10.7 (the sanctioned substitute).
%
% LISIRD mirrors CLS. Product is selected with opts.product:
%   'standard' (default) -> 'cls_radio_flux_f30'           (1-AU adjusted series)
%   'absolute'           -> 'cls_radio_flux_absolute_f30'  (real-distance series)
%
% HOW IT FETCHES: pulls the WHOLE series in one request (no server-side time
% filter -- that hangs the server), subsets to the range, and computes the
% centered 81-day mean in MATLAB. Whole series is ~1.5 MB.
%
% USAGE
%   TT = get_f30_cls('2009-11-01','2009-11-30');                 % adjusted (canonical)
%   TT = get_f30_cls(..., struct('product','absolute'));         % compare only
%   TT = get_f30_cls(..., struct('url', <CLS adjusted file>));   % point at a CLS file
%
% INPUTS
%   startDate, endDate - 'YYYY-MM-DD' strings or datetimes (UTC)
%   opts (optional):
%     .product        (default 'standard') - 'standard' (adjusted) | 'absolute'
%     .localFile      (default '')         - parse a saved CSV instead of downloading
%     .meanWindowDays (default 81)         - centered window for F30_bar
%     .timeoutSec     (default 180)        - download timeout
%     .url            (default per product)- override the source URL if needed
%
% OUTPUT  TT - timetable (UTC daily): Time, F30 [sfu], F30_bar [sfu]

if nargin < 3, opts = struct(); end
if ~isfield(opts,'localFile'),      opts.localFile      = '';  end
if ~isfield(opts,'meanWindowDays'), opts.meanWindowDays = 81;  end
if ~isfield(opts,'timeoutSec'),     opts.timeoutSec     = 180; end
if ~isfield(opts,'product'),        opts.product        = 'standard'; end

switch lower(opts.product)
    case 'standard', dsid = 'cls_radio_flux_f30';            % 1-AU adjusted (canonical)
    case 'absolute', dsid = 'cls_radio_flux_absolute_f30';   % real distance (compare only)
    otherwise, error('get_f30_cls: opts.product must be ''standard'' or ''absolute''.');
end
if ~isfield(opts,'url') || isempty(opts.url)
    opts.url = sprintf('https://lasp.colorado.edu/lisird/latis/dap/%s.csv', dsid);
end

fprintf('[CLS-F30] product=%s  (id=%s, whole-series fetch)\n', opts.product, dsid);

s = datetime(startDate,'TimeZone','UTC');
e = datetime(endDate,  'TimeZone','UTC');

% ---- get the CSV (whole series; no time filter) -----------------------
if ~isempty(opts.localFile)
    datfile = opts.localFile;
    fprintf('[CLS-F30] Reading local file: %s\n', datfile);
else
    datfile = [tempname '.csv'];
    fprintf('[CLS-F30] Downloading full F30 series (~1.5 MB) ...\n');
    try
        websave(datfile, opts.url, weboptions('Timeout', opts.timeoutSec));
    catch ME
        error('get_f30_cls:download', ...
              ['Download failed (%s).\n' ...
               'Open this URL in a browser, save the file, pass it as opts.localFile:\n' ...
               '  %s\n' ...
               'Canonical CLS source (adjusted file): ' ...
               'https://spaceweather.cls.fr/services/radioflux/  (Solar flux archive)'], ...
              ME.message, opts.url);
    end
end

% ---- parse: header line, then "yyyy MM dd,<f30>,..." -------------------
raw   = fileread(datfile);
lines = regexp(raw, '\r\n|\r|\n', 'split');
lines = lines(~cellfun(@isempty, strtrim(lines)));

t   = datetime.empty(0,1);
f30 = [];
for i = 1:numel(lines)
    parts = strsplit(strtrim(lines{i}), ',');
    if numel(parts) < 2, continue; end
    dnum = sscanf(parts{1}, '%d %d %d');      % date field "yyyy MM dd"
    if numel(dnum) ~= 3, continue; end        % skips header line
    val = str2double(parts{2});               % F30 flux column [sfu]
    if isnan(val), continue; end
    t(end+1,1)   = datetime(dnum(1),dnum(2),dnum(3));  %#ok<AGROW>
    f30(end+1,1) = val;                                %#ok<AGROW>
end
if isempty(f30), error('get_f30_cls: no rows parsed; check the CSV format.'); end
t.TimeZone = 'UTC';

[t, ix] = sort(t);  f30 = f30(ix);            % chronological (be safe)

% ---- 81-day centered mean on the FULL series, then subset -------------
f30bar = movmean(f30, opts.meanWindowDays, 'omitnan', 'Endpoints','shrink');

TT = timetable(t, f30, f30bar, 'VariableNames', {'F30','F30_bar'});
TT.Properties.DimensionNames{1} = 'Time';
TT.Properties.VariableUnits = {'sfu','sfu'};

TT = TT(TT.Time >= s & TT.Time <= e, :);      % trim to requested range
if isempty(TT)
    error('get_f30_cls: no data in [%s, %s]. Series covers 1957-11 to present.', ...
          char(s), char(e));
end
TT.Properties.Description = sprintf('CLS/LISIRD F30 (%s) + %dd mean. UTC.', ...
                                    opts.product, opts.meanWindowDays);
fprintf('[CLS-F30] Done: %d days. F30(1)=%.1f  F30_bar(1)=%.1f sfu (%s).\n', ...
        height(TT), TT.F30(1), TT.F30_bar(1), opts.product);
end
