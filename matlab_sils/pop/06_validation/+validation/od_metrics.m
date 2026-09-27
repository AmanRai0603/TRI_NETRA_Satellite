function R = od_metrics(sol, W, REF, opts)
%VALIDATION.OD_METRICS  The four OD comparisons, computed once, in one place.
%
%   R = validation.od_metrics(sol, W, REF, opts)
%
%   ---------------------------------------------------------------------------
%   WHY THIS IS A FUNCTION AND NOT INLINE IN validate_OD
%   ---------------------------------------------------------------------------
%   validate_OD answers "is my force model right for THIS setup?". compare_OD
%   answers "which setup is least wrong?" -- which is the same four metrics
%   evaluated N times. If each script carried its own copy of the metric code they
%   would drift, and the day they disagreed you would not know which one to trust.
%   So the metrics live here and both scripts call this. Change a metric once.
%
%   ---------------------------------------------------------------------------
%   INPUT
%   ---------------------------------------------------------------------------
%     sol   propagation result: .t [N x 1] s from epoch, .r .v [N x 3] CELESTIAL
%     W     world from op.buildWorld (needs .swmanual/.swtable for the atmosphere)
%     REF   the ITSG products, already fetched. Fields (each optional):
%             .RDO  reducedDynamicOrbit  -> .mjd .r .v
%             .KIN  kinematicOrbit       -> .mjd .r
%             .ACC  nonConservativeForces-> .mjd .a_sat   (SATELLITE frame)
%             .ATT  attitude             -> .mjd .q       (sat -> celestial)
%             .DEN  neutralDensity[_ACC] -> .mjd .rho [+ .lat .lon .alt .lst]
%             .TUD  TU Delft measured density -> .mjd .rho .lat .lon .alt .lst
%                   (INDEPENDENT of ITSG: different processing chain, different
%                    group. Where both exist, agreement between them is a check on
%                    the TRUTH, not on us.)
%           Fetch these ONCE and pass them in: ITSG products are immutable, so a
%           sweep must not re-read them per value.
%     opts  .mjd0     seed epoch (MJD) -- the t=0 of sol            [required]
%           .tspan_s  arc length [s]                                [required]
%           .epoch    UTC [Y Mo D H Mi S] of mjd0                   [required]
%           .frame    'gmst'|'A'|'B'|'C'  (only the r1.0 density branch uses it)
%           .atmos    atmosphere model name for the density comparison
%           .verbose  true -> print each line as it is computed (default false)
%
%   ---------------------------------------------------------------------------
%   OUTPUT  R
%   ---------------------------------------------------------------------------
%     .rdo  RTN stats vs reducedDynamicOrbit + .dv + .t
%     .kin  RTN stats vs kinematicOrbit      + .t   (the REAL, irregular epochs)
%     .refSpread  kinematic - reducedDynamic [m RMS] -- the reference's OWN
%                 uncertainty. A residual below this is not resolvable, and
%                 reporting one is how you end up modelling noise.
%     .acc  .t .meas .mod .rms_meas .rms_mod .rms_diff   [m/s^2]
%     .den  .t .meas .mod .ratio .rms_logerr .product .geoFromProduct
%     .tud  same shape, vs TU Delft measured density (an independent second truth)
%   Missing inputs are skipped silently; missing OUTPUT fields mean "not computed".
%
%   ---------------------------------------------------------------------------
%   THINGS THAT ARE DELIBERATE HERE
%   ---------------------------------------------------------------------------
%   * Sampling uses validation.interp_state (Hermite on r,v), never pchip. pchip
%     off a 30 s grid injected 72.6 m RMS of pure interpolant into .kin -- 2000x
%     the reference floor. See that file's header.
%   * Attitude is sign-unwrapped before interpolation (validation.quat_continuous).
%   * The atmosphere is reached exactly the way forces.drag reaches it, so the
%     model in .den sees the SAME indices the propagation saw.

    if ~isfield(opts,'verbose'), opts.verbose = false; end
    if ~isfield(opts,'frame'),   opts.frame   = 'gmst'; end
    if ~isfield(opts,'atmos'),   opts.atmos   = 'exponential'; end
    V = @(varargin) vprint(opts.verbose, varargin{:});
    R = struct();
    mjd0 = opts.mjd0; TSPAN_S = opts.tspan_s;

    % ---- (1) vs reduced dynamic -------------------------------------------------
    t_ref = []; r_ref = []; v_ref = [];
    if isfield(REF,'RDO') && ~isempty(REF.RDO)
        t_ref = (REF.RDO.mjd - mjd0)*86400;
        sel   = t_ref >= 0 & t_ref <= TSPAN_S;
        t_ref = t_ref(sel); r_ref = REF.RDO.r(sel,:); v_ref = REF.RDO.v(sel,:);
        [rr, vv] = validation.interp_state(t_ref, r_ref, v_ref, sol.t);
        R.rdo   = validation.rtn_stats(sol.r - rr, sol.r, sol.v);
        R.rdo.t = sol.t;
        % The reference CURVES, not only the residual. An overlay needs the actual
        % series; reconstructing them in the plotting code is how two versions of
        % "measured" come to exist and quietly disagree.
        R.rdo.r_ref = rr;      % reduced-dynamic position, on OUR epochs [m]
        R.rdo.v_ref = vv;      % reduced-dynamic velocity [m/s]
        R.rdo.r_our = sol.r;
        R.rdo.v_our = sol.v;
        % The SERIES, not only its RMS. show_OD's velocity panel could only draw a
        % horizontal line at a scalar, which is why it read as an empty plot with a
        % dashed line through zero: there was nothing else to draw. A residual's
        % SHAPE is the diagnostic -- a bias shows as an offset, a drag error grows
        % secularly, a frame error oscillates once per rev. The RMS collapses all
        % three into one number that cannot distinguish them.
        dv_vec   = sol.v - vv;                 % [N x 3] ECI velocity residual
        R.rdo.dv_vec = dv_vec;
        R.rdo.dv_mag = sqrt(sum(dv_vec.^2, 2));
        % RTN decomposition of the VELOCITY residual, using the same triad as position
        R.rdo.dv_rtn = validation.rtn_project(dv_vec, sol.r, sol.v);
        R.rdo.dv = sqrt(mean(sum(dv_vec.^2, 2)));
        V('[1] vs reducedDynamicOrbit : radial %8.2f  along %8.2f  cross %8.2f  |3D| %8.2f m  |dv| %.4f m/s\n', ...
          R.rdo.radial, R.rdo.along, R.rdo.cross, R.rdo.pos3D, R.rdo.dv);
    end

    % ---- (2) vs kinematic (INDEPENDENT) -----------------------------------------
    if isfield(REF,'KIN') && ~isempty(REF.KIN)
        tk = (REF.KIN.mjd - mjd0)*86400;
        sk = tk >= 0 & tk <= TSPAN_S;
        if sum(sk) > 10
            rk = REF.KIN.r(sk,:); tk = tk(sk);
            % kinematic epochs are NOT equidistant -> sample OUR solution onto THEM
            [rp, vp] = validation.interp_state(sol.t, sol.r, sol.v, tk);
            R.kin   = validation.rtn_stats(rp - rk, rp, vp);
            R.kin.t = tk;
            R.kin.r_ref = rk;      % kinematic position, on ITS OWN epochs [m]
            R.kin.r_our = rp;      % our solution sampled onto those epochs
            R.kin.v_our = vp;
            % NO v_ref: the kinematic product is MJD,x,y,z -- POSITION ONLY. There
            % is no kinematic velocity to compare against, and a velocity-vs-
            % kinematic panel therefore cannot exist. The figures say so rather than
            % omitting it silently, because a missing panel looks like an oversight
            % and an absent product is a fact about the data.
            V('[2] vs kinematicOrbit      : radial %8.2f  along %8.2f  cross %8.2f  |3D| %8.2f m\n', ...
              R.kin.radial, R.kin.along, R.kin.cross, R.kin.pos3D);
            if ~isempty(t_ref)
                rr2 = validation.interp_state(t_ref, r_ref, v_ref, tk);
                R.refSpread = sqrt(mean(sum((rr2 - rk).^2, 2)));
                V('    reference spread (kinematic - reducedDynamic) = %.2f m RMS\n', R.refSpread);
            end
        end
    end

    % ---- (3) non-conservative force: measured vs modelled ------------------------
    if isfield(REF,'ACC') && ~isempty(REF.ACC) && isfield(REF,'ATT') && ~isempty(REF.ATT)
        ta = (REF.ACC.mjd - mjd0)*86400;
        sa = ta >= 0 & ta <= TSPAN_S;
        ta = ta(sa); a_sat = REF.ACC.a_sat(sa,:);
        [qa, nflip] = validation.quat_continuous(REF.ATT.q);
        if nflip > 0
            V('    attitude: %d/%d sign flips unwrapped before interpolation\n', nflip, size(qa,1));
        end
        q = interp1(REF.ATT.mjd, qa, mjd0 + ta/86400, 'spline');
        q = q ./ sqrt(sum(q.^2,2));
        % ---- THE FRAME, STATED ONCE, WHERE THE ROTATION HAPPENS ---------------
        % ITSG nonConservativeForces is measured in the SATELLITE BODY frame. The
        % attitude quaternion is satellite -> celestial, so this rotates the
        % measurement INTO ECI, which is where op.accel works. Both sides of every
        % comparison downstream are therefore ECI -- consistent.
        %
        % What that costs: the PER-AXIS split now inherits the attitude product's
        % error. A bad attitude moves signal BETWEEN the x/y/z panels without
        % changing the total, so |a| is robust to attitude and the per-axis
        % breakdown is not. Worth knowing before reading a per-axis delta as physics.
        a_meas = zeros(numel(ta),3);
        for k = 1:numel(ta)
            a_meas(k,:) = (validation.quat2dcm(q(k,:)) * a_sat(k,:).').';
        end
        % op.accel returns [a, parts]; the accelerometer senses drag+SRP+ERP, so the
        % modelled counterpart is just that sum out of parts -- no second run needed.
        NONCONS = {'drag','srp','erp'};
        [rp, vp] = validation.interp_state(sol.t, sol.r, sol.v, ta);
        a_mod = zeros(numel(ta),3);
        % Keep the components SEPARATELY. "the non-grav sum is 10% low" is not an
        % actionable statement -- drag, SRP and ERP are three different physics with
        % three different fixes, and at 345 km they differ by two orders of
        % magnitude. Summing them before plotting throws away the only information
        % that says WHICH one to look at.
        a_cmp = struct('drag',zeros(numel(ta),3),'srp',zeros(numel(ta),3),'erp',zeros(numel(ta),3));
        for k = 1:numel(ta)
            [~, parts] = op.accel(ta(k), rp(k,:).', vp(k,:).', W);
            ak = zeros(3,1);
            for j = 1:numel(NONCONS)
                if isfield(parts, NONCONS{j})
                    ak = ak + parts.(NONCONS{j})(:);
                    a_cmp.(NONCONS{j})(k,:) = parts.(NONCONS{j})(:).';
                end
            end
            a_mod(k,:) = ak.';
        end
        % ---- THE ACCELEROMETER CARRIES A BIAS, AND RMS-ABOUT-ZERO EATS IT ------
        % An RMS taken about zero includes a constant offset at FULL STRENGTH. CHAMP's
        % STAR accelerometer has a known along-track bias of order 1e-6 m/s^2 -- which
        % is roughly TEN TIMES the drag it is trying to measure at 345 km. That is the
        % whole reason TU Delft publish a CALIBRATED density product: they estimate
        % bias and scale before inverting the accelerometer.
        %
        % So the old rms_meas was ~|bias|, the ratio rms_mod/rms_meas was
        % drag/bias ~ 0.1, and it was reported as "the model is 10x too small".
        % It is not a physics result at all; it is the signal-to-offset ratio.
        %
        % The tell was there the whole time and we walked past it: the ratio DID NOT
        % MOVE when the drag model changed. Swapping cannonball->dria shifts the
        % numerator ~20%, and 0.097 -> 0.117 is invisible against a denominator that
        % is 10x bigger and constant. A physics error moves when you change the
        % physics. This one never did.
        %
        % And the along-track arithmetic settles it independently: a genuine 1.79e-6
        % m/s^2 along-track deficit integrates to 0.5*a*t^2 = 104 m over a 3 h arc.
        % The run reported |3D| = 2.20 m against the reduced-dynamic orbit. A real
        % 10x drag deficit cannot hide inside a 2.2 m residual.
        %
        % FIX: compare the VARYING part. Remove the per-axis mean from BOTH series
        % (the model has no bias, so its mean removal is a no-op on the physics but
        % keeps the comparison symmetric), and report the bias separately -- it is a
        % real, interesting, instrument-level number, not something to hide.
        bias_meas = mean(a_meas, 1);
        bias_mod  = mean(a_mod, 1);
        ac_meas   = a_meas - bias_meas;
        ac_mod    = a_mod  - bias_mod;

        % A moving average over one orbit separates the slowly-varying part (which a
        % bias drift lives in) from the per-revolution drag signature we actually
        % model. Window = the output spacing that spans ~1 orbit, clipped to the arc.
        dt_   = median(diff(ta));
        nOrb_ = max(3, min(round(5400/max(dt_,1)), floor(numel(ta)/3)));
        R.acc = struct('t',ta, 'meas',a_meas, 'mod',a_mod, ...
                       'rms_meas',sqrt(mean(sum(a_meas.^2,2))), ...
                       'rms_mod', sqrt(mean(sum(a_mod.^2,2))), ...
                       'rms_diff',sqrt(mean(sum((a_mod-a_meas).^2,2))), ...
                       'bias_meas',bias_meas, 'bias_mod',bias_mod, ...
                       'ac_meas',ac_meas, 'ac_mod',ac_mod, ...
                       'rms_ac_meas',sqrt(mean(sum(ac_meas.^2,2))), ...
                       'rms_ac_mod', sqrt(mean(sum(ac_mod.^2,2))), ...
                       'rms_ac_diff',sqrt(mean(sum((ac_mod-ac_meas).^2,2))), ...
                       'movavg_win', nOrb_, 'cmp', a_cmp);
        R.acc.ratio_raw = R.acc.rms_mod    / max(R.acc.rms_meas,eps);
        R.acc.ratio     = R.acc.rms_ac_mod / max(R.acc.rms_ac_meas,eps);   % THE physical one

        V('[3] non-conservative force (RMS about zero -- INCLUDES the instrument bias):\n');
        V('      measured %.3e | modelled %.3e | ratio %.3f\n', ...
          R.acc.rms_meas, R.acc.rms_mod, R.acc.ratio_raw);
        V('    accelerometer bias (mean of the measured series, satellite->ECI):\n');
        V('      [%.3e %.3e %.3e] m/s^2, |bias| = %.3e\n', bias_meas, norm(bias_meas));
        % Report the RATIO we measured, not a remembered fact about CHAMP. The
        % previous text said "CHAMP STAR carries ~1e-6" on every satellite's run,
        % including GRACE-A. Accelerometer biases are per-instrument and per-epoch;
        % the run in front of us knows its own.
        V('      |bias| / |signal| = %.1f  -- an RMS about zero is therefore mostly the\n', ...
          norm(bias_meas)/max(R.acc.rms_ac_mod,eps));
        V('      OFFSET, not the signal. (Accelerometers on these missions carry biases\n');
        V('      of order the drag itself or larger; that is why TU Delft publish a\n');
        V('      CALIBRATED density rather than inverting the raw ACC.)\n');
        V('[3] BIAS-REMOVED (this is the physics comparison):\n');
        V('      measured %.3e | modelled %.3e | diff %.3e m/s^2\n', ...
          R.acc.rms_ac_meas, R.acc.rms_ac_mod, R.acc.rms_ac_diff);
        V('      ratio modelled/measured = %.3f  (1.0 = perfect Cd*A/m and density)\n', R.acc.ratio);
    end

    % ---- (4) density -------------------------------------------------------------
    if isfield(REF,'DEN') && ~isempty(REF.DEN)
        td = (REF.DEN.mjd - mjd0)*86400;
        sd = td >= 0 & td <= TSPAN_S;
        td = td(sd); rho_meas = REF.DEN.rho(sd);
        if ~isempty(td)
            rho_mod = nan(numel(td),1);
            haveGeo = isfield(REF.DEN,'lat') && ~isempty(REF.DEN.lat);
            if haveGeo
                lat = REF.DEN.lat(sd); lon = REF.DEN.lon(sd);
                alt = REF.DEN.alt(sd); lst = REF.DEN.lst(sd);
            end
            % Same recipe forces.drag uses. W.sw does NOT exist -- buildWorld makes
            % swmanual/swtable and drag turns them into sw per step. Getting this
            % wrong is why the density block silently reported NaN for so long.
            swopts = struct();
            if isstruct(W) && isfield(W,'swmanual') && ~isempty(W.swmanual), swopts.manual = W.swmanual; end
            if isstruct(W) && isfield(W,'swtable')  && ~isempty(W.swtable),  swopts.table  = W.swtable;  end
            usesSW = ~strcmpi(opts.atmos,'exponential');
            nfail = 0; firstErr = '';
            for k = 1:numel(td)
                uk = validation.mjd2utc(mjd0 + td(k)/86400);
                if haveGeo
                    geo = struct('alt_km',alt(k)/1000,'lat_deg',lat(k),'lon_deg',lon(k), ...
                                 'lst_h',lst(k),'doy',timeconv.doy(uk(1),uk(2),uk(3)),'utc',uk);
                else
                    % r1.0 ships MJD+rho only -> the sample point comes from OUR state,
                    % so our own position error leaks into the density comparison.
                    rp  = validation.interp_state(sol.t, sol.r, sol.v, td(k)).';
                    T   = timeconv.convertUTC(uk(1),uk(2),uk(3),uk(4),uk(5),uk(6),0);
                    Rot = frames.eci2ecef(uk, opts.frame, struct('gmst_rad',T.gmst_rad));
                    re  = Rot*rp;
                    % op.geodetic returns RADIANS (its own header says so).
                    % de440.constants() already ships Re_earth / f_earth. Retyping
                    % WGS84 here means two sources of truth that can silently drift.
                    Kc = de440.constants();
                    [la, lo, al] = op.geodetic(re, Kc.Re_earth, Kc.f_earth);
                    la = rad2deg(la); lo = rad2deg(lo);
                    geo = struct('alt_km',al/1000,'lat_deg',la,'lon_deg',lo, ...
                                 'lst_h',mod(uk(4)+uk(5)/60+uk(6)/3600 + lo/15, 24), ...
                                 'doy',timeconv.doy(uk(1),uk(2),uk(3)),'utc',uk);
                end
                try
                    if usesSW, sw = atmos.spaceweather(uk, swopts); else, sw = struct(); end
                    atm = atmos.provider(opts.atmos, geo, sw);
                    rho_mod(k) = atm.rho;
                catch MEk
                    nfail = nfail + 1;
                    if isempty(firstErr), firstErr = regexprep(MEk.message,'\n.*',''); end
                end
            end
            if nfail > 0
                V('    [!] atmosphere failed on %d/%d epochs -- first: %s\n', nfail, numel(td), firstErr);
            end
            good = isfinite(rho_mod) & isfinite(rho_meas) & rho_meas > 0;
            if any(good)
                td = td(good); rho_meas = rho_meas(good); rho_mod = rho_mod(good);
                R.den = struct('t',td, 'meas',rho_meas, 'mod',rho_mod, ...
                               'ratio',median(rho_mod./max(rho_meas,eps)), ...
                               'rms_logerr',sqrt(mean((log(rho_mod)-log(rho_meas)).^2)), ...
                               'model',opts.atmos, 'geoFromProduct',haveGeo);
                V('[4] density (%s): measured %.3e | modelled %.3e kg/m^3 | ratio %.3f | RMS log-err %.3f\n', ...
                  opts.atmos, median(rho_meas), median(rho_mod), R.den.ratio, R.den.rms_logerr);
            end
        end
    end

    % ---- INDEPENDENT TRACKS (TU Delft / TLE): overlay + error ---------------------
    % Not metrics -- diagnostics. Both are independent of the ITSG chain, at very
    % different accuracies, and both answer only "are we on the same orbit".
    for src = {'TUDTRK','TLETRK'}
        if ~isfield(REF, src{1}) || isempty(REF.(src{1})), continue, end
        Tk = REF.(src{1});
        tt = (Tk.mjd - mjd0)*86400;
        st = tt >= 0 & tt <= TSPAN_S;
        if sum(st) < 2, continue, end
        tt = tt(st);
        rq = validation.interp_state(sol.t, sol.r, sol.v, tt);
        d  = rq - Tk.r(st,:);
        out = struct('t',tt, 'r_ours',vecnorm(rq,2,2), 'r_track',vecnorm(Tk.r(st,:),2,2), ...
                     'err',vecnorm(d,2,2));
        if strcmp(src{1},'TUDTRK'), R.tud_track = out; else, R.tle_track = out; end
        V('[t] %s track: %d epochs in arc, |error| RMS %.1f m%s\n', src{1}, numel(tt), ...
          sqrt(mean(out.err.^2)), subsref_tern(strcmp(src{1},'TLETRK'), ...
          '   (a TLE is km-class BY DESIGN -- this is not a propagator error)', ''));
    end

    % ---- (5) density vs TU DELFT measured -----------------------------------------
    % A SECOND, INDEPENDENT truth. ITSG's neutralDensity and TU Delft's density are
    % derived by different groups from different processing of the same
    % accelerometer, with different assumptions about Cd and the gas-surface
    % interaction. So:
    %   * agreement between OUR model and BOTH  -> the model is probably right
    %   * we match one and not the other        -> look at the truth, not the model
    %   * the two truths disagree with each other -> that gap is the REAL floor on
    %     any density claim, exactly like refSpread is for position. Neither is
    %     "measured density" in an absolute sense; both are retrievals.
    if isfield(REF,'TUD') && ~isempty(REF.TUD)
        tt = (REF.TUD.mjd - mjd0)*86400;
        st = tt >= 0 & tt <= TSPAN_S;
        tt = tt(st);
        if ~isempty(tt)
            rho_meas = REF.TUD.rho(st);
            % TU Delft ships the geodetic track with the density, so the model is
            % evaluated at ITS points -- no leak from our own position error.
            lat = REF.TUD.lat(st); lon = REF.TUD.lon(st);
            alt = REF.TUD.alt(st); lst = REF.TUD.lst(st);
            swopts = struct();
            if isstruct(W) && isfield(W,'swmanual') && ~isempty(W.swmanual), swopts.manual = W.swmanual; end
            if isstruct(W) && isfield(W,'swtable')  && ~isempty(W.swtable),  swopts.table  = W.swtable;  end
            usesSW = ~strcmpi(opts.atmos,'exponential');
            rho_mod = nan(numel(tt),1); nfail = 0; firstErr = '';
            for k = 1:numel(tt)
                uk = validation.mjd2utc(mjd0 + tt(k)/86400);
                geo = struct('alt_km',alt(k)/1000,'lat_deg',lat(k),'lon_deg',lon(k), ...
                             'lst_h',lst(k),'doy',timeconv.doy(uk(1),uk(2),uk(3)),'utc',uk);
                try
                    switch lower(opts.atmos)
                        case 'exponential', sw = struct();
                        case 'jb2008',      sw = struct('jb_idx', getfd(W,'jbidx',[]));
                        case {'dtm2020_research','dtm2020_res'}
                            sw = atmos.research_drivers(getfd(W,'drv',[]), uk);
                        otherwise
                            if usesSW, sw = atmos.spaceweather(uk, swopts); else, sw = struct(); end
                    end
                    atm = atmos.provider(opts.atmos, geo, sw);
                    rho_mod(k) = atm.rho;
                catch MEk
                    nfail = nfail + 1;
                    if isempty(firstErr), firstErr = regexprep(MEk.message,'\n.*',''); end
                end
            end
            if nfail > 0
                V('    [!] atmosphere failed on %d/%d TU Delft epochs -- first: %s\n', nfail, numel(tt), firstErr);
            end
            good = isfinite(rho_mod) & isfinite(rho_meas) & rho_meas > 0;
            if any(good)
                R.tud = struct('t',tt(good), 'meas',rho_meas(good), 'mod',rho_mod(good), ...
                               'ratio',median(rho_mod(good)./max(rho_meas(good),eps)), ...
                               'rms_logerr',sqrt(mean((log(rho_mod(good))-log(rho_meas(good))).^2)), ...
                               'model',opts.atmos);
                V('[5] density vs TU Delft (%s): measured %.3e | modelled %.3e | ratio %.3f | RMS log-err %.3f\n', ...
                  opts.atmos, median(rho_meas(good)), median(rho_mod(good)), R.tud.ratio, R.tud.rms_logerr);
                % the truth-vs-truth gap: the real floor under any density claim
                if isfield(R,'den')
                    R.densitySpread = abs(log(median(R.den.meas)) - log(median(R.tud.meas)));
                    V('    ITSG vs TU Delft (truth vs truth) = %.1f%% in median density.\n', ...
                      100*(exp(R.densitySpread)-1));
                    V('    -> a model closer to either than THIS is not distinguishable.\n');
                end
            end
        end
    end

    % ---- CLOSURE: the metrics must agree with EACH OTHER ---------------------
    % A missing along-track acceleration MUST produce a position drift, and the
    % relationship is arithmetic (dx = 0.5*da*t^2). Metrics [1] and [3] are not
    % independent facts, and comparing them is what broke the CHAMP mystery: [3]
    % claimed a deficit demanding 104 m of drift while [1] measured 2.2 m. They
    % contradicted each other by 48x for twelve rounds because nothing compared them.
    %
    % (This block first landed INSIDE vprint, because I anchored the insert on the
    % file's LAST `end` instead of on the main function's. It would never have run,
    % and it referenced a `VERBOSE` I invented -- the toggle is opts.verbose. Both
    % caught by 08_test/check_undefined.py before shipping, which is the argument for
    % having it.)
    % Split metric [3] by eclipse BEFORE closure, so the verdict is available to it.
    % SRP is zero in shadow and drag is not: that is the only lever in this toolbox
    % that separates two non-gravitational forces by DATA rather than assumption.
    try
        R.acc_split = validation.acc_split(R, opts.verbose);
    catch err_
        fprintf('  [eclipse split failed: %s]\n', err_.message);
    end
    try
        R.closure = validation.closure(R, opts.verbose);
    catch err_
        % report, never swallow: a broken cross-check must not look like a passing one
        fprintf('  [closure check failed: %s]\n', err_.message);
    end
end
function v = getfd(s,f,d)
    if isstruct(s) && isfield(s,f) && ~isempty(s.(f)), v = s.(f); else, v = d; end
end

function vprint(on, varargin)
    if on, fprintf(varargin{:}); end
end
