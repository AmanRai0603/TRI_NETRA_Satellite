function T = fetch_tudelft_density(satellite, startDate, endDate, opts)
%FETCH_TUDELFT_DENSITY  Download TU Delft thermosphere density data in a loop.
%
% Input  : satellite name, start/end date, opts (version, download dir, force re-download)
% Process: lists the TU Delft web directory -> selects the monthly files covering the range ->
%          downloads and unzips -> parses each file via read_tudelft_density_file -> concatenates,
%          crops to the range, caches as .mat
% Output : timetable T of measured density (and winds where present) at native 10 s resolution
%
%
% Lists the TU Delft web directory for a satellite, discovers the monthly files
% covering the requested range (no filename guessing), downloads + unzips, parses each via
% read_tudelft_density_file, concatenates, restricts to [startDate,endDate], saves.
%
% Source (Apache web server, replaced the old FTP in Dec 2025):
%   https://thermosphere.tudelft.nl/data/data/version_0X/<SAT>_data/
%
% USAGE
%   T = fetch_tudelft_density('GOCE','2009-11-01','2009-11-30');
%   T = fetch_tudelft_density('CHAMP','2003-01-01','2003-01-31');
%   T = fetch_tudelft_density('Swarm-A','2014-01-01','2014-01-31', struct('dataType','POD'));
%
% INPUTS
%   satellite : 'GOCE' | 'CHAMP' | 'GRACE' | 'GRACE-FO' | 'Swarm-A/B/C'
%   startDate, endDate : 'YYYY-MM-DD' or datetime
%   opts (optional):
%     .dataType   'ACC' (accelerometer, default) | 'POD' (GNSS/GPS-based)
%                 NOTE: Swarm density is usually POD -> set dataType='POD' for Swarm.
%     .version    2 (default, latest reprocessing) | 1
%     .folder     override the directory name on the server (e.g. 'Swarm_data')
%     .outDir     where to save the .mat and downloads (default pwd)
%     .downsample keep every Nth row after parsing (default 1 = all)
%     .save       true (default) -> write <SAT>_<TYPE>_<start>_<end>.mat
%
% OUTPUT  T - table (DateTime, Altitude_m, Longitude_deg, Latitude_deg, LST_h,
%             Density_kgm3, [DensityMean_kgm3], [ArgLat_deg], flags).

if nargin < 4, opts = struct(); end
if ~isfield(opts,'dataType'),   opts.dataType   = 'ACC'; end
if ~isfield(opts,'version'),    opts.version    = 2;     end
if ~isfield(opts,'outDir'),     opts.outDir     = pwd;   end
if ~isfield(opts,'downsample'), opts.downsample = 1;     end
if ~isfield(opts,'save'),       opts.save       = true;  end
if ~isfield(opts,'folder'),     opts.folder     = '';    end

s = datetime(startDate,'TimeZone','UTC');
e = datetime(endDate,  'TimeZone','UTC');
dtype = upper(opts.dataType);

% ---- satellite -> CANDIDATE directory folders + (optional) Swarm letter ----
% TU Delft uses 2-letter mission prefixes in the FILE names: GO (GOCE), CH (CHAMP),
% GR/GA (GRACE), GC (GRACE-FO = GRACE-C), SA/SB/SC (Swarm-A/B/C). The DIRECTORY is
% "<SAT>_data". Folder names can vary slightly, so we try a few and use opts.folder
% to force one.
satU = upper(strrep(satellite,' ',''));
swarmLetter = '';
% which GRACE twin? '' = caller said plain 'GRACE' and accepts either.
graceTwin = '';
if any(strcmp(satU,{'GRACE-A','GRACE-1'})), graceTwin = 'A'; end
if any(strcmp(satU,{'GRACE-B','GRACE-2'})), graceTwin = 'B'; end

if startsWith(satU,'SWARM')
    folders = {'Swarm_data','SWARM_data','Swarm'};
    if     contains(satU,'-A')||endsWith(satU,'A'), swarmLetter = 'A';
    elseif contains(satU,'-B')||endsWith(satU,'B'), swarmLetter = 'B';
    elseif contains(satU,'-C')||endsWith(satU,'C'), swarmLetter = 'C';
    end
else
    switch satU
        case 'GOCE',                            folders = {'GOCE_data','GOCE'};
        case 'CHAMP',                           folders = {'CHAMP_data','CHAMP','Champ_data'};
        case {'GRACE','GRACE-A','GRACE-B','GRACE-1','GRACE-2'}
            folders = {'GRACE_data','GRACE'};
        case {'GRACE-FO','GRACEFO','GRACE_FO'}, folders = {'GRACE-FO_data','GRACEFO_data','GRACE_FO_data','GRACE-FO'};
        otherwise, error('fetch_tudelft_density:sat','Unknown satellite "%s".', satellite);
    end
end
if ~isempty(opts.folder), folders = {opts.folder}; end

% ---- 1) list the directory (try candidates, pick the first that returns links) ----
html = ''; baseUrl = ''; lastErr = '';
for fi = 1:numel(folders)
    bu = sprintf('https://thermosphere.tudelft.nl/data/data/version_%02d/%s/', opts.version, folders{fi});
    try
        h = webread(bu, weboptions('Timeout',120,'ContentType','text'));
    catch ME
        lastErr = ME.message; continue;
    end
    if ~isempty(regexp(h, '\.(?:zip|txt)', 'once'))
        html = h; baseUrl = bu; break;
    end
end
if isempty(html)
    error('fetch_tudelft_density:list', ...
        ['Could not list a density folder for %s under version_%02d.\n' ...
         'Tried folder(s): %s   (last error: %s)\n' ...
         'Open https://thermosphere.tudelft.nl/data/data/version_%02d/ in a browser to read the\n' ...
         'EXACT folder name, then pass it via opts.folder = ''<name>''.'], ...
         satellite, opts.version, strjoin(folders,', '), lastErr, opts.version);
end
fprintf('[TUD] directory: %s\n', baseUrl);

% ---- 2) discover candidate files (href targets + bare names), .zip/.txt ----
raw   = regexp(html, 'href\s*=\s*"([^"]+\.(?:zip|txt))"', 'tokens');
cand  = {};
for i = 1:numel(raw), [~,nm,ext] = fileparts(raw{i}{1}); cand{end+1} = [nm ext]; end %#ok<AGROW>
bare  = regexp(html, '[A-Za-z0-9][\w\-.]*\.(?:zip|txt)', 'match');
cand  = unique([cand, bare]);
fprintf('[TUD] %d file link(s) listed.\n', numel(cand));

% month-granularity range
s_ym = year(s)*12 + month(s);  e_ym = year(e)*12 + month(e);

% ---- 3) filter to DENSITY files of the requested method + month range ----
% The version tag _vNN[a-z] is OPTIONAL (some products ship un-versioned); when
% present we rank by it (v02 < v02b < v02c) and keep only the NEWEST per month.
want = strings(0,1); wYear = []; wMon = []; wRank = [];
for k = 1:numel(cand)
    fname = cand{k};
    % year-month: CHAMP uses a HYPHEN (2009-11), the others underscore (2009_11)
    ym = regexp(fname, '[-_](\d{4})[-_](\d{2})(?=[-_.])', 'tokens', 'once');
    if isempty(ym), continue; end
    yy = str2double(ym{1});  mm = str2double(ym{2});
    if mm < 1 || mm > 12, continue; end
    vt = regexp(fname, '[-_]v(\d{2})([a-z]?)', 'tokens', 'once');
    if isempty(vt)
        rank = 100;                                   % un-versioned -> base rank
    else
        vlet = vt{2};  lrank = 0;
        if ~isempty(vlet), lrank = double(vlet)-double('a')+1; end
        rank = str2double(vt{1})*100 + lrank;         % v02=200, v02b=202, v02c=203
    end
    if (yy*12+mm) < s_ym || (yy*12+mm) > e_ym,            continue; end   % in range (month)
    if isempty(regexp(upper(fname),'_DNS_','once')),      continue; end   % DENSITY only (skip _WND_)

    % ---- GRACE-A vs GRACE-B -------------------------------------------------
    % TU Delft prefixes the FILES GA_ / GB_. This filter did not exist: the
    % satellite argument only understood 'GRACE', so ANY GRACE density request
    % took whichever file listed first -- the GA_ ones. GRACE-B was therefore
    % silently validated against GRACE-A's density. They fly ~200 km apart, which
    % is exactly the 247 km track mismatch that exposed this.
    % Returning the WRONG SATELLITE's data is worse than returning none, so if the
    % requested twin has no files we stop rather than substitute.
    if ~isempty(graceTwin)
        if isempty(regexp(upper(fname), ['(^|[^A-Z0-9])G' graceTwin '[_\W]'], 'once'))
            continue
        end
    end
    if ~isempty(dtype) && isempty(regexp(upper(fname),dtype,'once')), continue; end  % ACC vs POD
    if ~isempty(swarmLetter)                                            % Swarm A/B/C selector
        L = swarmLetter;
        ok = ~isempty(regexp(fname, ['(^|[^A-Za-z0-9])S' L '[_\W]'], 'once')) ...  % SA_/SB_/SC_ prefix
          || ~isempty(regexp(fname, ['SW_?' L '[_\W]'], 'once')) ...               % SWA_/SW_A_
          || ~isempty(regexp(fname, ['DNS_?' L '[_\W]'], 'once')) ...              % DNSA/DNS_A
          || ~isempty(regexp(fname, ['_' L '_'], 'once'));                          % _A_ anywhere
        if ~ok, continue; end
    end
    want(end+1,1) = string(fname);  wYear(end+1,1) = yy;  wMon(end+1,1) = mm;  wRank(end+1,1) = rank; %#ok<AGROW>
end

% keep only the NEWEST reprocessing vintage per (year,month) -> no triplicated rows
if ~isempty(want)
    [~,o]  = sortrows([wYear wMon wRank], [1 2 -3]);  want = want(o); wYear = wYear(o); wMon = wMon(o);
    [~,iu] = unique([wYear wMon], 'rows', 'stable');  want = want(iu); wYear = wYear(iu); wMon = wMon(iu);
    [~,o2] = sortrows([wYear wMon]);                  want = want(o2);
end

% ---- diagnostics if nothing matched: show what IS there ----
if isempty(want)
    dnsAll = cand(~cellfun(@isempty, regexp(upper(cand),'_DNS_','once')));
    nshow  = @(c) strjoin(c(1:min(numel(c),12)), sprintf('\n   '));
    if isempty(dnsAll)
        error('fetch_tudelft_density:none', ...
            ['No density (_DNS_) files in this directory for %s.\n' ...
             'The folder listed %d file(s); first few:\n   %s\nBrowse: %s'], ...
             satellite, numel(cand), nshow(cand), baseUrl);
    else
        hasACC = any(~cellfun(@isempty, regexp(upper(dnsAll),'ACC','once')));
        hasPOD = any(~cellfun(@isempty, regexp(upper(dnsAll),'POD','once')));
        avail  = strtrim([repmat('ACC ',1,hasACC) repmat('POD ',1,hasPOD)]);
        extra  = '';
        if ~isempty(swarmLetter), extra = sprintf('\nSwarm letter filter = %s -> check the prefix in the names above.', swarmLetter); end
        error('fetch_tudelft_density:none', ...
            ['Density files exist but none matched your filters (%s, %s..%s).\n' ...
             'Example density files here:\n   %s\n' ...
             'Method tags available in this folder: %s   (you requested %s).%s\nBrowse: %s'], ...
             dtype, char(s), char(e), nshow(dnsAll), avail, dtype, extra, baseUrl);
    end
end
fprintf('[TUD] %d monthly file(s) to fetch: %s\n', numel(want), strjoin(cellstr(want),', '));

% ---- 4) download + unzip + parse each month (unchanged) ----
dl = fullfile(opts.outDir,'tudelft_downloads');  if ~exist(dl,'dir'), mkdir(dl); end
T = table;
for k = 1:numel(want)
    fname = char(want(k));  url = [baseUrl fname];  local = fullfile(dl,fname);
    if ~exist(local,'file')
        fprintf('[TUD] downloading %s ...\n', fname);
        websave(local, url, weboptions('Timeout',600));
    else
        fprintf('[TUD] using cached %s\n', fname);
    end
    if endsWith(lower(fname),'.zip')
        names = unzip(local, dl);
        txt = names(endsWith(lower(names),'.txt'));
        base = erase(fname,'.zip');
        keep = (contains(lower(txt),lower(base)) | contains(lower(txt),'_dns_')) & ...
               ~contains(lower(txt),'license') & ~contains(lower(txt),'readme') & ...
               ~contains(lower(txt),'copying');
        if any(keep), txt = txt(keep); end
        if isempty(txt), warning('No data .txt inside %s',fname); continue; end
        datafile = txt{1};
    else
        datafile = local;
    end
    Ti = read_tudelft_density_file(datafile);
    T  = [T; Ti];  %#ok<AGROW>
end

% ---- 5) restrict, downsample, save (unchanged) ----
T = sortrows(T,'DateTime');
T = T(T.DateTime >= s & T.DateTime <= e, :);
if opts.downsample > 1, T = T(1:opts.downsample:end, :); end
if isempty(T), error('fetch_tudelft_density: no rows in [%s,%s] after parse.', char(s),char(e)); end

fprintf('[TUD] DONE: %d rows, %s..%s, alt %.0f-%.0f km.\n', height(T), ...
        char(min(T.DateTime)), char(max(T.DateTime)), ...
        min(T.Altitude_m)/1000, max(T.Altitude_m)/1000);

if opts.save
    tag = sprintf('%s_%s_%s_%s', satU, dtype, datestr(s,'yyyymmdd'), datestr(e,'yyyymmdd'));
    matfile = fullfile(opts.outDir, [tag '.mat']);
    save(matfile, 'T');
    fprintf('[TUD] saved %s\n', matfile);
end
end
