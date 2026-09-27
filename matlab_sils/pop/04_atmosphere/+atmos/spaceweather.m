function sw = spaceweather(utc, opts)
%ATMOS.SPACEWEATHER  Space-weather indices for a UTC epoch (real data or manual).
%   sw = atmos.spaceweather(utc[, opts])
%     utc  : [Y Mo D H Mi S]
%     opts : .manual  struct('F107',..,'F107a',..,'Kp',..,'ap',..) -> use these
%            .source  'celestrak' (default)  .table (a pre-loaded data.spaceweather
%                     struct, to avoid re-reading the file every step)
%   RETURNS sw.F107 sw.F107a sw.Kp sw.ap (daily) sw.ap3 (3-hourly at the epoch)
%           sw.source ('manual' | 'celestrak')
%
%   NO HARDCODED DEFAULTS.  If you do not supply manual values and the cached
%   table does not cover this date (e.g. offline, or a future/predicted epoch not
%   in the file), this ERRORS and tells you to pass opts.manual.  This is the
%   "data available vs. give manual values" decision, made explicit.
%
%   PSEUDOCODE
%     if opts.manual: return it tagged 'manual'
%     T   <- opts.table or data.spaceweather()
%     mjd <- MJD(utc date)
%     row <- find T.mjd == mjd
%     if none: error "no space weather for <date>; pass opts.manual"
%     F107=row.f107obs ; F107a=row.f107c81 ; ap3=row.ap(hourbin) ; Kp=mean(row.kp)
    if nargin<2, opts=struct(); end

    if isfield(opts,'manual') && ~isempty(opts.manual)
        m = opts.manual;
        sw.F107  = req(m,'F107');
        sw.F107a = getf(m,'F107a', sw.F107);

        % ---------------------------------------------------------------------
        % NaN POISONING -- this used to be: sw.Kp = getf(m,'Kp', NaN).
        %
        % That put a NaN *field* on sw. Everything downstream reads its indices
        % with getf(sw,'Kp',3), and getf defaults on MISSING or EMPTY -- never on
        % NaN. So the field existed, getf returned the NaN instead of the default,
        % and the NaN flowed into rho -> acceleration -> the integrator. That is
        % the whole story behind "dtm2020 returns NaN": manual space weather given
        % as struct('F107',90,'F107a',90,'ap',8) -- ap but no Kp -- silently
        % poisoned DTM2020, which reads Kp. NRLMSISE reads ap, so it survived the
        % same input. The model that broke depended only on which index you omitted.
        %
        % ap and Kp measure the same thing on different scales, so if we have one
        % we can derive the other instead of inventing a NaN. If we have NEITHER,
        % that is a real decision the caller has to make, so it ERRORS -- which is
        % what this file's header has always promised and did not deliver on this
        % path.
        % ---------------------------------------------------------------------
        hasKp = isfield(m,'Kp') && ~isempty(m.Kp) && ~any(isnan(m.Kp));
        hasAp = isfield(m,'ap') && ~isempty(m.ap) && ~any(isnan(m.ap));
        if hasKp && hasAp
            sw.Kp = m.Kp;          sw.ap = m.ap;
        elseif hasKp
            sw.Kp = m.Kp;          sw.ap = kp2ap(m.Kp);
        elseif hasAp
            sw.ap = m.ap;          sw.Kp = ap2kp(m.ap);
        else
            error('atmos:spaceweather:manualGeomag', ...
              ['manual space weather must include a geomagnetic index: give ''ap'' ' ...
               'or ''Kp'' (either one; the other is derived). Without it DTM2020 ' ...
               'and NRLMSISE have no storm driver, and silently returning NaN is ' ...
               'worse than stopping here.']);
        end
        sw.ap3   = getf(m,'ap3', sw.ap);
        % NRLMSISE's aph(1..7) wants a 57-hour ap HISTORY. Manual input is a single
        % scalar, so there is no history to give: every slot gets the same value and
        % the model has no storm response. That is a real limitation of manual mode,
        % not a detail -- tagged so the caller can see it in sw.aph_source.
        sw.aph        = getf(m,'ap3', sw.ap) * ones(1,7);
        sw.aph(1)     = sw.ap;
        sw.aph_source = 'manual-flat (no storm history)';
        sw.F107_today = sw.F107;
        sw.source= 'manual';
        assertFinite(sw);
        return
    end

    if isfield(opts,'table') && ~isempty(opts.table)
        T = opts.table;
    else
        error('atmos:spaceweather:noTable', ...
          ['no space-weather table supplied and no manual values. Either pass ' ...
           'opts.table (from data.spaceweather(startDate,endDate)) or opts.manual ' ...
           '= struct(''F107'',..,''F107a'',..,''Kp'',..,''ap'',..).']);
    end

    mjd = floor(datenum(utc(1),utc(2),utc(3)) - 678942);
    k = find(T.mjd == mjd, 1);
    if isempty(k)
        dstr = sprintf('%04d-%02d-%02d', utc(1),utc(2),utc(3));
        error('atmos:spaceweather:noData', ...
          ['no space-weather record for %s in the cached table (range MJD ' ...
           '%.0f..%.0f). Fetch fresh data (online) or pass opts.manual = ' ...
           'struct(''F107'',..,''F107a'',..,''Kp'',..,''ap'',..).'], ...
           dstr, min(T.mjd), max(T.mjd));
    end
    hourbin = min(8, floor((utc(4)+utc(5)/60)/3)+1);

    % ------------------------------------------------------------------------
    % THE F10.7 LAG. This used to be sw.F107 = T.f107obs(k) -- the SAME day as
    % the epoch. Every model in this tree wants the PREVIOUS day:
    %   NRLMSISE-00 spec : "f107 - DAILY F10.7 FLUX FOR PREVIOUS DAY"
    %   dtm2020_oper_density header : "F107 - F10.7 flux at t-24h [sfu]"
    %   jb2008_density   : interp1(solT, F10, dnum-1)   <- already does it right
    % The reason is physical, not clerical: the thermosphere responds to EUV with
    % a lag, and the models were FITTED with the lagged index. Feeding same-day
    % F10.7 into a model calibrated on t-24h is a systematic error that grows with
    % dF10.7/dt -- i.e. it is smallest at solar minimum, when you would test it,
    % and largest during a rising-phase storm, when you care.
    %
    % >> THIS CHANGES RESULTS vs every previous run of this toolbox. <<
    % Set opts.lag_f107 = false to reproduce old numbers. sw.F107_today is always
    % carried so you can see both.
    % ------------------------------------------------------------------------
    % ------------------------------------------------------------------
    % WHICH CONVENTION? Default = the one the GOCE study VALIDATED with.
    %
    % GOCE_density_study/4_comparison/compute/align_drivers_to_track.m does:
    %     prev = @(tt,v) interp1(posixtime(tt), v, tp, 'previous', NaN);
    %     D.F107 = prev(f107TT.Time, f107TT.F107);
    % 'previous' = the most recent sample at or before the epoch. For a DAILY
    % F10.7 table stamped at 00:00 that is the SAME day -- which is exactly what
    % this function has always done via T.mjd == floor(datenum(utc)).
    %
    % The model specs disagree with that: NRLMSISE-00 says "f107 - DAILY F10.7
    % FLUX FOR PREVIOUS DAY" and dtm2020_oper_density's header says "F107 - F10.7
    % flux at t-24h". So spec and proven pipeline differ by one day.
    %
    % DEFAULT = false = the proven convention, because the density models in this
    % toolbox were validated against GOCE MEASURED density with that alignment.
    % Turning this on makes the propagator spec-correct and simultaneously makes it
    % DISAGREE with the study you trust. That is a decision for you, not for this
    % function to take silently. Either way sw.F107_lag records which was used.
    % ------------------------------------------------------------------
    lagF107 = getf(opts,'lag_f107', false);
    kprev = find(T.mjd == mjd-1, 1);
    sw.F107_today = T.f107obs(k);
    if lagF107 && ~isempty(kprev) && isfinite(T.f107obs(kprev))
        sw.F107     = T.f107obs(kprev);
        sw.F107_lag = 'previous day (t-24h, per model spec)';
    else
        sw.F107 = T.f107obs(k);
        sw.F107_lag = 'same day (''previous'' sample -- matches align_drivers_to_track)';
        if lagF107
            % First row of the table has no yesterday. Say so rather than silently
            % using today and letting it look identical to a lagged run.
            sw.F107_lag = 'SAME DAY (lag requested but no t-24h record -- extend padDays)';
        end
    end
    sw.F107a = T.f107c81(k);
    sw.Kp    = mean(T.kp(k,:));
    sw.ap    = T.apDaily(k);
    sw.ap3   = T.ap(k,hourbin);

    % ------------------------------------------------------------------------
    % THE ap HISTORY. NRLMSISE-00's aph is not one number, it is a 57-hour history:
    %   aph(1) daily Ap
    %   aph(2) ap now       aph(3) ap -3h    aph(4) ap -6h    aph(5) ap -9h
    %   aph(6) mean ap over -12..-33h        aph(7) mean ap over -36..-57h
    % atmos/nrlmsise.m used to build aph = Ap*ones(1,7) from the DAILY Ap, which
    % erases exactly the structure the array exists to carry. During a storm ap3
    % can be 200+ while the daily Ap is 50: the model was being told the storm did
    % not happen. Note this file already computed sw.ap3 -- and nothing consumed it.
    % ------------------------------------------------------------------------
    % ------------------------------------------------------------------
    % aph MODE. Same story as the lag above.
    %   'flat'    = D.ap .* ones(1,7)  -- what run_comparison_study.m uses, and
    %               what the validated GOCE numbers were produced with.
    %   'history' = the real 57 h ap history NRLMSISE-00's interface defines
    %               (aph(1) daily; aph(2..5) now/-3/-6/-9h; aph(6) mean -12..-33h;
    %               aph(7) mean -36..-57h). Requires flags(9) = -1 in the adapter,
    %               otherwise atmosnrlmsise00 ignores the array entirely.
    % DEFAULT 'flat' = the proven convention. 'history' is more faithful to the
    % model spec and gives it a real storm response, but it is NOT what the study
    % validated, so it is opt-in and tagged in sw.aph_source.
    % ------------------------------------------------------------------
    switch lower(getf(opts,'aph_mode','flat'))
        case 'history'
            sw.aph = buildAph(T, k, hourbin);
            sw.aph_source = 'history (real 57 h, spec form; needs flags(9)=-1)';
        otherwise
            sw.aph = sw.ap * ones(1,7);
            sw.aph_source = 'flat (matches run_comparison_study.m -- the validated convention)';
    end
    if isfield(T,'source') && ~isempty(T.source), sw.source=T.source; else, sw.source='table'; end
    assertFinite(sw, T, k, utc);
end

function assertFinite(sw, T, k, utc)
%ASSERTFINITE  A NaN index is never a usable answer -- stop here, not 200 km later.
%   A NaN in F107/ap/Kp propagates to rho, to the acceleration, and then into the
%   integrator, where it becomes either a silent NaN trajectory or a step-size
%   collapse thousands of evaluations away from the cause. Failing at the source
%   turns a two-hour hunt into a one-line message.
    f = {'F107','F107a','Kp','ap'};
    for i=1:numel(f)
        if ~isfield(sw,f{i}) || isempty(sw.(f{i})) || any(~isfinite(sw.(f{i})))
            % SAY WHAT IS WRONG. "F107 is NaN" sends you looking at the atmosphere;
            % the actual cause is almost always one of three things, and the table
            % itself can distinguish them in one line:
            %   - the epoch is outside the table  -> extend padDays
            %   - the row exists but the SOURCE had a gap (OMNI2 writes 999.9 as a
            %     fill value, which parses to NaN) -> that day genuinely has no F10.7
            %   - the table never loaded          -> a fetch failed upstream
            diag = '';
            try
                if isempty(T) || ~isfield(T,'mjd') || isempty(T.mjd)
                    diag = sprintf('\n  The table is EMPTY -- the upstream fetch failed.');
                else
                    diag = sprintf('\n  requested epoch  : %s (MJD %.4f)', ...
                                   datestr(datenum(utc),31), T.mjd(min(max(k,1),numel(T.mjd))));
                    diag = [diag sprintf('\n  table covers     : MJD %.1f .. %.1f (%d rows)', ...
                                   min(T.mjd), max(T.mjd), numel(T.mjd))];
                    if isempty(k)
                        diag = [diag sprintf('\n  -> the epoch is NOT IN the table. Extend the driver window (padDays).')];
                    else
                        nn = 0; tot = 0;
                        if isfield(T,'f107obs'), nn = sum(~isfinite(T.f107obs)); tot = numel(T.f107obs); end
                        diag = [diag sprintf('\n  row %d of %d found, so the epoch IS covered.', k, numel(T.mjd))];
                        diag = [diag sprintf('\n  f107obs has %d NaN of %d rows -- OMNI2 writes 999.9 for a', nn, tot)];
                        diag = [diag sprintf('\n  missing day and that parses to NaN. This day simply has no F10.7.')];
                        diag = [diag sprintf('\n  -> set opts.manual (e.g. struct(''F107'',%.0f,''F107a'',%.0f,''ap'',%.0f,''Kp'',%.0f))', ...
                                   nanOr(sw,'F107a',75), nanOr(sw,'F107a',75), nanOr(sw,'ap',5), nanOr(sw,'Kp',1))];
                        diag = [diag sprintf('\n     or pick a different DATE, or use a source without the gap.')];
                    end
                end
            catch
            end
            error('atmos:spaceweather:nonfinite', ...
                ['space-weather index "%s" is missing or non-finite (%g). This would ' ...
                 'have reached the density model and then the integrator as a NaN.%s'], f{i}, ...
                 subsref_tern(isfield(sw,f{i}) && ~isempty(sw.(f{i})), sw_first_(sw,f{i}), NaN), diag);
        end
    end
end

function out = tern(c,a,b)
    if c, out=a(); else, out=b(); end
end

function Kp = ap2kp(ap)
%AP2KP  Inverse of the standard Kp->ap table (nearest node). ap and Kp are the
%   same geomagnetic activity on two scales; deriving one from the other is far
%   better than handing the models a NaN.
    kpv=[0 .33 .67 1 1.33 1.67 2 2.33 2.67 3 3.33 3.67 4 4.33 4.67 5 5.33 5.67 6 6.33 6.67 7 7.33 7.67 8 8.33 8.67 9];
    apv=[0 2 3 4 5 6 7 9 12 15 18 22 27 32 39 48 56 67 80 94 111 132 154 179 207 236 300 400];
    if isnan(ap), Kp=NaN; return; end
    Kp = interp1(apv, kpv, max(0,min(400,ap)), 'nearest');
end

function aph = buildAph(T, k, hourbin)
%BUILDAPH  NRLMSISE-00's 7-element ap array from the 3-hourly table.
%   Slots per the NRLMSISE-00 / MSISE-90 interface definition. Walks BACKWARDS
%   through the flattened 3-hourly series, so it crosses day boundaries correctly
%   instead of clamping at midnight.
    flat = reshape(T.ap.', [], 1);          % (N*8) x 1, chronological, 3 h spacing
    now_ = (k-1)*8 + hourbin;               % index of the current 3 h bin
    at = @(back) pick(flat, now_ - back);   % back = how many 3 h bins ago

    aph = nan(1,7);
    aph(1) = T.apDaily(k);
    aph(2) = at(0);                         % now
    aph(3) = at(1);                         % -3 h
    aph(4) = at(2);                         % -6 h
    aph(5) = at(3);                         % -9 h
    aph(6) = meanBins(flat, now_, 4, 11);   % -12 .. -33 h  (eight bins)
    aph(7) = meanBins(flat, now_, 12, 19);  % -36 .. -57 h  (eight bins)
    % Before the table starts there is no history. Fall back to the daily Ap for
    % the missing slots -- degraded, but finite and flagged by sw.aph_source.
    aph(~isfinite(aph)) = T.apDaily(k);
end

function v = pick(flat, i)
    if i >= 1 && i <= numel(flat), v = flat(i); else, v = NaN; end
end

function m = meanBins(flat, now_, b0, b1)
    idx = now_ - (b1:-1:b0);
    idx = idx(idx >= 1 & idx <= numel(flat));
    v = flat(idx); v = v(isfinite(v));
    if isempty(v), m = NaN; else, m = mean(v); end
end

function ap = kp2ap(Kp)
% Standard Kp->ap conversion table midpoints (used only if manual gives Kp not ap)
    if isnan(Kp), ap=NaN; return; end
    kpv=[0 .33 .67 1 1.33 1.67 2 2.33 2.67 3 3.33 3.67 4 4.33 4.67 5 5.33 5.67 6 6.33 6.67 7 7.33 7.67 8 8.33 8.67 9];
    apv=[0 2 3 4 5 6 7 9 12 15 18 22 27 32 39 48 56 67 80 94 111 132 154 179 207 236 300 400];
    ap = interp1(kpv, apv, max(0,min(9,Kp)), 'nearest');
end
function v=req(s,f)
    if ~isfield(s,f)||isempty(s.(f)), error('atmos:spaceweather:manualMissing', ...
        'manual space weather must include field "%s"', f); end
    v=s.(f);
end
function val=getf(s,f,d), if isfield(s,f)&&~isempty(s.(f)), val=s.(f); else, val=d; end, end

function v = nanOr(sw, fn, dflt)
%NANOR  A finite value from sw, else a plausible quiet-Sun default for the message.
    v = dflt;
    if isfield(sw,fn) && ~isempty(sw.(fn)) && isfinite(sw.(fn)(1)), v = sw.(fn)(1); end
end

function v = sw_first_(sw, fn)
%SW_FIRST_  First element, for the error message. NaN if absent.
    v = NaN;
    if isfield(sw,fn) && ~isempty(sw.(fn)), v = sw.(fn)(1); end
end
