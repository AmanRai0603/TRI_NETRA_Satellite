function DRV = drivers(model, startDate, endDate, opts)
%DATA.DRIVERS  The driver data ONE atmosphere model needs, for a date window.
%   DRV = data.drivers(model, startDate, endDate[, opts])
%
%   ---------------------------------------------------------------------------
%   WHY THIS EXISTS
%   ---------------------------------------------------------------------------
%   Each density model is driven by a DIFFERENT set of indices. Treating them as
%   interchangeable -- one space-weather struct handed to all of them -- is what
%   left JB2008 unrunnable, NRLMSISE without a storm history, and the DTM2020
%   research model wired to nothing at all.
%
%   The segmentation below is NOT invented here. It mirrors the proven one in
%   GOCE_density_study (2_spaceweather/ + 4_comparison/compute/align_drivers_to_track.m,
%   driven by 4_comparison/run/run_comparison_study.m), which is where these
%   fetchers come from and where they were validated:
%
%     model               solar driver              geomagnetic driver
%     ------------------  ------------------------  ------------------------
%     exponential         (none -- altitude only)   (none)
%     nrlmsise            F10.7, F10.7a             ap  (+ 57 h aph history)
%     dtm2020             F10.7, F10.7_bar          Kp, ap
%     dtm2020_research    F30, F30_bar  -> RESCALED ap60          <- GFZ
%                         to the F10.7 scale
%     jb2008              F10, S10, M10, Y10 (+81 d means) and DSTDTC   <- SET
%
%   ---------------------------------------------------------------------------
%   OUTPUT  DRV
%   ---------------------------------------------------------------------------
%     .model     the model these drivers are for (guard against cross-feeding)
%     .window    {startDate, endDate} actually requested
%     .swtable   data.spaceweather table      (nrlmsise, dtm2020)
%     .f30TT     F30 + F30_bar timetable      (dtm2020_research)
%     .ap60TT    ap60 timetable               (dtm2020_research)
%     .f30src    which F30 source was really used -- 'auto' can silently fall back
%                to an F10.7-DERIVED pseudo-F30, which is an approximation, not a
%                measurement. Always report it.
%     .jbidx     SET index struct             (jb2008)
%   Fields not needed by `model` are absent. Resolve ONCE per run, in
%   op.buildWorld -- never from inside the integrator's RHS.
%
%   opts: .manual  bypass the table (nrlmsise/dtm2020 only; JB2008 and the research
%                  model have no manual form -- their indices are whole files)
%         .source  'auto'|'historical'|'forecast'  (nrlmsise/dtm2020)
%         .f30source  'auto'|'lisird'|'f107'   .f30product 'standard'|'absolute'
%         .force   re-download rather than use cache

    if nargin < 4, opts = struct(); end
    if nargin < 3 || isempty(endDate), endDate = startDate; end
    model = lower(model);
    DRV = struct('model', model, 'window', {{startDate, endDate}});

    switch model
        case 'exponential'
            % altitude only. Deliberately no drivers -- this is the control case
            % that shows how much of your skill really comes from space weather.
            return

        case {'nrlmsise','dtm2020'}
            if isfield(opts,'manual') && ~isempty(opts.manual)
                return          % manual values bypass the table entirely
            end
            % 'auto' picks the NOAA/SWPC forecast for a future window and the
            % OMNI2+GFZ historical record otherwise.
            src = getf(opts,'source','auto');
            useForecast = strcmpi(src,'forecast') || ...
                          (strcmpi(src,'auto') && datenum(startDate) > datenum(clock()) + 3);
            if useForecast
                DRV.swtable = data.spaceweather_forecast(startDate, endDate, opts);
            else
                DRV.swtable = data.spaceweather(startDate, endDate, opts);
            end

        case 'dtm2020_research'
            % The F30/ap60 model. F30 is NOT F10.7 and not interchangeable with it.
            f30opts = struct('source',  getf(opts,'f30source','auto'), ...
                             'product', getf(opts,'f30product','standard'));
            DRV.f30TT  = get_f30(startDate, endDate, f30opts);
            DRV.ap60TT = get_gfz_hpo(startDate, endDate, {'ap60'});
            % get_f30's chain is: real CLS/LISIRD F30 first, then an F10.7-DERIVED
            % pseudo-F30 as a last resort. Those are different quantities and the
            % rescaling below must NOT be applied to the derived one. Carry which
            % one we actually got so nothing downstream has to guess.
            DRV.f30src = 'unknown';
            try
                DRV.f30src = DRV.f30TT.Properties.UserData.source;
            catch
                % LEGITIMATE swallow: the DEFAULT IS ALREADY SET ('unknown', line
                % above), so the failure cannot leave the field undefined -- which is
                % the whole difference between this and the bugs. UserData is
                % optional metadata; some fetch paths populate it, some do not, and
                % neither is an error. Compare op.buildWorld's eop_dir, which set
                % nothing before its try and so left the field MISSING on failure.
            end
            DRV.f30_is_derived = ~isempty(strfind(lower(DRV.f30src),'f10.7'));

        case 'jb2008'
            % SET's own files. No manual form: JB2008 is all-or-nothing on
            % SOLFSMY.TXT + DTCFILE.TXT.
            DRV.jbidx = data.jb2008_indices(startDate, endDate, opts);

        otherwise
            error('data:drivers:model', ...
              ['unknown atmosphere model "%s". Known: exponential, nrlmsise, dtm2020, ' ...
               'dtm2020_research, jb2008.'], model);
    end
end

function val = getf(s,f,d)
    if isfield(s,f) && ~isempty(s.(f)), val = s.(f); else, val = d; end
end
