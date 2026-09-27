%% ============================================================================
%  EXAMPLE_16U.m  —  THE MASTER PROPAGATOR. ONE FILE. EVERY KNOB.
%  ----------------------------------------------------------------------------
%  A 16U-class VLEO satellite, propagated with the full force model. Change any
%  value below and re-run to see the effect. Nothing is decided behind your back:
%  every parameter the engine would otherwise pick silently is written out here at
%  its default, with the consequence stated next to it.
%
%  HOW TO USE THIS FILE
%    * Section 1-8 = the knobs. Edit, run, look at the plots.
%    * Section 9   = a SWEEP. Pick one knob, give it a list of values, and the
%                    script propagates once per value and overlays the results.
%                    This is how you see the EFFECT of a parameter, not just its
%                    value. Set SWEEP.on = false for a single run.
%    * Section 11  = every decision actually in force, printed AND saved.
%
%  WHAT "VLEO" CHANGES (why the defaults are what they are)
%    At 250-350 km drag is no longer a perturbation -- it is the second force after
%    gravity, and it varies by a factor of ~10 with solar activity. So: the density
%    model matters more than the gravity degree; the ballistic coefficient Cd*A/m
%    matters more than either; and attitude (which A you present) matters as much as
%    the drag model itself. The knobs are ordered to reflect that.
%% ============================================================================
setup_paths;

%% ------------------------------------------------------- 1. THE SPACECRAFT ---
% A 16U CubeSat: 20 x 20 x 40 cm bus (0.016 m^3), ~24 kg.
SC = struct();
SC.mass_kg  = 24.0;          % 16U at ~1.5 kg/U
SC.Aref_m2  = 0.08;          % 20x40 cm broadside. Use 0.04 for 20x20 nadir-pointing.
                             %   -> Aref is the single biggest lever on lifetime.
SC.Cd       = 2.2;           % 2.0-2.4 typical for a compact body in free-molecular
                             %   flow. At VLEO the REAL Cd depends on gas-surface
                             %   interaction (see DRAG_MODEL below) -- 2.2 is the
                             %   cannonball stand-in, not physics.
SC.Cr       = 1.3;           % SRP reflectivity: 1.0 = fully absorbing, 2.0 = mirror
% Derived, and the number that actually governs decay:
%   B = Cd*A/m = 2.2*0.08/24 = 7.3e-3 m^2/kg. Compare CHAMP: 3.0*0.767/522 = 4.4e-3.
%   -> this 16U is ~1.7x "draggier" per unit mass than CHAMP.

%% --------------------------------------------------------- 2. THE ORBIT -----
ALT_KM   = 300;              % VLEO. Try 250 / 300 / 400 to see the drag cliff.
INC_DEG  = 96.7;             % ~sun-synchronous at this altitude
RAAN_DEG = 0;
ECC      = 0.0;              % near-circular: drag circularises quickly anyway
AOP_DEG  = 0;
TA_DEG   = 0;
EPOCH    = [2024 1 1 0 0 0]; % UTC [Y M D h m s]. Drives the space weather it fetches.
TSPAN_S  = 6*3600;           % arc length. Use 86400*30 for a decay study.

%% --------------------------------------------------------- 3. FORCES -------
% Rough magnitudes at 300 km, for a sense of scale [m/s^2]:
%   gravity 8.9 | J2 1.7e-2 | drag 1e-6..1e-5 | thirdbody 1e-6 | SRP 1e-8 | GR 1e-9
FORCES = struct( ...
  'gravity',   struct('on',true, 'model','sphharm', ...   % twobody|j2|j4|j6|sphharm
                      'degree',70,'order',70), ...        % 70 is overkill below
                                                          % ~1 day; 20 is plenty for
                                                          % a drag study, and much faster
  'drag',      struct('on',true, 'model','cannonball', ...% cannonball|sentman|dria|cll
                      'atmos','nrlmsise', ...             % exponential|nrlmsise|dtm2020|dtm2020_research|jb2008
                      'corotate',true), ...               % false = ignore the wind the
                                                          % rotating atmosphere imparts
                                                          % (~5% of drag at VLEO)
  'thirdbody', struct('on',true, 'model','battin'), ...   % Sun+Moon via DE440
  'srp',       struct('on',true, 'model','cannonball','Cr',SC.Cr), ...
  'relativity',struct('on',true, 'terms',{{'schwarzschild'}}), ...
  'erp',       struct('on',false), ...                    % Earth IR+albedo (~1e-8)
  'solidtides',struct('on',false), 'oceantides',struct('on',false));
GRAV_FIELD = 'EGM2008';      % 'default' SILENTLY caps at degree 6 -- section 11 flags it

%% ------------------------------------------------- 4. SPACE WEATHER --------
% The single largest uncertainty at VLEO. F10.7 swings 65 -> 250 over a solar cycle,
% and density with it by ~10x. A "300 km orbit" has no single lifetime -- it has a
% RANGE, and this is the knob that sets it.
SW_MODE = 'measured';        % 'measured' = real F10.7/ap for EPOCH (needs network)
                             % 'manual'   = fix it yourself, below
SW_MANUAL = struct('F107',150,'F107a',150,'ap',15,'Kp',3.0);
% Useful trio for a sensitivity study:
%   quiet  F107=70,  ap=4     (solar min)
%   mean   F107=150, ap=15
%   active F107=250, ap=80    (solar max + storm)

%% ---------------------------------------------------- 5. INTEGRATOR -------
INTEG_METHOD = 'rk78';       % FIXED : rk4 | nystrom4 | rk6luther | gaussJackson8
                             % ADAPT : rk45 | rk78
                             % MATLAB: ode45 | ode78 | ode89 | ode113
INTEG = struct();
INTEG.rtol     = 1e-11;      % adaptive rel tol   (engine default 1e-9)
INTEG.atol     = 1e-6;       % adaptive abs tol   (engine default 1e-12)
INTEG.h        = [];         % FIXED step [s]. [] -> min(30, span/1000).
                             %   KEEP h <= OUT_DT_S or the dense output degrades.
INTEG.h0       = [];         % [] -> min(span/100,10)
INTEG.hmin     = [];         % [] -> 1e-6
INTEG.hmax     = [];         % [] -> capped at the output spacing (dense-output guard)
INTEG.facmin   = [];         % [] -> 0.2
INTEG.facmax   = [];         % [] -> 5
INTEG.maxsteps = [];         % [] -> 2e6 (also the NaN-loop guard)
% SANITY RULE: a correct setup gives the SAME answer on every integrator. If
% changing INTEG_METHOD changes your result, it is a tolerance/step problem --
% not physics. Section 9 can sweep this to prove it.

%% -------------------------------------------------------- 6. FRAME --------
FRAME = 'C';                 % 'gmst' fast + offline, no EOP
                             % 'A'/'B'/'C' full IAU 2006/2000A + IERS EOP (downloads)
% 'gmst' is self-consistent for a synthetic run like this one and costs nothing.
% It is ONLY wrong when comparing against REAL ITRF measurements -- which is why
% validate_OD uses ITSG data that is already inertial.

%% -------------------------------------------------------- 7. OUTPUT -------
OUT_DT_S = 60;               % report the state every N seconds
SAVE     = true;             % write decisions + results into results/
PLOTS    = true;

%% ------------------------------------------------ 8. WHAT TO PLOT ---------
PLOT_ALTITUDE   = true;      % altitude + the secular decay trend
PLOT_ELEMENTS   = true;      % sma / ecc / inc / RAAN drift
PLOT_FORCES     = true;      % every force's magnitude vs time -- the ladder
PLOT_ENERGY     = true;      % energy drift = the integrator's own error
PLOT_GROUND     = true;      % ground track
PLOT_3D         = true;      % ECI trajectory

%% ------------------------------------------------ 9. SWEEP (the point) ----
% Give one knob a list of values -> one run per value, overlaid. This is how you
% SEE an effect instead of reading a number.
SWEEP = struct();
SWEEP.on     = false;                       % <-- true to enable
SWEEP.knob   = 'SW_MANUAL.F107';            % any knob below, by name:
%   'ALT_KM'                 [250 300 350 400]     the drag cliff
%   'SC.Cd'                  [1.8 2.2 2.6]         Cd sensitivity
%   'SC.Aref_m2'             [0.04 0.08 0.16]      attitude/area effect
%   'SW_MANUAL.F107'         [70 150 250]          solar cycle -> lifetime range
%   'FORCES.gravity.degree'  [4 20 70]             when does gravity degree stop mattering
%   'FORCES.drag.atmos'      {'nrlmsise','dtm2020','dtm2020_research','jb2008'}
%                                                    model spread. NOTE dtm2020 is the
%                                                    OPERATIONAL (F10.7+Kp) variant;
%                                                    dtm2020_research is F30+ap60 --
%                                                    different drivers, different model.
%   'INTEG_METHOD'           {'rk4','rk78','ode45'}           must NOT change the answer
SWEEP.values = [70 150 250];
SWEEP.label  = 'F10.7 [sfu]';

%% ================================================================= 10. RUN ==
K = de440.constants(); mu = K.mu_earth; Re = K.Re_earth;   % not a retyped 6378137
if ~SWEEP.on, vals = {[]};
elseif iscell(SWEEP.values), vals = SWEEP.values;
else, vals = num2cell(SWEEP.values); end
RUNS = cell(1,numel(vals));

for iv = 1:numel(vals)
    % --- apply the sweep value, if any ---
    A_ = ALT_KM; SC_ = SC; F_ = FORCES; SWM_ = SW_MANUAL; IM_ = INTEG_METHOD;
    if SWEEP.on
        % validation.sweep_knob covers FORCES.*/SC.*/INTEG_*; ALT_KM and
        % SW_MANUAL.F107 are this script's own knobs, so they are handled here where
        % you can see them rather than hidden in a shared switch.
        switch SWEEP.knob
            case 'ALT_KM',         A_ = vals{iv};
            case 'SW_MANUAL.F107', SWM_.F107 = vals{iv}; SWM_.F107a = vals{iv};
            case 'SW_MANUAL.ap',   SWM_.ap   = vals{iv};
            otherwise
                SCk = struct('mass_kg',SC_.mass_kg,'Aref_m2',SC_.Aref_m2,'Cd',SC_.Cd,'Cr',SC_.Cr);
                [F_, SCk, IM_, ~] = validation.sweep_knob(SWEEP.knob, vals{iv}, F_, SCk, IM_, struct());
                SC_.mass_kg=SCk.mass_kg; SC_.Aref_m2=SCk.Aref_m2; SC_.Cd=SCk.Cd; SC_.Cr=SCk.Cr;
        end
    end

    % --- initial state from the elements ---
    a = Re + A_*1e3;
    [r0, v0] = op.coe2rv(a, ECC, deg2rad(INC_DEG), deg2rad(RAAN_DEG), ...
                         deg2rad(AOP_DEG), deg2rad(TA_DEG), mu);

    % --- config ---
    cfg = config.defaultConfig();
    cfg.epoch = EPOCH; cfg.r0 = r0(:); cfg.v0 = v0(:); cfg.tspan = TSPAN_S;
    cfg.spacecraft.mass = SC_.mass_kg; cfg.spacecraft.Aref = SC_.Aref_m2;
    cfg.spacecraft.Cd   = SC_.Cd;      cfg.spacecraft.Cr   = SC_.Cr;
    cfg.forces = F_; cfg.forces.srp.Cr = SC_.Cr;
    cfg.gravityField = struct('field',GRAV_FIELD,'degree',F_.gravity.degree);
    cfg.frame = struct('build',FRAME);
    if strcmpi(SW_MODE,'manual'), cfg.spaceweather.manual = SWM_; end
    cfg.output = struct('times',(0:OUT_DT_S:TSPAN_S).');
    cfg.integrator = INTEG; cfg.integrator.method = IM_;
    fn = fieldnames(INTEG);
    for i=1:numel(fn)
        if isempty(INTEG.(fn{i})) && isfield(cfg.integrator,fn{i})
            cfg.integrator = rmfield(cfg.integrator, fn{i});
        end
    end

    W = op.buildWorld(cfg); sol = op.propagate(cfg);
    RUNS{iv} = struct('cfg',cfg,'W',W,'sol',sol,'val',vals{iv}, ...
                      'B', SC_.Cd*SC_.Aref_m2/SC_.mass_kg);
    if SWEEP.on
        fprintf('[sweep] %s = %s\n', SWEEP.knob, validation.val_str(vals{iv}));
    end
end

%% ================================================== 11. DECISIONS TAKEN =====
outdir = '';
if SAVE, outdir = save_results(sprintf('16U_%dkm', ALT_KM)); end
R1 = RUNS{1};
if isempty(outdir), config.report(R1.cfg, R1.W, R1.sol);
else,               config.report(R1.cfg, R1.W, R1.sol, fullfile(outdir,'decisions.txt')); end

fprintf('\n---- 16U summary ----\n');
for iv = 1:numel(RUNS)
    S = RUNS{iv}; alt = (vecnorm(S.sol.r,2,2)-Re)/1000;
    a0 = op.rv2coe(S.sol.r(1,:).',   S.sol.v(1,:).',   mu);
    aF = op.rv2coe(S.sol.r(end,:).', S.sol.v(end,:).', mu);
    decay_m_per_day = (a0-aF)/ (TSPAN_S/86400);
    tag = ''; if SWEEP.on, tag = sprintf('  [%s=%s]', SWEEP.knob, validation.val_str(S.val)); end
    fprintf('  B = Cd*A/m = %.2e m^2/kg%s\n', S.B, tag);
    fprintf('    altitude %.2f -> %.2f km | sma change %+.1f m | decay %.1f m/day\n', ...
            alt(1), alt(end), aF-a0, decay_m_per_day);
    if decay_m_per_day > 0
        yrs = (a0 - (Re+150e3)) / max(decay_m_per_day,eps) / 365.25;
        fprintf('    naive time to 150 km at THIS rate: %.2f years\n', yrs);
        fprintf('    (naive: decay accelerates as it descends, and F10.7 will change)\n');
    end
end
if ~isempty(outdir)
    save(fullfile(outdir,'runs.mat'),'RUNS','-v7');
    fprintf('  saved: %s\n', outdir);
end

%% ======================================================== 12. PLOTS =========
if PLOTS
    show_16U(RUNS, SWEEP, struct('alt',PLOT_ALTITUDE,'elem',PLOT_ELEMENTS, ...
             'forces',PLOT_FORCES,'energy',PLOT_ENERGY,'ground',PLOT_GROUND, ...
             'three_d',PLOT_3D), outdir);
end

%% ============================================================================
%  NO LOCAL FUNCTIONS IN THIS FILE -- ON PURPOSE.
%
%  MATLAB hoists a script's local functions; Octave does NOT -- it defines them
%  only when execution REACHES them, so helpers at the end of a script never exist
%  and every call fails with "'applyKnob' undefined". This file used to carry
%  num2cellAny / valStr / applyKnob down here and would die on the first sweep
%  under Octave. They are now package functions, shared with compare_OD:
%      validation.sweep_knob    validation.val_str
%  and the one-liner is inlined above. Same treatment as validate_OD.
%% ============================================================================
