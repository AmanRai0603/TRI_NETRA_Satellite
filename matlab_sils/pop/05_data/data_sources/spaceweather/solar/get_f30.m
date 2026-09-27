function TT = get_f30(startDate, endDate, opts)
%GET_F30  Native F30 + 81-day mean (UTC timetable) with a REAL-F30-first chain.
%
% Input  : start/end date, opts
% Process: tries LISIRD/CLS measured F30 (get_f30_cls) first -> falls back to the OMNI2
%          F10.7-derived approximation only if unreachable -> adds the 81-day centered mean
% Output : UTC timetable TT with F30, F30_bar and the source actually used
%
%
% Fallback order (F10.7-derived F30 is the LAST resort, ):
%   1. LISIRD processed CLS F30   (real, flare-cleaned, 1-AU)   <- PRIMARY
%   2. NASA OMNI2 F10.7 -> F30     (approximate, derived)        <- LAST RESORT
%
% Native F30 (~48-52 sfu at the Nov 2009 minimum) is what the DTM2020 RESEARCH
% model wants -- well below F10.7 (~73). Feed TT.F30 / TT.F30_bar straight in.
%
% USAGE
%   TT = get_f30('2009-11-01','2009-11-30');            % auto: real first
%   TT = get_f30(...,struct('source','lisird'));        % force real F30
%   TT = get_f30(...,struct('source','f107'));          % force F10.7 fallback
%   TT = get_f30(...,struct('localFile','f30.csv'));    % offline LISIRD CSV
%   TT = get_f30(...,struct('localF107Files',{{'omni2_2009.dat','omni2_2010.dat'}})); % offline OMNI2
%
% OUTPUT  TT - timetable: Time, F30 [sfu], F30_bar [sfu]
%   TT.Properties.UserData.source records which path was used.

if nargin < 3, opts = struct(); end
if ~isfield(opts,'source'), opts.source = 'auto'; end
src = lower(opts.source);

% ---- 1) LISIRD real F30 (primary) ------------------------------------
if any(strcmp(src,{'auto','lisird'}))
    try
        TT = get_f30_cls(startDate, endDate, opts);
        TT.Properties.UserData = struct('source','LISIRD processed F30 (REAL)');
        fprintf('[get_f30] SOURCE = REAL F30 (LISIRD).\n');
        return;
    catch ME
        if strcmp(src,'lisird'), rethrow(ME); end
        warning('[get_f30] LISIRD F30 unavailable (%s).\n  -> falling back to F10.7->F30 (approximate).', ...
                ME.message);
    end
end

% ---- 2) F10.7 -> F30 (last resort) -----------------------------------
if any(strcmp(src,{'auto','f107'}))
    o2 = struct();
    if isfield(opts,'meanWindowDays'), o2.meanWindowDays = opts.meanWindowDays; end
    if isfield(opts,'localF107Files'), o2.localFiles = opts.localF107Files; end
    f107TT = get_f107(startDate, endDate, o2);   % NASA OMNI2 F10.7
    F30    = f30_from_f107(f107TT.F107);
    F30bar = f30_from_f107(f107TT.F107_bar);
    TT = timetable(f107TT.Time, F30, F30bar, 'VariableNames', {'F30','F30_bar'});
    TT.Properties.DimensionNames{1} = 'Time';
    TT.Properties.VariableUnits = {'sfu','sfu'};
    TT.Properties.UserData = struct('source','F10.7->F30 (APPROXIMATE, last resort)');
    fprintf('[get_f30] SOURCE = F10.7->F30 (APPROXIMATE). Use real F30 for final validation.\n');
    return;
end

error('get_f30: no source produced data (source=%s).', opts.source);
end
