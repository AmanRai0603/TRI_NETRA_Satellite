function T = read_tudelft_density_file(filename)
%READ_TUDELFT_DENSITY_FILE  Parse a TU Delft thermosphere density/wind file.
%
% Input  : path to a single TU Delft version-01/02 data file
% Process: reads the '# Column N:' header to locate fields by NAME (version-proof) -> textscan of
%          the numeric block -> assembles named columns
% Output : table T with Time (UTC) plus altitude, lat, lon, LST, density and wind columns
%
%
% Reads the "# Column N:" header to locate fields BY NAME, so it works across
% versions (v01 has a wind column block; v02 differs) and across satellites.
% Columns 1-3 are always Date, Time, Time-system (strings); the rest numeric.
%
% OUTPUT  T - table with:
%   DateTime (UTC), Altitude_m, Longitude_deg, Latitude_deg, LST_h,
%   Density_kgm3, DensityErr_kgm3, and Flag1..Flag4 if present.

fid = fopen(filename,'r');
if fid == -1, error('read_tudelft_density_file:open','Cannot open %s', filename); end

% ---- 1) parse the header: map field name -> column index ----
col = struct('alt',[],'lon',[],'lat',[],'lst',[],'arglat',[],'dens',[],'densmean',[],'denserr',[], ...
             'f1',[],'f2',[],'f3',[],'f4',[]);
ncol = 0; nHeader = 0;
while true
    pos  = ftell(fid);
    line = fgetl(fid);
    if ~ischar(line), break; end
    if ~startsWith(strtrim(line),'#')
        fseek(fid,pos,'bof');           % rewind to first data line
        break;
    end
    nHeader = nHeader + 1;
    tok = regexp(line, '#\s*Column\s+(\d+)\s*:\s*(.*)', 'tokens', 'once');
    if isempty(tok), continue; end
    idx = str2double(tok{1});  desc = lower(tok{2});
    ncol = max(ncol, idx);
    if     contains(desc,'altitude'),                       col.alt     = idx;
    elseif contains(desc,'longitude'),                      col.lon     = idx;
    elseif contains(desc,'latitude') && ~contains(desc,'argument'), col.lat = idx;
    elseif contains(desc,'local solar time'),               col.lst     = idx;
    elseif contains(desc,'argument of latitude'),           col.arglat  = idx;
    elseif contains(desc,'density') && ~contains(desc,'flag') && isempty(col.densmean) && ...
           ( contains(desc,'running') || contains(desc,'orbit average') || ...
             contains(desc,'orbit-average') || (contains(desc,'average') && contains(desc,'orbit')) )
        col.densmean = idx;                                % running-orbit-mean (NOT its flag)
    elseif contains(desc,'density') && ~contains(desc,'flag') && ( contains(desc,'error') || ...
           contains(desc,'uncert') || contains(desc,'rss') || contains(desc,'sum square') || ...
           contains(desc,'sigma') || contains(desc,'std') ),  col.denserr = idx;
    elseif contains(desc,'density') && contains(desc,'kg')
        if isempty(col.dens), col.dens = idx; end          % first true density column wins
    elseif contains(desc,'flag 1'),                         col.f1      = idx;
    elseif contains(desc,'flag 2'),                         col.f2      = idx;
    elseif contains(desc,'flag 3'),                         col.f3      = idx;
    elseif contains(desc,'flag 4'),                         col.f4      = idx;
    end
end

% ---- determine the TRUE column count from the first data line ----
% The fid is positioned at the first data line. Counting tokens here (instead of
% guessing 18) is critical: textscan ignores newlines, so a wrong ncol silently
% mis-parses every column. Handles both v02 layouts (density-only and +wind).
posData = ftell(fid);
firstLine = fgetl(fid);
while ischar(firstLine) && isempty(strtrim(firstLine)), firstLine = fgetl(fid); end
ncolActual = numel(regexp(strtrim(firstLine), '\s+', 'split'));
fseek(fid, posData, 'bof');
if ncolActual >= 9, ncol = ncolActual; elseif ncol < 9, ncol = max(ncolActual, 9); end

% Fallback to the documented column layout for any field the header did not name
if isempty(col.alt),  col.alt = 4;  end
if isempty(col.lon),  col.lon = 5;  end
if isempty(col.lat),  col.lat = 6;  end
if isempty(col.lst),  col.lst = 7;  end
if isempty(col.dens), col.dens = 9; end

% ---- 2) read the data: 3 leading strings + (ncol-3) numerics ----
fmt = ['%s %s %s' repmat(' %f', 1, ncol-3)];
C = textscan(fid, fmt, 'MultipleDelimsAsOne', true, 'CollectOutput', false);
fclose(fid);

% ---- guard against ragged textscan output ----
% A truncated final line, a blank running-orbit-mean cell, or one unreadable token
% leaves the HIGH-index columns a row short, so building the table below throws
% "number of rows must match the height of the table" (e.g. on DensityMean). Trim
% every column to the common length (drops only the incomplete tail row) and warn.
lens = cellfun(@numel, C);
if any(lens ~= min(lens))
    Lmin = min(lens);
    warning('read_tudelft_density_file:ragged', ...
        ['textscan returned unequal column lengths (%d..%d) -> trimming all to %d rows.\n' ...
         '         Normal for a truncated last line. If MANY rows are missing here, a\n' ...
         '         mid-file bad line stopped the parse early - inspect the file near row %d.'], ...
        min(lens), max(lens), Lmin, Lmin);
    C = cellfun(@(c) c(1:Lmin), C, 'UniformOutput', false);
end

% ---- find the DENSITY-ERROR column if the header did not name it ----
% This is what switches the measurement-uncertainty layer ON. The error is the
% only POSITIVE column at the SAME magnitude as density (a few % of it); wind
% (~1e2 m/s), flags (0/1) and angles are orders of magnitude off. Works whether
% the error sits at c13 (combined +wind) or c10 (density-only).
if isempty(col.denserr) && ~isempty(col.dens) && numel(C) >= col.dens
    densCol = C{col.dens};  dmed = median(abs(densCol),'omitnan');
    assigned = [col.alt col.lon col.lat col.lst col.arglat col.dens col.f1 col.f2 col.f3 col.f4];
    bestj = []; bestd = inf;
    for j = 4:ncol
        if any(j==assigned) || j > numel(C), continue; end
        cj = C{j};  fin = isfinite(cj);
        if ~any(fin) || min(cj(fin)) < 0, continue; end       % errors are non-negative
        rj = median(abs(cj(fin)))/dmed;                        % scale relative to density
        if rj > 1e-3 && rj < 0.95                              % same order, smaller than density
            d = abs(log(rj) - log(0.06));                      % prefer ~few-% magnitude
            if d < bestd, bestd = d; bestj = j; end
        end
    end
    if ~isempty(bestj)
        col.denserr = bestj;
        fprintf('[TUD-parse] density-error column not named in header -> picked c%d by value (%.1f%% of rho).\n', ...
                bestj, 100*median(abs(C{bestj}),'omitnan')/dmed);
    end
end

% ---- running-orbit-mean density column (if header did not name it) ----
% It is the column right after density at the SAME magnitude (ratio ~1), used
% below to derive sub-orbital variability. Distinct from the error scan above.
if isempty(col.densmean) && ~isempty(col.dens) && numel(C) >= col.dens+1
    cj = C{col.dens+1};  fin = isfinite(cj);
    if any(fin) && min(cj(fin)) >= 0
        rj = median(abs(cj(fin)))/median(abs(C{col.dens}),'omitnan');
        if rj > 0.3 && rj < 3, col.densmean = col.dens+1; end   % same scale as density
    end
end

% ---- 3) assemble table ----
T = table;
T.DateTime = datetime(strcat(C{1}, " ", C{2}), ...
                      'InputFormat','yyyy-MM-dd HH:mm:ss.SSS', 'TimeZone','UTC');
getcol = @(i) C{i};               % numeric columns are at their absolute index
T.Altitude_m     = getcol(col.alt);
T.Longitude_deg  = getcol(col.lon);
T.Latitude_deg   = getcol(col.lat);
T.LST_h          = getcol(col.lst);
if ~isempty(col.arglat) && col.arglat <= numel(C), T.ArgLat_deg = getcol(col.arglat); end
T.Density_kgm3   = getcol(col.dens);
if ~isempty(col.densmean) && col.densmean <= numel(C), T.DensityMean_kgm3 = getcol(col.densmean); end
if ~isempty(col.denserr)  && col.denserr  <= numel(C), T.DensityErr_kgm3  = getcol(col.denserr);  end
if ~isempty(col.f1) && col.f1 <= numel(C), T.Flag1 = getcol(col.f1); end
if ~isempty(col.f2) && col.f2 <= numel(C), T.Flag2 = getcol(col.f2); end
if ~isempty(col.f3) && col.f3 <= numel(C), T.Flag3 = getcol(col.f3); end
if ~isempty(col.f4) && col.f4 <= numel(C), T.Flag4 = getcol(col.f4); end

medD = median(T.Density_kgm3,'omitnan');
hasErrCol  = ismember('DensityErr_kgm3', T.Properties.VariableNames);
hasMeanCol = ismember('DensityMean_kgm3', T.Properties.VariableNames);
fprintf('[TUD-parse] %s : %d rows, %d cols (alt=c%d dens=c%d densmean=c%d denserr=c%d), median rho=%.3e kg/m^3.\n', ...
        filename, height(T), ncol, col.alt, col.dens, max([col.densmean 0]), max([col.denserr 0]), medD);
if ~(medD > 1e-16 && medD < 1e-6)
    warning('read_tudelft_density_file:density', ...
        ['Median density %.3e is NON-PHYSICAL -> wrong column picked (used c%d).\n' ...
         'Open the file header; confirm which column is the real "Density (kg/m3)".'], medD, col.dens);
end
if hasErrCol
    er = median(T.DensityErr_kgm3,'omitnan');
    fprintf('[TUD-parse] measurement 1-sigma present: median %.2e kg/m^3 (~%.1f%% of rho) -> measurement-uncertainty path.\n', ...
            er, 100*er/medD);
elseif hasMeanCol
    fprintf(['[TUD-parse] no per-sample error column in this product (expected for TU Delft ACC files),\n' ...
             '            but running-orbit-mean (c%d) IS present -> sub-orbital variability path available.\n'], col.densmean);
else
    warning('read_tudelft_density_file:nounc', ...
        ['Neither a density-error nor a running-mean column was found -> NO uncertainty layer.\n' ...
         '         Open the "# Column N:" header and tell me what columns this file has.']);
end
end
