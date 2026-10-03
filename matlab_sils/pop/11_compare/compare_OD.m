%
%  KNOBS: see 09_docs/KNOBS.md. They are FOUR different kinds and they do not behave
%  alike under a sweep:
%    INDEPENDENT  DRAG_MODEL, ATMOS, SRP_MODEL, ERP_MODEL, ATTITUDE, GEOMETRY,
%                 GRAV_FIELD, FORCES.<x>.on   -- a change here is a real claim
%    DEGENERATE   SC.Cd, SC.Aref, SC.mass     -- rho*Cd*A/m is ONE number on metrics
%                 [1]/[2]; sweep them to SEE the degeneracy, not to fit through it
%    CONDITIONAL  GSI.aT (sentman only), SC.Cr (cannonball only), BOX_ASPECT
%                 (aref_box only) -- a flat sweep here is CORRECT, not a bug.
%                 sat.reads(cfg) prints which your toggles actually consult.
%    NUMERICAL    INTEG_METHOD, rtol/atol -- these MUST NOT move the answer. If one
%                 does, every other row in your table is noise.
%% ============================================================================
%  compare_OD.m  —  WHICH SETUP IS LEAST WRONG?  (SCRIPT: edit top, run, read table)
%  ============================================================================
%  validate_OD answers "is my force model right for THIS setup?".
%  This answers the next question: "of N setups, which one is least wrong, and by
%  how much -- against the same real satellite, on the same arc, at the same epochs?"
%
%  It is validate_OD's four metrics, evaluated once per sweep value:
%    (1) position vs reducedDynamicOrbit        [m RMS, RTN]
%    (2) position vs kinematicOrbit             [m RMS] -- INDEPENDENT of any force model
%    (3) non-conservative force vs accelerometer[m/s^2] -- tests Cd*A/m + density DIRECTLY
%    (4) density vs measured                    [ratio] -- tests the atmosphere alone
%  Both scripts call validation.od_metrics, so there is exactly ONE implementation
%  of each metric. They cannot drift apart.
%
%  ---------------------------------------------------------------------------
%  READ THIS BEFORE BELIEVING THE TABLE
%  ---------------------------------------------------------------------------
%  * THE REFERENCE HAS A FLOOR. `refSpread` (kinematic - reducedDynamic) is the
%    reference's OWN uncertainty, ~2-8 cm for ITSG. Two configs whose residuals
%    differ by less than that are NOT distinguishable, and the ranking between
%    them is noise. The table marks that line explicitly. Do not tune below it.
%  * (1) IS THE WEAKEST METRIC, not the strongest. The reduced-dynamic orbit was
%    itself produced with a force model and empirical accelerations that absorb
%    exactly the mismodelling you are trying to measure. A config can win on (1)
%    by resembling TU Graz's model rather than by resembling physics. (3) and (4)
%    are measurements; prefer them when they disagree with (1).
%  * DRAG AND Cd ARE DEGENERATE ON (1) AND (2). Cd*A/m and density enter the
%    equations of motion only as a product. A 20% density bias and a 20% Cd error
%    are indistinguishable from position alone -- that is the whole reason the
%    accelerometer (3) and the density (4) comparisons exist. Sweeping SC.Cd will
%    happily "improve" (1) while making the physics worse.
%  * ONE ARC IS ONE SAMPLE. A 3 h arc on one day at one solar activity level does
%    not rank atmosphere models. It tells you what happened on that arc. Sweep
%    DATE too before you believe any ordering.
%
%  ---------------------------------------------------------------------------
%  HOW THE SWEEP WORKS  (same pattern as EXAMPLE_16U section 9)
%  ---------------------------------------------------------------------------
%  Pick one knob, give it a list of values -> one propagation per value, all
%  compared against the SAME reference products, fetched ONCE. Set SWEEP.on=false
%  for a single run (which is then just validate_OD without the plots).
%% ============================================================================
setup_paths;

%% ---------------------------------------------------------------- 1. CASE ----
SAT      = 'CHAMP';            % itsg.list for all 23 names + coverage
DATE     = '2007-01-01';       % UTC day, inside that satellite's coverage
TSPAN_S  = 3*3600;             % arc length [s]
OUT_DT_S = 30;                 % output spacing [s]. Sampling is Hermite on (r,v),
                               % so this is NOT the accuracy limiter it looks like.

%% ------------------------------------------------------------ 2. WHAT TO RUN --
DO_RDO      = true;            % (1) vs reducedDynamicOrbit  (also the seed source)
DO_KIN      = true;            % (2) vs kinematicOrbit       (independent)
DO_ACC      = true;            % (3) accelerometer           (CHAMP/GRACE/GRACE-FO only)
DO_DENSITY  = true;            % (4) measured density
DO_TUDELFT_DENSITY = true;     % (5) ALSO compare to TU Delft measured density
DENSITY_PRODUCT = 'auto';      % 'auto' | 'neutralDensity' | 'neutralDensity_ACC'
FORCE_REFETCH   = false;       % ITSG products are immutable -> cached forever

%% ---------------------------------------------------------- 3. SPACECRAFT ----
USE_CATALOG = true;            % mass/area/Cd/Cr from itsg_catalog.csv
SC = struct('mass_kg',[], 'Aref_m2',[], 'Cd',[], 'Cr',[]);   % [] = take the catalog value

%% --------------------------------------------------------------- 4. FORCES ---
% The BASELINE. The sweep perturbs one knob of this; everything else is held.
FORCES = struct( ...
  'gravity',   struct('on',true, 'model','sphharm','degree',70,'order',70), ...
  'drag',      struct('on',true, 'model','cannonball','atmos','nrlmsise','corotate',true), ...
  'thirdbody', struct('on',true, 'model','battin'), ...
  'srp',       struct('on',true, 'model','cannonball'), ...
  'relativity',struct('on',true, 'terms',{{'schwarzschild'}}), ...
  'erp',       struct('on',false), 'solidtides',struct('on',false), 'oceantides',struct('on',false));
GRAV_FIELD   = 'EGM2008';      % 'default' SILENTLY caps at degree 6
SW_MANUAL    = [];             % [] = measured F10.7/ap; else struct('F107',..,'ap',..)
INTEG_METHOD = 'rk78';
INTEG        = struct('rtol',1e-11,'atol',1e-6);
FRAME        = 'gmst';         % ITSG r,v are already inertial, so no frame is built
                               % on the main path. Only the r1.0 density branch uses
                               % this (ECI->ECEF for the geodetic sample point).

%% ------------------------------------------------ 5. SWEEP (the whole point) --
SWEEP = struct();
SWEEP.on     = true;                           % false -> single baseline run
SWEEP.knob   = 'FORCES.drag.atmos';
SWEEP.values = {'exponential','nrlmsise','dtm2020','dtm2020_research','jb2008'};
SWEEP.label  = 'atmosphere model';

%  LEGAL KNOBS -- and what each one actually asks:
%   'FORCES.drag.atmos'      {'exponential','nrlmsise','dtm2020','dtm2020_research','jb2008'}
%         Each of these eats a DIFFERENT driver set, resolved per-model by
%         data.drivers (see 09_docs/DRIVERS_AUDIT.md):
%           nrlmsise         F10.7 + ap (+57h aph history)
%           dtm2020          F10.7 + Kp          (OPERATIONAL, dtm3)
%           dtm2020_research F30   + ap60        (RESEARCH, dtm5 -- a different
%                                                 model, not a refinement)
%           jb2008           F10/S10/M10/Y10 + DSTDTC  (SET files; ignores F10.7)
%         So this sweep is also a test of the driver plumbing, not just the physics.
%         The real question at low LEO. Read metric (4) FIRST, not (1): (4) is the
%         atmosphere on its own, (1) is the atmosphere convolved with Cd and arc
%         length. 'exponential' is the control -- it has no space weather at all,
%         so it shows you how much of your skill is really coming from F10.7/ap.
%   'FORCES.drag.model'      {'cannonball','sentman','dria','cll'}
%         Gas-surface interaction. Only meaningful on (3): it changes the DIRECTION
%         and magnitude of the force, and only the accelerometer sees direction.
%         Needs panel geometry for the non-cannonball models.
%   'FORCES.gravity.degree'  [4 20 40 70]
%         Where gravity stops mattering. On a 3 h arc the answer is usually ~20;
%         if 70 beats 20 by more than refSpread, something else is absorbing it.
%         Needs GRAV_FIELD='EGM2008' -- 'default' caps at 6 and the table will
%         show four identical rows, which is the tell.
%   'SC.Cd'                  [2.5 3.0 3.5]
%         DEGENERATE with density on (1) and (2) -- see the warning above. Sweep it
%         to SEE the degeneracy, not to fit it. Watch (3) and (4) stay put while
%         (1) moves: that is the degeneracy, drawn.
%   'SC.Aref'                [0.7 0.767 0.85]      same degeneracy, same warning
%
%% ===========================================================================
%  DATA PROVENANCE IN A SWEEP -- WHAT YOU ARE ACTUALLY MEASURING
%
%  A sweep does not measure the atmosphere. It measures HOW MUCH THE ANSWER DEPENDS
%  ON ONE INPUT. What that is worth depends entirely on which KIND of input you
%  swept, and the three cases are not comparable:
%
%    Sweeping an ASSUMED input (Cd, Cr, GSI.Tw, OPTICS, BOX_ASPECT, lag_f107)
%      -> you are measuring YOUR OWN UNCERTAINTY. If the table moves a lot, the
%         honest report is a BAND, not a number. This is the most valuable thing
%         this script does and the least often done.
%
%    Sweeping a FETCHED input (F10.7 source, gravity field, EOP)
%      -> you are measuring how much someone else's data choice costs you.
%
%    Sweeping a MODEL (drag.model, srp.model, erp.model, atmos)
%      -> you are measuring model spread, which is the closest thing available to
%         an error bar on a quantity nobody can measure directly.
%
%    Sweeping a NUMERICAL knob (INTEG_METHOD, INTEG.rtol)
%      -> this MUST NOT move the answer. If it does, every other row in the table
%         is noise and you are reading tea leaves.
%
%  ---- THE DEGENERACY THAT WILL FOOL YOU --------------------------------------
%  On metrics [1]/[2] (position), Cd and density are DEGENERATE: rho*Cd*A/m is one
%  number and the orbit cannot tell you which factor was wrong. Sweep SC.Cd and the
%  position residual moves beautifully -- and you have learned NOTHING about Cd,
%  only that the product is off. Metrics [3] (accelerometer) and [4]/[5] (density)
%  break the degeneracy because they see the factors separately.
%    -> Cd(t) from a GSI model is NOT degenerate the same way: it has a SHAPE with
%       altitude and local time that a scalar cannot mimic. That is why
%       FORCES.drag.model is the sweep worth running against metric [3].
%
%  ---- A FLAT COLUMN IS AMBIGUOUS ---------------------------------------------
%  It means either "the model does not consult this knob" (correct -- 'dria'
%  DERIVES aT and ignores GSI.aT) or "the knob is dropped on the floor" (a bug).
%  They look identical. 11_compare/verify_sweep_matrix.m tests every knob against
%  its documented expectation, and sat.reads(cfg) tells you which fields your
%  toggles read BEFORE you spend the arcs.
%% ===========================================================================

%   ---- THE MODEL KNOBS. These are the ones worth your time on metric [3]. -----
%   'FORCES.drag.model'      {'cannonball','sentman','dria','cll','sesam'}
%         The real question is not which GSI model but whether the accommodation
%         coefficient is TYPED or COMPUTED. 'sentman' takes gsi.aT (everyone types
%         0.9) and is therefore FLAT with altitude by construction. 'dria'/'sesam'
%         derive aT from the local atomic-oxygen density: measured 2.25 at 250 km
%         rising to 3.35 at 550 km as O thins and accommodation falls. 'sesam' is
%         an alias of 'dria' -- SESAM supplies aT, DRIA is Sentman using it.
%         NOT degenerate with density the way SC.Cd is: Cd(t) has a SHAPE that a
%         scalar cannot mimic, so (3) and (4) can tell them apart.
%   'FORCES.drag.gsi.aT'     [0.7 0.8 0.9 1.0]
%         Only 'sentman' reads it -- 'dria'/'sesam' compute aT and IGNORE this, so
%         a flat table here means you swept a knob the model does not consult.
%         sat.reads(cfg) will tell you which before you spend the arcs.
%   'FORCES.srp.model'       {'cannonball','boxwing'}
%   'FORCES.erp.model'       {'knocke','simple','ceres','boxwing'}
%         Both box-wings need SC.facets AND SC.attitude. Without a geometry they
%         error rather than quietly falling back -- see validation.capability(SAT),
%         which says which models the satellite's DATA can support at all. No
%         CHAMP/GRACE dimensions exist in this toolbox, so box-wing is 16U-only.
%   'SC.attitude'            {[], 'ram', struct('mjd',..,'q',..)}
%         [] with a panel model is a TRAP: R_bi = eye(3) puts the body axes on the
%         INERTIAL axes, the plate faces inertial +x instead of the flow, and the
%         drag collapses toward ZERO. validate_OD errors on that pair; here you
%         would just get a suspiciously good-looking row.
%   'SC.facets'              {plate, boxwing}   srp-format, ONE set for drag+SRP
%   'INTEG_METHOD'           {'rk4','rk78','ode89'}
%         MUST NOT change the answer. If it does, it is a tolerance/step problem,
%         not physics. This is the sweep that proves the rest of the table means
%         anything -- run it FIRST, once, and then forget about it.
%   'INTEG.rtol'             [1e-9 1e-11 1e-13]    same purpose
%   'FORCES.relativity.on'   {true,false}          ~1e-9 m/s^2: expect (1) to move
%                                                  a few cm over 3 h, below the floor
%   'FORCES.erp.on'          {true,false}          ~1e-8: visible on (3), not (1)
%   'FORCES.srp.on'          {true,false}          eclipse-dependent; visible on (3)
%   'DATE'                   {'2007-01-01','2007-07-01'}
%         The honest one. Same config, different arc. The spread ACROSS dates is
%         your real error bar; the spread across models on ONE date is not.

%% -------------------------------------------------------------- 6. OUTPUT ----
SAVE  = true;
PLOTS = true;

%% ================================================================== 7. RUN ===
K = de440.constants(); mu = K.mu_earth; Re = K.Re_earth;   % not a retyped 6378137
if SWEEP.on, knobName = SWEEP.knob; else, knobName = '(single run)'; end
opts_f = struct(); if FORCE_REFETCH, opts_f.force = true; end
if ~SWEEP.on, vals = {[]};
elseif iscell(SWEEP.values), vals = SWEEP.values;
else, vals = num2cell(SWEEP.values); end

CAT = validation.itsg_catalog(SAT);
fprintf('\n===== compare_OD: %s %s, %.1f h arc, %d configs =====\n', ...
        SAT, DATE, TSPAN_S/3600, numel(vals));

% ---- 7a. reference products: fetched ONCE, shared by every run ---------------
% This is why REF is built out here and not inside the loop. ITSG products are
% immutable; re-reading them per sweep value would be pure waste, and would also
% let a transient network failure silently change the truth mid-sweep.
REF = struct(); prov = {};
RDO = data.itsg(SAT, DATE, 'reducedDynamicOrbit', opts_f);
REF.RDO = RDO; prov{end+1} = RDO.source;
mjd0  = RDO.mjd(1);
epoch = validation.mjd2utc(mjd0);
t0    = (RDO.mjd - mjd0)*86400;
sel0  = t0 >= 0 & t0 <= TSPAN_S;
r0 = RDO.r(find(sel0,1),:).';  v0 = RDO.v(find(sel0,1),:).';   % the seed: file's own r,v
fprintf('[seed] %s from reducedDynamicOrbit: |r0|=%.3f km |v0|=%.4f km/s (no derivation, no rotation)\n', ...
        SAT, norm(r0)/1000, norm(v0)/1000);
if DO_KIN
    try, REF.KIN = data.itsg(SAT, DATE, 'kinematicOrbit', opts_f); prov{end+1} = REF.KIN.source;
    catch ME, fprintf('  kinematicOrbit unavailable: %s\n', regexprep(ME.message,'\n.*','')); end
end
if DO_ACC && strcmpi(CAT.has_acc,'yes')
    try
        REF.ACC = data.itsg(SAT, DATE, 'nonConservativeForces', opts_f); prov{end+1} = REF.ACC.source;
        REF.ATT = data.itsg(SAT, DATE, 'attitude', opts_f);              prov{end+1} = REF.ATT.source;
    catch ME, fprintf('  accelerometer/attitude unavailable: %s\n', regexprep(ME.message,'\n.*','')); end
elseif DO_ACC
    fprintf('  %s has no accelerometer on ITSG -- metric (3) skipped (see itsg.list).\n', SAT);
end
if DO_DENSITY && strcmpi(CAT.has_density,'yes')
    switch lower(DENSITY_PRODUCT)
        case 'auto'
            % CHAMP's only density release is neutralDensity_ACC. Asking for
            % 'neutralDensity' 404s -- which used to be swallowed silently.
            if strcmpi(CAT.dir,'CHAMP'), dprod = 'neutralDensity_ACC';
            else,                        dprod = 'neutralDensity'; end
        otherwise, dprod = DENSITY_PRODUCT;
    end
    try, REF.DEN = data.itsg(SAT, DATE, dprod, opts_f); prov{end+1} = REF.DEN.source;
    catch ME, fprintf('  %s unavailable: %s\n', dprod, regexprep(ME.message,'\n.*','')); end
end


%% ---- (5) TU Delft measured density: an INDEPENDENT second truth -------------
% ITSG's neutralDensity and TU Delft's density come from different groups doing
% different processing of the same accelerometer, with different Cd and
% gas-surface assumptions. Neither is "measured density" in an absolute sense --
% both are retrievals. So the gap BETWEEN them is the real floor under any density
% claim we make, exactly as refSpread is for position. Matching one and not the
% other is a fact about the truth, not about us.
%
% NOTE the knob is DO_TUDELFT_DENSITY, NOT DO_TUDELFT: the latter already exists
% and means the POSITION overlay. Two different products, two different questions.
if DO_TUDELFT_DENSITY && CAT.has_tudelft
    try
        Ttd = data.tudelft_density(CAT.tudelft_name, DATE, DATE);
        REF.TUD = validation.tudelft_to_ref(Ttd);
        prov{end+1} = struct('product','TU Delft density','date',DATE, ...
                             'provider','TU Delft thermosphere.tudelft.nl','frame','geodetic', ...
                             'nEpochs',numel(REF.TUD.mjd),'url','https://thermosphere.tudelft.nl/data/');
    catch ME
        fprintf('[5] TU Delft density unavailable: %s\n', regexprep(ME.message,'\n.*',''));
    end
elseif DO_TUDELFT_DENSITY
    fprintf('[5] %s has no TU Delft density (itsg_catalog.csv has_tudelft=no) -- skipped.\n', SAT);
end

% ---- 7b. one propagation per sweep value ------------------------------------
RUNS = cell(1, numel(vals));
for iv = 1:numel(vals)
    F_ = FORCES; SC_ = SC; IM_ = INTEG_METHOD; IN_ = INTEG; GF_ = GRAV_FIELD;
    if SWEEP.on
        [F_, SC_, IM_, IN_] = validation.sweep_knob(SWEEP.knob, vals{iv}, F_, SC_, IM_, IN_);
    end

    cfg = config.defaultConfig();
    cfg.epoch = epoch; cfg.r0 = r0; cfg.v0 = v0; cfg.tspan = TSPAN_S;
    cfg.forces = F_;
    cfg.gravityField = struct('field',GF_,'degree',F_.gravity.degree);
    cfg.frame = struct('build',FRAME);
    if USE_CATALOG
        % canonical names throughout now -- no rename in flight (see sat.spacecraft)
        cfg.spacecraft.mass = CAT.mass; cfg.spacecraft.Aref = CAT.Aref;
        cfg.spacecraft.Cd   = CAT.Cd;   cfg.spacecraft.Cr   = CAT.Cr;
    end
    % attitude and geometry must reach cfg.spacecraft or the panel/box-wing knobs
    % are set and then dropped -- a sweep that silently does nothing is worse than
    % an error, because the flat result looks like a physics finding.
    if isfield(SC_,'attitude'), cfg.spacecraft.attitude = SC_.attitude; end
    if isfield(SC_,'facets') && ~isempty(SC_.facets), cfg.spacecraft.facets = SC_.facets; end
    if ~isempty(SC_.mass_kg), cfg.spacecraft.mass = SC_.mass_kg; end
    if ~isempty(SC_.Aref_m2), cfg.spacecraft.Aref = SC_.Aref_m2; end
    if ~isempty(SC_.Cd),      cfg.spacecraft.Cd   = SC_.Cd;      end
    if ~isempty(SC_.Cr),      cfg.spacecraft.Cr   = SC_.Cr;      end
    cfg.forces.srp.Cr = cfg.spacecraft.Cr;
    if ~isempty(SW_MANUAL), cfg.spaceweather.manual = SW_MANUAL; end
    cfg.output     = struct('times', (0:OUT_DT_S:TSPAN_S).');
    cfg.integrator = IN_; cfg.integrator.method = IM_;

    lbl = validation.val_str(vals{iv});
    fprintf('\n--- [%d/%d] %s = %s ---\n', iv, numel(vals), knobName, lbl);
    try
        W   = op.buildWorld(cfg);
        sol = op.propagate(cfg);
        R   = validation.od_metrics(sol, W, REF, struct( ...
                 'mjd0',mjd0, 'tspan_s',TSPAN_S, 'epoch',epoch, ...
                 'frame',FRAME, 'atmos',cfg.forces.drag.atmos, 'verbose',true));
        RUNS{iv} = struct('label',lbl, 'val',vals{iv}, 'cfg',cfg, 'W',W, 'sol',sol, ...
                          'R',R, 'B', cfg.spacecraft.Cd*cfg.spacecraft.Aref/cfg.spacecraft.mass, ...
                          'ok',true, 'err','');
    catch ME
        % A config that dies is a RESULT, not a reason to abort the sweep.
        fprintf('    FAILED: %s\n', regexprep(ME.message,'\n.*',''));
        RUNS{iv} = struct('label',lbl, 'val',vals{iv}, 'cfg',cfg, 'W',[], 'sol',[], ...
                          'R',struct(), 'B',NaN, 'ok',false, 'err',regexprep(ME.message,'\n.*',''));
    end
end

%% ============================================================ 8. THE TABLE ===
outdir = '';
if SAVE, outdir = save_results(sprintf('compareOD_%s_%s', SAT, DATE)); end
floorM = NaN;
for iv = 1:numel(RUNS)
    if RUNS{iv}.ok && isfield(RUNS{iv}.R,'refSpread'), floorM = RUNS{iv}.R.refSpread; break, end
end

fprintf('\n');
fprintf('================================================================================\n');
fprintf(' compare_OD   %s   %s   %.1f h arc\n', SAT, DATE, TSPAN_S/3600);
fprintf(' sweep: %s\n', knobName);
fprintf('================================================================================\n');
fprintf(' %-14s | %9s %9s | %9s | %9s %6s | %7s\n', ...
        knobName, 'rdo|3D|', 'rdo dv', 'kin|3D|', 'acc gap', 'ratio', 'rho');
fprintf(' %-14s | %9s %9s | %9s | %9s %6s | %7s\n', ...
        '', '[m]', '[m/s]', '[m]', '[m/s^2]', 'mod/ms', 'mod/ms');
fprintf('--------------------------------------------------------------------------------\n');
for iv = 1:numel(RUNS)
    S = RUNS{iv};
    if ~S.ok
        fprintf(' %-14s | FAILED: %s\n', S.label, S.err); continue
    end
    R = S.R;
    accR = NaN;
    % validation.acc_ratio, NOT an inline divide: this SWEEPS against metric [3],
    % and the raw ratio is ~drag/bias with a constant denominator 10x the signal.
    % Sweeping against it returns a nearly FLAT table -- which reads as "this knob
    % does not matter" and is a confident wrong answer.
    if isfield(R,'acc'), accR = validation.acc_ratio(R); end
    fprintf(' %-14s | %9.3f %9.5f | %9.3f | %9.3e %6.3f | %7.3f\n', ...
            S.label, getm(R,'rdo','pos3D'), getm(R,'rdo','dv'), getm(R,'kin','pos3D'), ...
            getm(R,'acc','rms_diff'), accR, getm(R,'den','ratio'));
end
fprintf('--------------------------------------------------------------------------------\n');
if isfinite(floorM)
    fprintf(' REFERENCE FLOOR (kinematic - reducedDynamic) = %.3f m RMS\n', floorM);
    fprintf(' -> any two rows whose |3D| differ by less than this are NOT distinguishable.\n');
end
fprintf(' rdo|3D| ranks agreement with TU Graz''s force model, not with physics.\n');
fprintf(' Trust the acc gap and rho ratio first: those are measurements.\n');
fprintf('================================================================================\n');

% ---- provenance: every input, tagged ----------------------------------------
fprintf('\n---- PROVENANCE ----\n');
for i = 1:numel(prov)
    p = prov{i};
    fprintf('  %-24s %-12s %s\n', p.product, p.date, p.provider);
    fprintf('  %-24s frame=%s  epochs=%d  %s\n', '', p.frame, p.nEpochs, p.url);
end

if ~isempty(outdir)
    % CSV, because a sweep is a table and a table belongs in a file
    fid = fopen(fullfile(outdir,'compare_OD.csv'),'w');
    fprintf(fid,'knob,value,ok,rdo_pos3D_m,rdo_radial_m,rdo_along_m,rdo_cross_m,rdo_dv_mps,');
    fprintf(fid,'kin_pos3D_m,refSpread_m,acc_rms_meas,acc_rms_mod,acc_rms_diff,acc_ratio,');
    fprintf(fid,'rho_ratio,rho_rms_logerr,B_m2perkg,error\n');
    for iv = 1:numel(RUNS)
        S = RUNS{iv}; R = S.R;
        ar = NaN; if isfield(R,'acc'), ar = validation.acc_ratio(R); end
        rs = NaN; if isfield(R,'refSpread'), rs = R.refSpread; end
        fprintf(fid,'%s,%s,%d,%g,%g,%g,%g,%g,%g,%g,%g,%g,%g,%g,%g,%g,%g,%s\n', ...
            knobName, S.label, S.ok, getm(R,'rdo','pos3D'), getm(R,'rdo','radial'), ...
            getm(R,'rdo','along'), getm(R,'rdo','cross'), getm(R,'rdo','dv'), getm(R,'kin','pos3D'), rs, ...
            getm(R,'acc','rms_meas'), getm(R,'acc','rms_mod'), getm(R,'acc','rms_diff'), ar, ...
            getm(R,'den','ratio'), getm(R,'den','rms_logerr'), S.B, S.err);
    end
    fclose(fid);
    save(fullfile(outdir,'compare_OD.mat'), 'RUNS','SWEEP','REF','prov','-v7');
    ok1 = find(cellfun(@(s) s.ok, RUNS), 1);
    if ~isempty(ok1)
        config.report(RUNS{ok1}.cfg, RUNS{ok1}.W, RUNS{ok1}.sol, fullfile(outdir,'decisions.txt'));
    end
    fprintf('\n  saved: %s\n', outdir);
end

%% ============================================================== 9. FIGURES ===
if PLOTS
    show_compare_OD(RUNS, SWEEP, sprintf('%s  %s', SAT, DATE), outdir);
end

%% ============================================================================
%  NO LOCAL FUNCTIONS IN THIS FILE -- ON PURPOSE.
%
%  MATLAB hoists a script's local functions; Octave does NOT. Octave defines them
%  only when execution REACHES them, so helpers sitting at the end of a script are
%  never defined and every call to one fails with "'applyKnob' undefined". That is
%  precisely how validate_OD broke, and the first draft of THIS file reproduced the
%  same bug within an hour of the fix. So the helpers are package functions:
%      validation.sweep_knob   validation.val_str   validation.od_metrics
%  and the two-line ones are inlined above.
%
%  KNOWN, NOT FIXED: EXAMPLE_16U.m still has this shape (num2cellAny / valStr /
%  applyKnob at the end of a script) and will fail identically under Octave. It is
%  fine on MATLAB. If you ever run the test suite under Octave, lift those three
%  the same way -- validation.sweep_knob already covers applyKnob's job.
%% ============================================================================
