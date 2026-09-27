function TT = get_omni2(year, opts)
%GET_OMNI2  Download a full year of NASA OMNI2 hourly data as a TIMETABLE.
%
% Input  : year, opts (cache dir, force)
% Process: downloads omni2_YYYY.dat from NASA/SPDF -> fixed-width parse of all 55 variables -> flags
%          fill values as NaN
% Output : hourly UTC timetable TT with all 55 OMNI2 variables
%
%
% Returns a timetable whose row index is the UTC timestamp of each hour and
% whose columns are ALL 55 OMNI2 variables. Because it is a timetable, every
% variable is paired with its date/time: opening it shows Time next to
% each column, and it plots directly with stackedplot.
%
% USAGE
%   TT = get_omni2(2009);                 % download + parse 2009
%   save('omni2_2009.mat','TT');          % save the timetable
%   stackedplot(TT, {'f107','Kp','ap'});  % quick look
%   TT.f107            % a single variable (with TT.Time alongside)
%   TT('2009-06-01',:) % index by date
%
% INPUT
%   year - 4-digit year (>=1963). The only required input.
%   opts - (optional) struct:
%            .localFile (default '')    - parse this local file instead of
%                                         downloading (offline use)
%            .keepFill  (default false) - keep raw fill values (no NaN, no Kp scale)
%
% OUTPUT  TT - timetable, 8760x55 (8784 in leap years):
%   Row index : Time  (datetime, UTC, start of each hour)
%   Columns   : year doy hour bartels id_imf id_sw n_imf n_plasma
%               B_mag_avg B_vec_mag B_lat B_long Bx_gse By_gse Bz_gse By_gsm Bz_gsm
%               sigma_B_mag sigma_B sigma_Bx sigma_By sigma_Bz
%               proton_temp proton_density flow_speed flow_long flow_lat na_np flow_pressure
%               sigma_T sigma_N sigma_V sigma_phi_V sigma_theta_V sigma_na_np
%               E_field plasma_beta alfven_mach Kp R_sunspot Dst AE
%               pflux_gt1 pflux_gt2 pflux_gt4 pflux_gt10 pflux_gt30 pflux_gt60
%               flag ap f107 pcn AL AU magnetosonic_mach
%
%   Kp is in DECIMAL form (3+ stored as 33 -> 3.3). f107 in sfu, ap in nT.
%   Fill values are converted to NaN. Variable units are set where known.

if nargin < 2, opts = struct(); end
if ~isfield(opts,'localFile'), opts.localFile = '';    end
if ~isfield(opts,'cacheDir'),  opts.cacheDir  = '';    end   % '' -> no cache (old behaviour)
if ~isfield(opts,'keepFill'),  opts.keepFill  = false; end

% ---- field definitions: {name, fill value, unit} (order = OMNI2 words 1-55)
F = {
 'year',NaN,'';            'doy',NaN,'day';          'hour',NaN,'hour';
 'bartels',9999,'';        'id_imf',99,'';           'id_sw',99,'';
 'n_imf',999,'';           'n_plasma',999,'';
 'B_mag_avg',999.9,'nT';   'B_vec_mag',999.9,'nT';   'B_lat',999.9,'deg';   'B_long',999.9,'deg';
 'Bx_gse',999.9,'nT';      'By_gse',999.9,'nT';      'Bz_gse',999.9,'nT';
 'By_gsm',999.9,'nT';      'Bz_gsm',999.9,'nT';
 'sigma_B_mag',999.9,'nT'; 'sigma_B',999.9,'nT';     'sigma_Bx',999.9,'nT';
 'sigma_By',999.9,'nT';    'sigma_Bz',999.9,'nT';
 'proton_temp',9999999,'K';'proton_density',999.9,'n/cc'; 'flow_speed',9999,'km/s';
 'flow_long',999.9,'deg';  'flow_lat',999.9,'deg';   'na_np',9.999,'';      'flow_pressure',99.99,'nPa';
 'sigma_T',9999999,'K';    'sigma_N',999.9,'n/cc';   'sigma_V',9999,'km/s';
 'sigma_phi_V',999.9,'deg';'sigma_theta_V',999.9,'deg'; 'sigma_na_np',9.999,'';
 'E_field',999.99,'mV/m';  'plasma_beta',999.99,'';  'alfven_mach',999.9,'';
 'Kp',99,'';               'R_sunspot',999,'';       'Dst',99999,'nT';      'AE',9999,'nT';
 'pflux_gt1',999999.99,'1/cm2-s-sr';  'pflux_gt2',99999.99,'1/cm2-s-sr';
 'pflux_gt4',99999.99,'1/cm2-s-sr';   'pflux_gt10',99999.99,'1/cm2-s-sr';
 'pflux_gt30',99999.99,'1/cm2-s-sr';  'pflux_gt60',99999.99,'1/cm2-s-sr';
 'flag',NaN,'';            'ap',999,'nT';            'f107',999.9,'sfu';    'pcn',999.9,'';
 'AL',99999,'nT';          'AU',99999,'nT';          'magnetosonic_mach',99.9,'';
};
names = F(:,1);
units = F(:,3);
ncol  = numel(names);   % 55

% ---- get the file ------------------------------------------------------
if ~isempty(opts.localFile)
    datfile = opts.localFile;
    fprintf('[OMNI2] Reading local file: %s\n', datfile);
else
    url = sprintf('https://spdf.gsfc.nasa.gov/pub/data/omni/low_res_omni/omni2_%04d.dat', year);
    % ---------------------------------------------------------------------
    % CACHING. This used to be  datfile = [tempname '.dat'];  -- i.e. every call
    % re-downloaded ~20 MB of OMNI2 to a throwaway temp file. Nothing was ever
    % reused, despite data.spaceweather's header promising "both sub-fetchers
    % cache into <data.root>/spaceweather". A compare_OD sweep calls buildWorld
    % once per config, so an N-config sweep re-downloaded the same two year files
    % N times.
    %
    % OMNI2 year files for PAST years are immutable, so they can be cached
    % forever. The CURRENT year is still being appended to, so it is re-fetched
    % when the cache copy is older than maxAgeDays.
    % ---------------------------------------------------------------------
    if isempty(opts.cacheDir)
        datfile = [tempname '.dat'];   % no cache dir given: old behaviour
        stale   = true;
    else
        if ~exist(opts.cacheDir,'dir'), mkdir(opts.cacheDir); end
        datfile = fullfile(opts.cacheDir, sprintf('omni2_%04d.dat', year));
        stale   = ~exist(datfile,'file');
        if ~stale
            cy = clock(); 
            if year >= cy(1)               % current (or future) year: still growing
                d = dir(datfile);
                stale = (now - d.datenum) > 1;    % refresh a live year daily
            end
        end
    end
    if stale
        fprintf('[OMNI2] Downloading %s ...\n', url);
        try
            websave(datfile, url);
        catch ME
            if exist(datfile,'file')
                warning('get_omni2:downloadStale', ...
                    'Download failed (%s); using the cached copy of %d.', ...
                    regexprep(ME.message,'\n.*',''), year);
            else
                error('get_omni2:download', ...
                  'Download failed (%s). Check year/connection or pass opts.localFile.', ME.message);
            end
        end
    else
        fprintf('[OMNI2] cache: omni2_%04d.dat\n', year);
    end
end

% ---- read & parse (whitespace tokens; OMNI2 has a leading space/field) --
raw   = fileread(datfile);
lines = regexp(raw, '\r\n|\r|\n', 'split');
lines = lines(~cellfun(@isempty, strtrim(lines)));
nrow  = numel(lines);

M    = nan(nrow, ncol);
nbad = 0;
for i = 1:nrow
    v = sscanf(lines{i}, '%f');
    if numel(v) >= ncol
        M(i,:) = v(1:ncol)';
    else
        nbad = nbad + 1;
    end
end
fprintf('[OMNI2] Parsed %d rows (%d skipped).\n', nrow-nbad, nbad);

% ---- fill -> NaN, Kp -> decimal ---------------------------------------
if ~opts.keepFill
    for k = 1:ncol
        fv = F{k,2};
        if ~isnan(fv)
            col = M(:,k);
            col(col == fv) = NaN;
            M(:,k) = col;
        end
    end
    kpcol = strcmp(names,'Kp');
    M(:,kpcol) = M(:,kpcol) / 10;   % 33->3.3, 57->5.7, 40->4.0
end

% ---- UTC time vector from year/doy/hour --------------------------------
yv = M(:, strcmp(names,'year'));
dv = M(:, strcmp(names,'doy'));
hv = M(:, strcmp(names,'hour'));
time = datetime(yv,1,1) + days(dv - 1) + hours(hv);
time.TimeZone = 'UTC';

% ---- build timetable ---------------------------------------------------
TT = array2timetable(M, 'RowTimes', time, 'VariableNames', names);
TT.Properties.VariableUnits = units;
TT.Properties.DimensionNames{1} = 'Time';
TT.Properties.Description = sprintf('NASA OMNI2 hourly data, year %d (UTC).', year);

fprintf('[OMNI2] Done: %d hours x %d variables (timetable).\n', height(TT), width(TT));
end
