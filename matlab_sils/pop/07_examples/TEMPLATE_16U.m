% TEMPLATE_16U  End-to-end low LEO 16U propagation with EVERY model reachable.
%
%   RUN:  setup_paths; TEMPLATE_16U
%
%   ---------------------------------------------------------------------------
%   WHY THE 16U AND NOT CHAMP
%   ---------------------------------------------------------------------------
%   This is the ONLY spacecraft in the toolbox with a real geometry
%   (sgeom.sat16u: a 0.20 x 0.20 x 0.34 m bus + two arrays, with optics). So it is
%   the only one where box-wing SRP and box-wing ERP can run at all. CHAMP, GRACE
%   and Swarm have mass, area, Cd and Cr in itsg_catalog.csv and NO dimensions --
%   run validation.capability('CHAMP') and it will tell you so, and validate_OD
%   refuses those models rather than inventing a box.
%
%   The trade is the opposite one: the 16U has a geometry and NO measured truth.
%   CHAMP has truth and no geometry. This file is where you see the models behave;
%   validate_OD is where you find out if they are right.
%
%   ---------------------------------------------------------------------------
%   WHAT THIS WRITES
%   ---------------------------------------------------------------------------
%   force_data/<tag>/ :  one .csv per force, plus accel_total.csv and a manifest.
%   Segregated on purpose -- "the drag was wrong" is answerable from drag.csv
%   alone, and you can diff two runs force by force.
%
%% ===========================================================================
%  DATA PROVENANCE FOR THE 16U -- AND IT IS THE MIRROR IMAGE OF validate_OD's
%
%  validate_OD runs CHAMP: measured truth, no geometry.
%  This runs a 16U:        real geometry, NO measured truth.
%  Neither is the whole picture, and knowing WHICH you are looking at is the
%  difference between a validation and a demonstration. This file is a
%  DEMONSTRATION: every model runs, nothing here is checked against reality.
%
%  ---- MEASURED: nothing. ------------------------------------------------------
%    There is no flight data for this spacecraft. Not one number below is compared
%    against an instrument. That is not a defect of the file, it is the file's
%    purpose -- but it means "the ladder looks right" is the strongest claim
%    available here, and "the drag is correct" is not.
%
%  ---- FETCHED: same open sources as validate_OD -------------------------------
%    F10.7, F10.7a      OMNI2 (NASA GSFC)   -- unless SW_MANUAL is set below, in
%    Kp, ap             GFZ Potsdam            which case they are ASSUMED too
%    EOP xp,yp,dUT1     IERS
%    Sun/Moon           JPL DE440
%    gravity field      GRAV_FIELD knob: 'default' is BUNDLED (assumed, caps at
%                       degree 6); 'EGM2008' is FETCHED from ICGEM (needs network)
%
%  ---- REAL, and unique to this satellite: THE GEOMETRY ------------------------
%    sgeom.sat16u   0.20 x 0.20 x 0.34 m bus + two tracking arrays, with optics.
%                    This is the ONLY real geometry in the toolbox, which is why
%                    this is the only spacecraft where box-wing SRP and box-wing
%                    ERP can run at all without assuming a shape.
%                    (dgeom.sat16u used to describe a DIFFERENT 16U -- long axis
%                     on X instead of Z, 1.7x the frontal area. It now forwards
%                     here and warns. One geometry, one spacecraft.)
%
%  ---- ASSUMED: everything else. Every one is a knob. --------------------------
%    SW_MANUAL      F107/F107a/ap/Kp typed below -> the space weather is ASSUMED,
%                   not fetched. Set it to [] to fetch the real indices (network).
%    SC.Cd, SC.Cr   only read by the cannonball models; the panel/box-wing models
%                   derive or ignore them. sat.reads(cfg) prints which, below.
%    GSI            Tw/aT/sig_n/sig_t -- chosen. 'dria' DERIVES aT and ignores it.
%    facet optics   the alpha/rho_s/rho_d inside sgeom.sat16u are plausible
%                   MLI/solar-cell values, NOT measured, and ONE set serves both
%                   the visible and thermal-IR bands.
%    ATTITUDE='ram' modelled, not measured. There is nothing to measure.
%    the ORBIT      ALT_KM/INC_DEG below are a scenario you invented.
%
%  ---- THE RULE ---------------------------------------------------------------
%  Every number this file prints is a model talking to itself. That is genuinely
%  useful -- it is how you see a model's behaviour without a measurement arguing
%  back -- and it is not evidence about the world. For evidence, use validate_OD.
%% ===========================================================================

%% ============================== EPOCH & ORBIT ===============================
EPOCH   = [2007 1 1 0 0 0];
ALT_KM  = 300;                 % low LEO
INC_DEG = 96.5;                % sun-synchronous-ish
TSPAN_S = 1800;                % 30 min. Not an orbit: see ERP_RINGS below -- box-wing
                               % ERP is expensive and a full orbit of it is minutes.
OUT_DT  = 30;

%% ============================== SPACECRAFT ==================================
% ONE geometry, srp-format, feeding drag AND srp AND erp. Two facet sets would be
% two spacecraft -- which is exactly what dgeom.sat16u vs sgeom.sat16u used to be
% (long axis on X vs Z, 1.7x different frontal area, no complaint from either).
SC16 = sgeom.sat16u();
SC   = struct('mass', SC16.mass, ...
              'Aref', 0.20*0.20, ...      % frontal (ram) face -- cannonball only
              'Cd',   2.2, ...            % cannonball only; panel models DERIVE Cd(t)
              'Cr',   1.3, ...            % cannonball only; boxwing uses facet optics
              'facets', SC16.facets, ...
              'R_bi', eye(3));

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
%% ================================ TOGGLES ===================================
% Change ONE and re-run to see it move. sat.reads(cfg) below prints which of the
% fields above each choice actually consults.
ATTITUDE   = 'ram';            % 'none' | 'ram' | 'measured'({.mjd,.q}) | 3x3 | @(utc)
DRAG_MODEL = 'dria';           % cannonball | sentman | dria | cll | sesam
ATMOS      = 'dtm2020';        % exponential | nrlmsise | dtm2020 | dtm2020_research | jb2008
SRP_MODEL  = 'boxwing';        % cannonball | boxwing
ERP_MODEL  = 'boxwing';        % knocke | simple | ceres | boxwing
ERP_RINGS  = 8;                % Earth-cap integration for erp 'boxwing'. Knocke's
ERP_SEGS   = 16;               % default is 16x48 = 768 ELEMENTS, and box-wing runs
                               % every facet at every element -- 768*8 = 6k facet
                               % evaluations PER RHS CALL, inside rk78. A full orbit
                               % is ~10^7 of them and takes minutes. The cap integral
                               % converges quickly, so 8x16 = 128 costs 6x less and
                               % is usually within a percent. Raise it and time it
                               % yourself rather than trusting this comment.
GSI        = struct('Tw',300,'aT',0.9,'sig_n',0.9,'sig_t',0.9);
GRAV_DEG   = 20;
GRAV_FIELD = 'default';        % 'default' (bundled, caps ~6-20, works OFFLINE)
                               % 'EGM2008' (auto-downloads from ICGEM; needs network)
INTEG      = struct('method','rk78','rtol',1e-11,'atol',1e-6);
SW_MANUAL  = struct('F107',90,'F107a',90,'ap',8,'Kp',2);   % [] = measured (network)
TAG        = sprintf('16U_%s_%s_srp-%s_erp-%s', DRAG_MODEL, ATMOS, SRP_MODEL, ERP_MODEL);
% force_data lives at the TOOLBOX ROOT, not next to this script. This file moved
% into 07_examples/ in round 24 and a path relative to mfilename would have started
% writing 07_examples/force_data/ silently -- the outputs would still appear, just
% in the wrong place, which is the kind of "working" that wastes an afternoon.
SAVE_DIR   = fullfile(fileparts(fileparts(mfilename('fullpath'))), 'force_data', TAG);
DO_IO_PLOT = true;

%% ============================== BUILD CONFIG ================================
K   = de440.constants();
a   = K.Re_earth + ALT_KM*1000;
vc  = sqrt(K.mu_earth/a);
inc = deg2rad(INC_DEG);

cfg = config.defaultConfig();
cfg.epoch  = EPOCH;
cfg.r0     = [a; 0; 0];
cfg.v0     = [0; vc*cos(inc); vc*sin(inc)];
cfg.tspan  = TSPAN_S;
cfg.output = struct('times', (0:OUT_DT:TSPAN_S).');
cfg.integrator   = INTEG;
cfg.gravityField = struct('field',GRAV_FIELD,'degree',GRAV_DEG);
cfg.spacecraft   = SC;
cfg.spacecraft.attitude = ATTITUDE;
if ~isempty(SW_MANUAL), cfg.spaceweather.manual = SW_MANUAL; end

cfg.forces = struct( ...
  'gravity',    struct('on',true,  'model','sphharm','degree',GRAV_DEG,'order',GRAV_DEG), ...
  'drag',       struct('on',true,  'model',DRAG_MODEL,'atmos',ATMOS,'corotate',true,'gsi',GSI), ...
  'thirdbody',  struct('on',true,  'model','battin'), ...
  'srp',        struct('on',true,  'model',SRP_MODEL), ...
  'erp',        struct('on',true,  'model',ERP_MODEL,'nrings',ERP_RINGS,'nseg',ERP_SEGS), ...
  'relativity', struct('on',true,  'terms',{{'schwarzschild'}}), ...
  'solidtides', struct('on',false), ...
  'oceantides', struct('on',false));

%% ================================ THE PLAN ==================================
fprintf('\n================================================================\n');
fprintf(' TEMPLATE_16U   %d km   inc %.1f deg   %.0f s\n', ALT_KM, INC_DEG, TSPAN_S);
fprintf('================================================================\n');
fprintf('  geometry   sgeom.sat16u: %d facets, %.1f kg (REAL dims -- the only\n', ...
        numel(SC.facets), SC.mass);
fprintf('             spacecraft here that has them, hence the only one that can\n');
fprintf('             run box-wing SRP/ERP at all)\n');
fprintf('  attitude   %s\n', ATTITUDE);
fprintf('  drag       %s / %s\n', DRAG_MODEL, ATMOS);
fprintf('  srp / erp  %s / %s   (erp cap %dx%d = %d elements x %d facets/RHS)\n', SRP_MODEL, ERP_MODEL, ERP_RINGS, ERP_SEGS, ERP_RINGS*ERP_SEGS, numel(SC.facets));
fprintf('  gravity    %s deg %d   integrator %s\n', GRAV_FIELD, GRAV_DEG, INTEG.method);
sat.reads(cfg);

%% =============================== PROPAGATE ==================================
W   = op.buildWorld(cfg);
sol = op.propagate(cfg);
fprintf('\n[propagate] %d epochs over %.0f s\n', numel(sol.t), TSPAN_S);

%% ===================== FORCES, SEGREGATED, TO DISK ==========================
% Re-walk the solution and record every force separately. op.accel's 4th output
% hands back what the models ACTUALLY used, so nothing here re-implements the
% chain -- re-implementing to inspect is how drag.force and drag.panelCoeffs
% drifted apart.
t   = sol.t(:);
n   = numel(t);
names = {'gravity','thirdbody','drag','srp','erp','relativity'};
A   = struct(); for i=1:numel(names), A.(names{i}) = nan(n,3); end
aux = struct('rho',nan(n,1),'T',nan(n,1),'nO',nan(n,1),'Cd',nan(n,1), ...
             'A_proj',nan(n,1),'alt_km',nan(n,1),'yaw_deg',nan(n,1));
atot = nan(n,3);
for k = 1:n
    [aa, parts, ctx, info] = op.accel(t(k), sol.r(k,:).', sol.v(k,:).', W);
    atot(k,:) = aa.';
    for i=1:numel(names)
        if isfield(parts,names{i}), A.(names{i})(k,:) = parts.(names{i}).'; end
    end
    if isfield(info,'drag') && ~isempty(info.drag)
        D = info.drag;
        if ~isempty(D.atm)
            aux.rho(k) = D.atm.rho;
            if isfield(D.atm,'T'), aux.T(k) = D.atm.T; end
            if isfield(D.atm,'n') && isfield(D.atm.n,'O'), aux.nO(k) = D.atm.n.O; end
        end
        if ~isempty(D.geo), aux.alt_km(k) = D.geo.alt_km; end
        if ~isempty(D.out), aux.Cd(k) = D.out.Cd; end
    end
    R = subsref_default(ctx.sc,'R_bi',eye(3));
    aux.yaw_deg(k) = atan2d(R(2,1),R(1,1));
    if ~isempty(subsref_default(ctx.sc,'facets',[]))
        % Pass the Sun: drag.force pivots tracking arrays to it, so a diagnostic
        % that does not would quietly disagree with the force it is reporting.
        sun = [];
        if isfield(ctx,'E') && ~isempty(ctx.E) && isfield(ctx.E,'sun_eci'), sun = ctx.E.sun_eci; end
        aux.A_proj(k) = validation.projected_area(ctx.sc.facets, R, ctx.v_rel_eci, sun);
    end
end

if ~exist(SAVE_DIR,'dir'), mkdir(SAVE_DIR); end
for i = 1:numel(names)
    nm = names{i};
    if all(isnan(A.(nm)(:))), continue, end
    M = [t, A.(nm), vecnorm(A.(nm),2,2)];
    validation.write_csv(fullfile(SAVE_DIR,[nm '.csv']), {'t_s','ax','ay','az','a_mag'}, M);
end
validation.write_csv(fullfile(SAVE_DIR,'accel_total.csv'), {'t_s','ax','ay','az','a_mag'}, ...
          [t, atot, vecnorm(atot,2,2)]);
validation.write_csv(fullfile(SAVE_DIR,'state.csv'), {'t_s','x','y','z','vx','vy','vz'}, ...
          [t, sol.r, sol.v]);
validation.write_csv(fullfile(SAVE_DIR,'model_inputs.csv'), ...
          {'t_s','alt_km','rho','T_K','nO','Cd','A_proj_m2','yaw_deg'}, ...
          [t, aux.alt_km, aux.rho, aux.T, aux.nO, aux.Cd, aux.A_proj, aux.yaw_deg]);

fid = fopen(fullfile(SAVE_DIR,'manifest.txt'),'w');
fprintf(fid, 'TEMPLATE_16U   %s\n', datestr(now,31));
fprintf(fid, 'epoch      %s\n', sprintf('%d ',EPOCH));
fprintf(fid, 'alt/inc    %g km / %g deg   tspan %g s\n', ALT_KM, INC_DEG, TSPAN_S);
fprintf(fid, 'geometry   sgeom.sat16u  %d facets  mass %g kg\n', numel(SC.facets), SC.mass);
fprintf(fid, 'attitude   %s\n', ATTITUDE);
fprintf(fid, 'drag       %s / %s   gsi Tw=%g aT=%g\n', DRAG_MODEL, ATMOS, GSI.Tw, GSI.aT);
fprintf(fid, 'srp / erp  %s / %s\n', SRP_MODEL, ERP_MODEL);
fprintf(fid, 'gravity    %s deg %d\n', GRAV_FIELD, GRAV_DEG);
fprintf(fid, 'integrator %s rtol %g\n', INTEG.method, INTEG.rtol);
fprintf(fid, '\nFiles are per-force ON PURPOSE: "the drag is wrong" is answerable\n');
fprintf(fid, 'from drag.csv alone, and two runs diff force by force.\n');
fclose(fid);
fprintf('\n[saved] %s\n', SAVE_DIR);

%% ============================== THE LADDER ==================================
fprintf('\n---- force ladder (mean over the arc, as a ratio to gravity) ----\n');
g0 = mean(vecnorm(A.gravity,2,2));
fprintf('  %-12s | %-13s | %s\n','force','|a| mean','/gravity');
for i = 1:numel(names)
    if all(isnan(A.(names{i})(:))), continue, end
    m = mean(vecnorm(A.(names{i}),2,2));
    fprintf('  %-12s | %.6e | %.4e\n', names{i}, m, m/g0);
end
ng = mean(vecnorm(A.drag + A.srp + A.erp, 2, 2));
fprintf('  %-12s | %.6e | %.4e   <- what an accelerometer would see\n','NON-GRAV', ng, ng/g0);
fprintf('\n  Cd(t)  : %.4f .. %.4f   %s\n', min(aux.Cd), max(aux.Cd), ...
        subsref_tern(strcmpi(DRAG_MODEL,'cannonball'),'(cannonball: you typed it)','(DERIVED from the GSI physics)'));
fprintf('           ^ referenced to Aref = %.4f m^2 (the BUS ram face). A Cd of 5-10 is\n', SC.Aref);
fprintf('             not a broken model: Cd is DEFINED as D/(q*Aref), and this 16U carries\n');
fprintf('             ~%.3f m^2 of tracking array against a %.3f m^2 bus face. Normalise by\n', ...
        max(aux.A_proj)-SC.Aref, SC.Aref);
fprintf('             the small number and you get a big coefficient. Cd alone is meaningless\n');
fprintf('             without its reference area -- the physical quantity is Cd*A/m:\n');
CdAm = aux.Cd .* aux.A_proj / SC.mass;
fprintf('  Cd*A/m : %.4e .. %.4e m^2/kg   <- this is what the drag actually depends on\n', ...
        min(CdAm), max(CdAm));
fprintf('  A(t)   : %.4f .. %.4f m^2  (arrays tracking the Sun, so it MOVES)\n', ...
        min(aux.A_proj), max(aux.A_proj));
fprintf('  rho    : %.3e .. %.3e kg/m^3\n', min(aux.rho), max(aux.rho));

%% ============================== THE PICTURE =================================
if DO_IO_PLOT
    show_model_io(sol, W, sprintf('16U %d km  %s/%s  srp=%s erp=%s', ...
                  ALT_KM, DRAG_MODEL, ATMOS, SRP_MODEL, ERP_MODEL), SAVE_DIR, ...
                  struct('every_s', max(OUT_DT, 60)));
end

fprintf('\n================================================================\n');
fprintf(' NOTE: the 16U has geometry and NO measured truth. Nothing here is\n');
fprintf(' validated against a real satellite -- for that, use validate_OD on\n');
fprintf(' CHAMP/GRACE, which have truth and (so far) no dimensions.\n');
fprintf('================================================================\n');

% ============================================================================
%  NO LOCAL FUNCTIONS: MATLAB hoists a script's locals, Octave does not, so a
%  helper at the bottom is undefined when the script runs. That bug has bitten
%  this codebase repeatedly. writeCsv_ lives in +validation.
% ============================================================================
