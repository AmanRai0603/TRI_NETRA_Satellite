function TT = get_gfz_hpo(startDate, endDate, indices, opts)
%GET_GFZ_HPO  Download GFZ geomagnetic/solar indices as ONE UTC timetable.
%
% Input  : start/end date, cellstr of index names, opts (cache dir)
% Process: queries the GFZ JSON web service once per index -> parses value/time arrays ->
%          synchronises all indices on their UTC timestamps
% Output : single UTC timetable TT with one column per requested index
%
%
% Fetches one or more GFZ indices over a date range via the JSON web service
% (no FTP) and combines them into a single timetable, so ALL the
% variables together with their UTC timestamps.
%
% USAGE
%   TT = get_gfz_hpo('2024-01-01','2024-12-31');                 % default: Hp60 + ap60
%   TT = get_gfz_hpo('2024-01-01','2024-12-31',{'Hp60','ap60'}); % explicit
%   TT = get_gfz_hpo('2024-01-01','2024-12-31',{'Kp','ap','Fobs'}); % operational drivers
%   save('hpo_2024.mat','TT');   stackedplot(TT);
%
% INPUTS
%   startDate, endDate - 'YYYY-MM-DD' strings or datetimes (UTC)
%   indices - cell array of index names (default {'Hp60','ap60'}). Any of:
%             'Hp30','Hp60','ap30','ap60','Kp','ap','Ap','Cp','C9','SN',
%             'Fobs','Fadj'   (Fobs=observed F10.7, Fadj=1-AU-adjusted F10.7)
%   opts - (optional) struct:
%            .status 'all' (def+nowcast, default) or 'def' (definitive only)
%
% OUTPUT  TT - timetable:
%   Time (UTC, interval start) <- row index
%   one column per requested index (named after it)
%   Indices of different cadence (e.g. hourly ap60 + daily F10.7) are merged
%   onto a common timeline; gaps are NaN. Missing values are NaN.

if nargin < 3 || isempty(indices), indices = {'Hp60','ap60'}; end
if ischar(indices) || isstring(indices), indices = cellstr(indices); end
if nargin < 4, opts = struct(); end
if ~isfield(opts,'status'), opts.status = 'all'; end

s = char(datetime(startDate,'Format','yyyy-MM-dd'));
e = char(datetime(endDate,  'Format','yyyy-MM-dd'));

TT = [];   % will grow by synchronising each index in
for ii = 1:numel(indices)
    idx = indices{ii};
    url = sprintf(['https://kp.gfz.de/app/json/?start=%sT00:00:00Z' ...
                   '&end=%sT23:59:59Z&index=%s&status=%s'], s, e, idx, opts.status);
    fprintf('[GFZ] %s ...\n', url);
    try
        raw = webread(url, weboptions('Timeout',120,'ContentType','json'));
    catch ME
        warning('get_gfz_hpo:download','Fetch of %s failed: %s', idx, ME.message);
        continue;
    end

    fn = fieldnames(raw);

    % time field
    tf = fn{find(contains(lower(fn),'datetime'),1)};
    times = datetime(string(raw.(tf)), ...
        'InputFormat','yyyy-MM-dd''T''HH:mm:ss''Z''','TimeZone','UTC');

    % value field: exact name match, else first numeric field of right length
    vf = find(strcmpi(fn, idx), 1);
    if isempty(vf)
        for k = 1:numel(fn)
            if ~strcmpi(fn{k},tf) && isnumeric(raw.(fn{k})) && ...
               numel(raw.(fn{k}))==numel(times)
                vf = k; break;
            end
        end
    end
    vals = double(raw.(fn{vf})); vals = vals(:);
    vals(vals == -1) = NaN;                         % GFZ missing flag

    one = timetable(times(:), vals, 'VariableNames', {idx});

    if isempty(TT)
        TT = one;
    else
        % merge onto a union timeline (handles differing cadences)
        TT = synchronize(TT, one, 'union');
    end
end

if isempty(TT)
    error('get_gfz_hpo: no indices were retrieved.');
end

TT.Properties.DimensionNames{1} = 'Time';
TT.Properties.Description = sprintf('GFZ indices {%s}, %s to %s (UTC).', ...
                                    strjoin(indices,', '), s, e);
fprintf('[GFZ] Done: %d rows x %d indices.\n', height(TT), width(TT));
end
