function idx = get_jb2008_indices(varargin)
%GET_JB2008_INDICES  Fetch & parse the JB2008 space-weather files from SET.
%
% Input  : optional date range and opts (cache dir, force)
% Process: downloads SOLFSMY.TXT (daily F10,S10,M10,Y10 + 81-day means) and DTCFILE.TXT (hourly
%          Dst-driven dTc) -> parses fixed formats -> crops to range
% Output : struct idx with daily solar-index timetable and hourly DSTDTC timetable, ready for
%          jb2008_density
%
%
%   JB2008 does NOT run on F10.7/Kp like the operational DTM. It needs four solar
%   indices (F10, S10, M10, Y10 + their 81-day centered means) and a Dst-driven
%   exospheric-temperature correction DSTDTC. These come from two files published
%   by Space Environment Technologies (SET):
%       SOLFSMY.TXT  - daily solar indices  (F10 F81 S10 S81 M10 M81 Y10 Y81)
%       DTCFILE.TXT  - hourly DSTDTC [K]
%   Source:  https://sol.spacenvironment.net/jb2008/indices/
%
%   NOTE (2025): SET re-derived S10 in May 2025 (it was too low at solar maximum),
%   so re-fetch these files for any 2024-2025 run rather than using a stale copy.
%
%   USAGE
%     idx = get_jb2008_indices();                       % download (cache in ./jb2008_data)
%     idx = get_jb2008_indices('cacheDir','C:\sw\jb');  % choose cache folder
%     idx = get_jb2008_indices('forceDownload',true);   % ignore cache, re-fetch
%     idx = get_jb2008_indices('solfsmyFile',f1,'dtcFile',f2);  % use local files (offline)
%
%   OUTPUT  idx struct:
%     .sol  - timetable (daily, UTC): F10 F81 S10 S81 M10 M81 Y10 Y81  [sfu / scaled]
%     .dtc  - timetable (hourly, UTC): DTC  [K]
%     .span - [firstDate lastDate] covered (datetime)
%
%   Feed jb2008_density.m with this struct. Only runs in real MATLAB (web + datetime).

ip = inputParser;
ip.addParameter('cacheDir', fullfile(pwd,'jb2008_data'), @(x)ischar(x)||isstring(x));
ip.addParameter('forceDownload', false, @islogical);
ip.addParameter('solfsmyFile', '', @(x)ischar(x)||isstring(x));
ip.addParameter('dtcFile',     '', @(x)ischar(x)||isstring(x));
ip.addParameter('maxAgeDays',  3,  @isnumeric);          % re-download if cache older than this
ip.parse(varargin{:});
o = ip.Results;

base = 'https://sol.spacenvironment.net/jb2008/indices/';   % SET (http:// also works)
if ~exist(o.cacheDir,'dir'), mkdir(o.cacheDir); end

solPath = char(o.solfsmyFile);  dtcPath = char(o.dtcFile);
if isempty(solPath), solPath = local_or_download([base 'SOLFSMY.TXT'], fullfile(o.cacheDir,'SOLFSMY.TXT'), o); end
if isempty(dtcPath), dtcPath = local_or_download([base 'DTCFILE.TXT'], fullfile(o.cacheDir,'DTCFILE.TXT'), o); end

idx.sol  = parse_solfsmy(solPath);
idx.dtc  = parse_dtcfile(dtcPath);
idx.span = [max(idx.sol.Time(1), idx.dtc.Time(1)), min(idx.sol.Time(end), idx.dtc.Time(end))];
fprintf('[JB2008-sw] SOLFSMY %d days, DTCFILE %d hours. Overlap %s .. %s\n', ...
        height(idx.sol), height(idx.dtc), datestr(idx.span(1),'yyyy-mm-dd'), datestr(idx.span(2),'yyyy-mm-dd'));
end

% --------------------------------------------------------------------------
function p = local_or_download(url, p, o)
fresh = exist(p,'file') && ~o.forceDownload;
if fresh
    d = dir(p);
    if (now - d.datenum) > o.maxAgeDays, fresh = false; end
end
if fresh
    fprintf('[JB2008-sw] using cached %s\n', p); return
end

% SET's server returns 403 Forbidden to non-browser clients (it blocks websave's
% default "MATLAB" User-Agent). Send browser headers, and try http:// and the
% capital /JB2008/ path as fallbacks. Each URL is tried via websave first, then
% via matlab.net.http (more header control).
urls = url_variants(url);
ua   = 'Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/124.0.0.0 Safari/537.36';
ok = false;  lastErr = '(no attempt)';
for k = 1:numel(urls)
    u = urls{k};
    fprintf('[JB2008-sw] downloading %s ...\n', u);
    try   % attempt 1: websave with browser User-Agent + Accept
        wo = weboptions('Timeout',90,'ContentType','text','UserAgent',ua, ...
                        'HeaderFields',{'Accept','text/plain,*/*';'Accept-Language','en-US,en;q=0.9'});
        websave(p, u, wo);  ok = check_download(p);
    catch ME1, lastErr = ME1.message; ok = false; end
    if ok, fprintf('[JB2008-sw]   ok via websave <- %s\n', u); break; end
    try   % attempt 2: matlab.net.http with a full browser header set
        ok = http_get_to_file(u, p, ua);
    catch ME2, lastErr = ME2.message; ok = false; end
    if ok, fprintf('[JB2008-sw]   ok via http <- %s\n', u); break; end
end

if ~ok
    if exist(p,'file')
        warning('get_jb2008_indices:dl','all downloads failed (%s) -> using stale cache %s', lastErr, p);
    else
        [~,nm,ext] = fileparts(p);
        error('get_jb2008_indices:dl', ['could not download %s (last error: %s).\n' ...
              'SET blocks some clients. Open these in a browser, Save As, then pass the local files:\n' ...
              '    https://sol.spacenvironment.net/JB2008/indices/SOLFSMY.TXT\n' ...
              '    https://sol.spacenvironment.net/JB2008/indices/DTCFILE.TXT\n' ...
              '    idx = get_jb2008_indices(''solfsmyFile'',''C:\\path\\SOLFSMY.TXT'',''dtcFile'',''C:\\path\\DTCFILE.TXT'');\n' ...
              '(missing file here: %s%s)'], url, lastErr, nm, ext);
    end
end
end

% --------------------------------------------------------------------------
function urls = url_variants(url)
% original first, then http:// and capital /JB2008/ swaps (dedup, order-preserving)
cand = {url, strrep(url,'jb2008','JB2008'), strrep(url,'https://','http://'), ...
        strrep(strrep(url,'https://','http://'),'jb2008','JB2008')};
urls = {};
for i = 1:numel(cand)
    if ~any(strcmp(urls, cand{i})), urls{end+1} = cand{i}; end %#ok<AGROW>
end
end

% --------------------------------------------------------------------------
function ok = http_get_to_file(u, p, ua)
% Low-level GET with explicit browser headers (works when websave's UA is ignored).
ok  = false;
hdr = matlab.net.http.HeaderField('User-Agent', ua, ...
                                  'Accept','text/plain,text/html,*/*', ...
                                  'Accept-Language','en-US,en;q=0.9', ...
                                  'Connection','keep-alive');
req  = matlab.net.http.RequestMessage('GET', hdr);
opts = matlab.net.http.HTTPOptions('ConnectTimeout',90);
resp = send(req, matlab.net.URI(u), opts);
if resp.StatusCode == matlab.net.http.StatusCode.OK
    body = resp.Body.Data;
    fid = fopen(p,'w');
    if ischar(body)||isstring(body), fwrite(fid, char(body));
    elseif isa(body,'uint8'),        fwrite(fid, body);
    else,                            fwrite(fid, char(string(body))); end
    fclose(fid);
    ok = check_download(p);
end
end

% --------------------------------------------------------------------------
function ok = check_download(p)
% Accept only a real data file: non-trivial size and not an HTML error page.
ok = false;
if exist(p,'file')
    d = dir(p);
    if d.bytes > 500
        fid = fopen(p,'r'); head = fread(fid, min(d.bytes,512), '*char')'; fclose(fid);
        if isempty(regexpi(head,'<html|<!doctype|403 Forbidden|Access Denied|Not Found','once'))
            ok = true;
        end
    end
end
end

% --------------------------------------------------------------------------
function TT = parse_solfsmy(fpath)
% data lines: YYYY DDD JD F10 F81 S10 S81 M10 M81 Y10 Y81 Ssrc...
txt = readlines_compat(fpath);
n = numel(txt);
D = nan(n,9);  k = 0;
for i = 1:n
    L = strtrim(txt{i});
    if isempty(L) || L(1)=='#' || L(1)=='%', continue; end
    t = sscanf(strrep(L,char(9),' '), '%f');           % numeric leading tokens (Ssrc may be non-numeric -> stops)
    if numel(t) >= 11 && t(1) > 1900 && t(1) < 2100
        k = k + 1;
        yr = t(1); doy = t(2);
        D(k,:) = [datenum(yr,1,1)+doy-1, t(4),t(5),t(6),t(7),t(8),t(9),t(10),t(11)];
    end
end
D = D(1:k,:);
if k==0, error('parse_solfsmy:empty','no SOLFSMY data lines parsed from %s (check format).', fpath); end
Time = datetime(D(:,1),'ConvertFrom','datenum','TimeZone','UTC');
TT = timetable(Time, D(:,2),D(:,3),D(:,4),D(:,5),D(:,6),D(:,7),D(:,8),D(:,9), ...
     'VariableNames', {'F10','F81','S10','S81','M10','M81','Y10','Y81'});
TT = sortrows(TT);
fprintf('[JB2008-sw] SOLFSMY parsed %d days (%s..%s). F10(1)=%.1f S10(1)=%.1f M10(1)=%.1f Y10(1)=%.1f\n', ...
        k, datestr(D(1,1),'yyyy-mm-dd'), datestr(D(end,1),'yyyy-mm-dd'), D(1,2),D(1,4),D(1,6),D(1,8));
end

% --------------------------------------------------------------------------
function TT = parse_dtcfile(fpath)
% data lines: DTC YYYY DDD  h0 h1 ... h23   (24 hourly DTC values, K)
txt = readlines_compat(fpath);
T = []; V = [];
for i = 1:numel(txt)
    L = strtrim(txt{i});
    if isempty(L) || ~strncmpi(L,'DTC',3), continue; end
    p = sscanf(strrep(L(4:end),char(9),' '), '%f');     % YYYY DDD then 24 values
    if numel(p) >= 26
        yr = p(1); doy = p(2); vals = p(3:26);
        base = datenum(yr,1,1)+doy-1;
        T = [T; base + (0:23)'/24];
        V = [V; vals(:)];
    end
end
if isempty(T), error('parse_dtcfile:empty','no DTC lines parsed from %s (check format).', fpath); end
Time = datetime(T,'ConvertFrom','datenum','TimeZone','UTC');
TT = sortrows(timetable(Time, V, 'VariableNames', {'DTC'}));
fprintf('[JB2008-sw] DTCFILE parsed %d hours (%s..%s). DTC range %d..%d K\n', ...
        numel(T), datestr(T(1),'yyyy-mm-dd'), datestr(T(end),'yyyy-mm-dd'), min(V), max(V));
end

% --------------------------------------------------------------------------
function lines = readlines_compat(fpath)
fid = fopen(fpath,'r');
if fid < 0, error('readlines_compat:open','cannot open %s', fpath); end
c = onCleanup(@() fclose(fid));
raw = fread(fid,'*char')';
lines = regexp(raw, '\r\n|\n|\r', 'split');
lines = lines(:);
end
