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
%  validate_OD.m  —  THE MASTER OD VALIDATION. ONE FILE. EVERY PARAMETER.
%  ============================================================================
%  ONE data source (ITSG / TU Graz), NO credentials, NO frame guessing.
%
%  WHAT THIS VALIDATES, and why each part exists:
%   (1) POSITION/VELOCITY vs reduced-dynamic orbit  -> the classic residual
%   (2) POSITION vs kinematic orbit                 -> an INDEPENDENT solution.
%         kinematic = GPS geometry only, no force model.
%         reduced-dynamic = geometry + a force model.
%         Their difference is the REFERENCE'S OWN uncertainty. If our residual is
%         smaller than that, we are chasing noise. This is the honesty check that
%         a single reference cannot give you.
%   (3) NON-CONSERVATIVE FORCE: measured accelerometer vs our model
%         The accelerometer measures drag+SRP+ERP DIRECTLY, in m/s^2, every 10 s.
%         Comparing it against our model tests Cd, the density model and SRP AT THE
%         SOURCE -- instead of inferring them from a position error 3 hours later.
%         This is the single most diagnostic plot in the file.
%   (4) DENSITY: measured neutral density vs our atmosphere model
%   (5) Optional overlays: TU Delft track (r only) and TLE (r,v ~km) on the SAME
%         epochs, purely for comparison. They are NOT the reference.
%
%  Everything the engine would decide silently is written below at its default.
%  Section 10 prints AND saves the full decision list + provenance of every input.
%
%  ---------------------------------------------------------------------------
%  SATELLITES — copy a NAME straight into SAT below. (`itsg.list` prints this live)
%  ---------------------------------------------------------------------------
%   NAME          ALT km  MASS   ACC  DENS  COVERAGE
%   CHAMP           450    522   yes  yes   2000-2010          <- best for OD: acc+density
%   GRACE-A         480    487   yes  yes   2002-2017          <- acc+density
%   GRACE-B         480    487   yes  yes   2002-2017          <- acc+density
%   GRACE-FO-1      490    600   yes  yes   2018-present       <- acc+density
%   GRACE-FO-2      490    600   yes  yes   2018-present       <- acc+density
%   SWARM-A         460    473   no   no    2013-present
%   SWARM-B         510    473   no   no    2013-present
%   SWARM-C         460    473   no   no    2013-present
%   TERRASAR-X      514   1230   no   yes   2007-present       (density from kinematic/POD)
%   TANDEM-X        514   1340   no   yes   2010-present       (density from kinematic/POD)
%   JASON-1        1336    500   no   no    2001-2013          (high alt: drag negligible)
%   JASON-2        1336    510   no   no    2008-2019
%   JASON-3        1336    510   no   no    2016-present
%   METOP-A         817   4085   no   no    2006-2021
%   METOP-B         817   4085   no   no    2012-present
%   SENTINEL-1A     693   2300   no   no    2014-present
%   SENTINEL-1B     693   2300   no   no    2016-2022
%   SENTINEL-1C     693   2300   no   no    2024-present
%   SENTINEL-2A     786   1140   no   no    2015-present
%   SENTINEL-2B     786   1140   no   no    2017-present
%   SENTINEL-2C     786   1140   no   no    2024-present
%   SENTINEL-3A     814   1150   no   no    2016-present
%   SENTINEL-3B     814   1150   no   no    2018-present
%   SENTINEL-6A    1336   1440   no   no    2020-present
%
%   ALL of them give: reducedDynamicOrbit (r,v INERTIAL, 10 s) + kinematicOrbit +
%   covariance + attitude.  ACC=yes also gives the accelerometer product, which is
%   what makes a DIRECT drag/SRP validation possible -> prefer CHAMP/GRACE/GRACE-FO.
%   GOCE is NOT on ITSG (ESA only, manual download).
%% ============================================================================
setup_paths;

%% ---------------------------------------------------------------- 1. CASE ----
SAT      = 'CHAMP';            % itsg.list for all names + coverage
DATE     = '2007-01-01';       % UTC day (must be inside that satellite's coverage)
TSPAN_S  = 3*3600;             % arc length [s]
OUT_DT_S = 30;                 % comparison spacing [s]

%% ------------------------------------------------------------ 2. WHAT TO RUN --
DO_RDO      = true;            % (1) compare vs reducedDynamicOrbit  (the seed source)
DO_KIN      = true;            % (2) compare vs kinematicOrbit       (independent check)
DO_ACC      = true;            % (3) measured vs modelled non-conservative force
DO_DENSITY  = true;            % (4) measured vs modelled density
DO_TUDELFT  = false;           % (5) overlay TU Delft track (position only)
DO_TLE      = false;           % (5) overlay TLE-propagated position (~km, mean elements)
FORCE_REFETCH = false;         % ITSG products are immutable -> cached forever
DO_TUDELFT_DENSITY = true;     % (5) ALSO compare density to TU Delft measured
                               %     (INDEPENDENT truth; distinct from DO_TUDELFT,
                               %      which is the position overlay)
DENSITY_PRODUCT = 'auto';      % 'auto' | 'neutralDensity' | 'neutralDensity_ACC'
% 'auto' = neutralDensity_ACC for CHAMP (that is the only density release it has),
% neutralDensity for everyone else. Override only to force a specific release.

%% -------------------------------------------------------------- 3. SEEDING ----
SEED_PRODUCT = 'reducedDynamicOrbit';   % r,v both, CELESTIAL frame, 10 s
SEED_INDEX   = 1;                       % which epoch to seed from (1 = first)
% No velocity is derived and no frame is rotated on this path: the file already
% gives v in the inertial frame. That is why ITSG replaces GFZ/ESA here.

%% ===========================================================================
%  DATA PROVENANCE -- WHAT IS REAL, WHAT IS FETCHED, WHAT YOU ARE ASSUMING
%
%  Four kinds of number go into the answer below. Once they are floats in a struct
%  they are indistinguishable, so they are declared here, at the top, where you set
%  them. validation.provenance(cfg,W,REF,SAT) recomputes this LIVE from the config
%  you actually ran and show_provenance plots it -- this block is the map, that is
%  the territory. If they disagree, believe the runtime one and fix this comment.
%
%  ---- MEASURED: a real instrument, THIS satellite, THIS epoch -----------------
%    reducedDynamicOrbit   ITSG (TU Graz) . . . . . . . . . metric [1] reference
%    kinematicOrbit        ITSG . . . . . . . . . . . . . . metric [2] reference
%    nonConservativeForces ITSG accelerometer . . . . . . . metric [3] reference
%    attitude (quaternions)ITSG . . . . . . . . . . . . . . drives A(t) and Cd(t)
%    neutralDensity        ITSG . . . . . . . . . . . . . . metric [4] reference
%    TU Delft density      TU Delft/Aerospace . . . . . . . metric [5] reference
%      -> WHICH of these exist depends on the satellite. CHAMP and the GRACE family
%         have all six; Swarm has only TU Delft; Sentinel/Jason have none. The
%         catalog columns has_acc/has_density/has_tudelft say so and
%         validation.capability(SAT) turns that into which models may run.
%
%  ---- FETCHED: someone else's measurement, from an open source ----------------
%    F10.7, F10.7a      OMNI2 (NASA GSFC) via data.spaceweather
%    Kp, ap             GFZ Potsdam (also mirrored in OMNI2)
%    ap60 / Hpo         GFZ Hpo product          (DTM2020 *research* only)
%    F30, F30_bar       CLS / LISIRD             (DTM2020 *research* only)
%    F10/S10/M10/Y10    Space Environment Technologies SOLFSMY   (JB2008 only)
%    DSTDTC             SET DTCFILE                              (JB2008 only)
%    EOP xp,yp,dUT1     IERS (data.eop_dir)
%    Sun/Moon/Earth     JPL DE440 (bundled)
%    gravity field      ICGEM (auto-cached), or gravity_data/ if you drop a .gfc
%      -> Real data, but NOT of this satellite. A 3-hour planetary Kp is standing in
%         for the local state above one spacecraft. That gap is physics, not a bug,
%         and it is part of why metric [4] will never read exactly 1.000.
%
%  ---- ASSUMED: we chose it because nobody published one. EVERY ONE IS A KNOB. --
%    Cd            catalog 3.0 for CHAMP -- a LITERATURE value, not a measurement.
%                  Only read when DRAG_MODEL='cannonball'; the panel models DERIVE
%                  Cd(t) instead. Degenerate with density on metrics [1]/[2].
%    Cr            catalog 1.3 -- an assumption for nearly every satellite here.
%                  Only read when SRP_MODEL='cannonball'.
%    GSI.Tw        wall temperature. Nobody measured this satellite's surface.
%    GSI.aT        accommodation. TYPED by 'sentman'; DERIVED by 'dria'/'sesam'
%                  from the local atomic oxygen -- so with those it is NOT a knob.
%    GSI.sig_n/t   CLL accommodation. Chosen.
%    OPTICS        facet alpha/rho_s/rho_d. Chosen -- and ONE set is applied to
%                  BOTH the visible and thermal-IR bands, which is wrong in kind,
%                  not just in value (see 09_docs/CODEBASE_AUDIT.md).
%    GEOMETRY      'plate'    -> A(t)=Aref*|n.vhat|. Aref IS the ram area, so this
%                                assumes NO shape. Good for CHAMP/GRACE (ram/nadir,
%                                small yaw). Useless for SRP.
%                  'aref_box' -> an ASSUMED box: ram face == Aref by construction,
%                                every other face is BOX_ASPECT. Measured cost on
%                                CHAMP: srp moves 2.2x, erp 4.4x across plausible
%                                shapes. A sensitivity band, never the answer.
%                  'boxwing'  -> real dimensions from the catalog. NO satellite in
%                                this catalog has them (has_geometry=no for all 24).
%    lag_f107      same-day (GOCE-validated) vs t-24h (spec). A CONVENTION.
%    aph_mode      flat vs the real 57-hour history. A CONVENTION.
%    ATTITUDE='ram'  MODELLED, not the real attitude. 'measured' uses the real one.
%
%  ---- THE RULE ---------------------------------------------------------------
%  A residual explained by an ASSUMED input is not a finding about the atmosphere.
%  It is a finding about your assumption. Sweep it in compare_OD before believing
%  it: 11_compare/verify_sweep_matrix.m shows which knobs each model actually reads
%  (a flat sweep can mean "not consulted", which is correct, or "dropped", which is
%  a bug -- and they look identical until you test).
%% ===========================================================================
%% ===========================================================================
%  WHICH MODELS CAN THIS SATELLITE ACTUALLY RUN?
%  (verified by 06_validation/verify_model_matrix.m -- it RUNS each combination,
%   it does not just consult a table that agrees with itself)
%
%   satellite    | drag                            | srp              | erp
%                | cann sent dria cll  sesam       | cann boxw        | knoc simp cere boxw
%   -------------+---------------------------------+------------------+--------------------
%   CHAMP        |  y    y    y    y    y          |  y    y*         |  y    y    y    y*
%   GRACE-A/B    |  y    y    y    y    y          |  y    y*         |  y    y    y    y*
%   GRACE-FO-1/2 |  y    y    y    y    y          |  y    y*         |  y    y    y    y*
%   SWARM-A/B/C  |  y    .    .    .    .          |  y    .          |  y    y    y    .
%   TERRASAR-X   |  y    .    .    .    .          |  y    .          |  y    y    y    .
%   TANDEM-X     |  y    .    .    .    .          |  y    .          |  y    y    y    .
%   JASON-1/2/3  |  y    .    .    .    .          |  y    .          |  y    y    y    .
%   METOP-A/B    |  y    .    .    .    .          |  y    .          |  y    y    y    .
%   SENTINEL-*   |  y    .    .    .    .          |  y    .          |  y    y    y    .
%
%     y  = runs.   . = REFUSED: the input data does not exist for that satellite.
%     y* = runs ONLY with GEOMETRY='aref_box', i.e. on an ASSUMED shape. No satellite
%          in this catalog has measured dimensions (has_geometry=no for all 24).
%
%  WHY THE PATTERN IS WHAT IT IS
%    Only CHAMP and the GRACE family carry an accelerometer, and ITSG publishes the
%    attitude WITH it -- the ACC product is useless without one. Attitude is what a
%    panel drag model acts on. Everywhere else R_bi would stay eye(3), which puts the
%    body axes on the INERTIAL axes: a +x plate then faces inertial +x instead of the
%    flow and the drag collapses toward ZERO -- silently, with no error. So those
%    cells are refused rather than run. A '.' is a DATA limit, not a code limit.
%
%  THE COST OF 'aref_box' -- measured on CHAMP, same ram face, same drag:
%    aspect [1 1 1]   -> srp 1.28e-08   erp 3.11e-09
%    aspect [4.6 1 1] -> srp 2.55e-08   erp 1.25e-08      (~2x srp, ~4x erp)
%    That spread IS the assumption. Use it as a sensitivity band, never as the answer.
%
%  For DRAG you never need a box: GEOMETRY='plate' gives A(t)=Aref*|n.vhat| straight
%  from the measured attitude with no invented shape, and CHAMP/GRACE fly ram/nadir
%  with small yaw, so a plate is a GOOD drag approximation. It is a poor SRP one --
%  a plate edge-on to the Sun has no lit area and a real satellite does. That, and
%  only that, is what the box buys.
%
%  METRICS, same rule:
%    [3] vs nonConservativeForces  needs has_acc      -> CHAMP/GRACE only
%    [4] vs neutralDensity         needs has_density  -> CHAMP/GRACE/TerraSAR/TanDEM
%    [5] vs TU Delft density       needs has_tudelft  -> + Swarm
%  validation.capability(SAT) prints the live version of all of this, and this script
%  REFUSES an unsupported model before propagating rather than after.
%% ===========================================================================

%% ---------------------------------------------------------- 4. SPACECRAFT ----
USE_CATALOG = true;            % mass/area/Cd/Cr from itsg_catalog.csv
SC_MASS_KG  = [];              % [] = catalog (CHAMP 522)
SC_AREA_M2  = [];              % [] = catalog (CHAMP 0.767)
SC_CD       = [];              % [] = catalog (CHAMP 3.0)
SC_CR       = [];              % [] = catalog (1.3)

% ---- AREA & DRAG COEFFICIENT: constant, or driven by the MEASURED attitude ----
% This is the toggle that decides whether A and Cd are numbers or functions of time.
%
%   ATTITUDE = 'none'      R_bi = eye(3) forever. A(t) = Aref, Cd(t) = Cd. The
%                          cannonball assumption. Fast, and wrong whenever the
%                          satellite is not a sphere.
%   ATTITUDE = 'measured'  R_bi(t) from ITSG's OWN attitude product -- the same
%                          quaternions metric [3] already uses to rotate the
%                          accelerometer. The satellite turns, the projected area
%                          follows, and with a GSI model below Cd(t) comes out of
%                          the physics instead of being typed in.
%
% Only meaningful with DRAG_MODEL ~= 'cannonball': the cannonball branch reads
% sc.Aref and sc.Cd directly and never looks at R_bi.
%   ATTITUDE = 'ram'       body +x along v_rel (dgeom.ramAttitude). The MODELLED
%                          attitude: A(t) is constant (the plate always faces the
%                          flow) but Cd(t) still falls out of the GSI physics.
%                          Use this when there is no attitude product.
%
%   WARNING on 'none' + a panel model: R_bi = eye(3) means body axes == INERTIAL
%   axes, so a +x plate faces inertial +x rather than the flow, and the drag is
%   whatever the geometry accidentally gives -- often ~ZERO. 'none' is only
%   meaningful with DRAG_MODEL='cannonball', which never reads R_bi.
ATTITUDE = 'measured';         % 'none' | 'ram' | 'measured'

%   DRAG_MODEL = 'cannonball'  -0.5*rho*Cd*(Aref/m)*|v|*v. Cd is an INPUT.
%              = 'sentman'     free-molecular, diffuse re-emission w/ accommodation
%              = 'dria'        Sentman + Langmuir adsorption (the low LEO workhorse)
%              = 'cll'         Cercignani-Lampis-Lord (separate normal/tangential)
%              = 'sesam'       ALIAS of 'dria' -- SESAM is an ACCOMMODATION model,
%                              not a panel model. It supplies aT; DRIA is Sentman
%                              WITH that aT. Kept because people ask for it by name.
%
% The real distinction is 'sentman' (you TYPE aT, and everyone types 0.9) versus
% 'dria'/'sesam' (aT computed from the local atomic-oxygen density and temperature).
% At low LEO that IS the physics: adsorbed O drives accommodation and it is rarely 0.9.
% The last four integrate over FACETS, so they return A(t) and Cd(t) together --
% you cannot ask them for "a Cd", which is the point. GSI parameters below.
DRAG_MODEL = 'cannonball';     % start here; switch to 'dria' to test the ratio
ATMOS      = 'dtm2020';        % exponential | nrlmsise | dtm2020 | dtm2020_research | jb2008
                               % The atmosphere used to live ONLY inside the FORCES
                               % struct as ATMOS, while the model sat up
                               % here as DRAG_MODEL -- one decision in the knob block,
                               % its partner 60 lines away in a struct literal. Both
                               % belong together, where you choose them.
GSI = struct('Tw',300, ...     % wall temperature [K]
             'aT',0.9, ...     % thermal accommodation
             'sig_n',0.9,'sig_t',0.9);   % CLL normal/tangential accommodation

%   GEOMETRY = 'plate'    one flat plate of area Aref facing +x(body). With
%                         ATTITUDE='measured' this ALONE gives A(t) = Aref*|n.v|,
%                         which is the honest minimum: it needs no invented shape.
%            = 'boxwing'  dgeom.buildBox + dgeom.addArray. Needs REAL dimensions;
%                         only set this for a satellite whose geometry you have.
%                         Inventing a shape to make a number move is not modelling.
%            = 'aref_box' AN ASSUMED box, built from Aref and an aspect ratio you
%                         choose. NOT the satellite's geometry -- an ASSUMPTION,
%                         made deliberately and recorded as such.
%
%   ---- ON ASSUMING A BOX -------------------------------------------------------
%   You have the attitude, so you CAN wrap a box around Aref and run box-wing SRP
%   and ERP. That is a legitimate engineering choice and this option exists to let
%   you make it. What it is not is CHAMP's geometry. Measured, for Aref=0.767:
%
%       yaw    | plate    | cube     | 4m box
%         0 deg| 0.767000 | 0.767000 | 0.767000
%        45 deg| 0.542351 | 1.084702 | 3.019446     <- 6x apart
%        90 deg| 0.000000 | 0.767000 | 3.503141
%
%   All three reproduce Aref at the one angle where Aref was defined, and diverge
%   by 6x elsewhere. Aref is ONE number and a box needs THREE: the inversion is not
%   unique, so BOX_ASPECT below is you supplying the missing information.
%
%   Use it for SENSITIVITY -- "how much could SRP move if the shape is roughly
%   this?" -- and not as CHAMP's answer. The run report and provenance tag every
%   aref_box result as ASSUMED so a number cannot escape this file looking measured.
%
%   For DRAG none of this is needed: 'plate' already gives A(t)=Aref*|n.vhat| from
%   the attitude, with no invented shape, and CHAMP/GRACE fly ram/nadir with small
%   yaw so a plate is a good drag approximation. It is a poor SRP approximation
%   (a plate edge-on to the Sun has no lit area; a real satellite does), which is
%   the whole reason to consider a box here.
GEOMETRY   = 'plate';          % 'plate' | 'boxwing' | 'aref_box'
BOX_DIMS   = [];               % [Lx Ly Lz] m for 'boxwing'; [] = take from catalog
BOX_ASPECT = [1 1 1];          % for 'aref_box': [ax ay az] shape, scaled so the
                               % +x (ram) face area == Aref. [1 1 1] = a cube.
                               % CHAMP is ~4 m long on a ~0.9 m face: [4.6 1 1] is
                               % closer to its real proportions than a cube, and
                               % still an assumption.

% ---- OPTICS: the SAME facets feed drag AND SRP ------------------------------
% There are two facet formats in this tree and they collided on one field name:
%   dgeom.buildBox -> .n .A                                (drag needs only these)
%   srp.buildBox   -> .type .n .A .alpha .rho_s .rho_d .axis .double
% Feeding drag facets to srp.boxwing dies with "structure has no member 'type'";
% feeding SRP facets to drag.force works fine, because drag reads only .n and .A.
% So the SRP format is a strict superset and is the one to build: ONE facet set,
% both forces take what they need, and the geometry can never disagree with itself.
% That matters because A(t) must be the SAME area for drag and SRP -- two facet
% sets would be two spacecraft.
OPTICS = struct('alpha',0.2, ...   % absorbed
                'rho_s',0.3, ...   % specularly reflected
                'rho_d',0.5);      % diffusely reflected   (must sum to 1)

%% --------------------------------------------------------------- 5. FORCES ---
% ---- ONE DECISION, ONE PLACE ------------------------------------------------
% This struct used to carry 'model' for drag/srp/erp AS WELL AS the DRAG_MODEL /
% SRP_MODEL / ERP_MODEL knobs above -- and then line ~540 did:
%     cfg.forces.drag.model = DRAG_MODEL;
%     cfg.forces.srp.model  = SRP_MODEL;
%     cfg.forces.erp.model  = ERP_MODEL;
% so whatever you set HERE was silently overwritten. Set FORCES.drag.model='dria'
% and leave DRAG_MODEL='sentman' and you ran SENTMAN, with no warning, and every
% conclusion you drew was about the wrong model.
%
% That is the same "one quantity, two names" bug this audit has been removing from
% the physics all along (.area/.Aref, drag.force vs panelCoeffs, two 16U geometries)
% -- except sitting in the USER-FACING KNOBS, which is the worst place for it,
% because the knobs are the part people actually edit.
%
% So: 'model' is GONE from here. FORCES carries on/off and the settings that have
% no dedicated knob. The model for drag/srp/erp comes from DRAG_MODEL / SRP_MODEL /
% ERP_MODEL above, and nowhere else. Same for the atmosphere (ATMOS) and the GSI.
% ---- WHAT IS ON, AND WHY --------------------------------------------------
% The notes live ABOVE the struct, not inside it. A line-continued struct literal
% with four-line comment blocks between its elements parses -- and is one stray
% keystroke from not parsing, because every line of the continuation has to be
% right. A comma that becomes a semicolon somewhere in the middle produces
% "parse error" pointing at a line that looks fine. Explanation belongs next to
% the code, not inside the expression.
%
%   erp        ON.  ~0.09 m/3h at 460 km. Small, but free -- and "off by default
%              in a struct literal" was never a decision anyone made on purpose.
%   solidtides OFF. ~0.6 m/3h -- the biggest single term still off. Left off until
%              the ephemeris + Love-number path is exercised on real MATLAB.
%              validate_OD PRICES it (validation.force_budget), so the choice is
%              visible rather than silent.
%   oceantides OFF. ~0.1 m/3h. Same argument, smaller.
%
% NOTE: solidtides/oceantides are GRAVITATIONAL. They move the ORBIT (metric [1])
% and are INVISIBLE to the accelerometer (metric [3]) -- an accelerometer in free
% fall cannot feel gravity. Turning them on to fix a metric [3] ratio does nothing.
% Verified: switching both on changed the non-grav sum by 0.0000e+00.
%
% The model for drag/srp/erp comes from DRAG_MODEL / SRP_MODEL / ERP_MODEL above.
% Do NOT add a 'model' field here -- validate_OD errors if you do, because the knob
% would silently win and your setting here would do nothing.
FORCES = struct( ...
  'gravity',    struct('on',true, 'model','sphharm','degree',70,'order',70), ...
  'drag',       struct('on',true, 'corotate',true), ...
  'thirdbody',  struct('on',true, 'model','battin'), ...
  'srp',        struct('on',true), ...
  'relativity', struct('on',true, 'terms',{{'schwarzschild'}}), ...
  'erp',        struct('on',true), ...
  'solidtides', struct('on',false), ...
  'oceantides', struct('on',false));

GRAV_FIELD = 'EGM2008';        % 'default' silently caps at degree 6 -- report flags it
SW_MANUAL  = [];               % [] = measured F10.7/ap; else struct('F107',..,'ap',..)

%% ----------------------------------------------------------- 6. INTEGRATOR ---
INTEG_METHOD = 'rk78';         % rk4|nystrom4|rk6luther|gaussJackson8|rk45|rk78|ode45|ode78|ode89|ode113
INTEG = struct('rtol',1e-11,'atol',1e-6);
% A correct setup gives the SAME residual on every integrator (~2 cm apart). If it
% does not, that is a step-size/tolerance problem, not physics.

%% -------------------------------------------------------------- 7. FRAME ----
SRP_MODEL = 'cannonball';      % 'cannonball' | 'boxwing'  (boxwing needs GEOMETRY facets
                               %  + ATTITUDE='measured'/'ram' to mean anything)
ERP_MODEL = 'knocke';          % 'knocke' | 'simple' | 'ceres' | 'boxwing'
% knocke/simple/ceres are CANNONBALL: they collapse the spacecraft to CrAoM = Cr*A/m
% and CANNOT see attitude. 'boxwing' applies each Earth element's beam to the facets,
% so A and the optical response follow the attitude.
%
% Measured at 350 km on a 4.0x1.6x0.75 m bus, ram-pointing:
%   knocke  1.790e-09 (flat at every yaw -- it has no way to vary)
%   boxwing 1.064e-08 at yaw 0, 9.362e-09 at 45 deg, 7.442e-09 at 90 deg
% The 6x is mostly area: the box has ~21 m^2 of facets vs Aref 0.767. Which is the
% point -- CrAoM*Aref is not the area the Earth actually sees.
%
% COST: erp 'boxwing' evaluates nrings*nseg*numel(facets) facet responses per RHS
% call (768*6 ~ 4.6k at the defaults). knocke already pays the 768; this pays 6x.

FRAME = 'C';                   % only used for the OPTIONAL TU Delft / TLE overlays
% The ITSG path needs NO frame build: its r,v are already celestial/inertial.

%% -------------------------------------------------------------- 8. OUTPUT ----
SAVE  = true;
PLOTS = true;

%% ------------------------------------------------------------------ 9. RUN ---
K = de440.constants(); mu = K.mu_earth; Re = K.Re_earth;   % not a retyped 6378137
op_getf = @(s,f,d) subsref_default(s,f,d);
tern_   = @(c,a,b) subsref_tern(c,a,b);   % no script-local functions: Octave never defines them
opts = struct(); if FORCE_REFETCH, opts.force = true; end

%% ---------------------------------------------------------------------------
%  THE ORDER BELOW IS THE POINT. Read it top to bottom:
%
%    9a  EPOCH + SATELLITE      what are we looking at
%    9b  SATELLITE DATA         everything the satellite gives us FOR THAT EPOCH,
%                               fetched UP FRONT: the seed r,v AND every product we
%                               will later compare against. If a product is missing
%                               you find out NOW -- not after paying for a full arc.
%    9c  SATELLITE PROPERTIES   mass/area/Cd/Cr from the catalog, or typed in here
%                               for a satellite the catalog does not know
%    9d  DRIVER DATA + FORCES   op.buildWorld resolves space weather / EOP / gravity
%                               / ephemeris FOR THAT EPOCH, then builds the world
%    9e  PROPAGATE              by this point everything is decided; this is just
%                               integration
%    9f  COMPARE                our state vs the data fetched in 9b
%
%  This is the same order compare_OD uses. Data first, world second, propagation
%  third, comparison last. Nothing is fetched from inside the integrator.
%% ---------------------------------------------------------------------------

% ---- 9a. THE PLAN: print every decision BEFORE acting on any of it ----------
% Nothing in this script runs without saying so first. If a number below is not
% what you meant, stop here -- not after the arc.
fprintf('\n================================================================\n');
fprintf(' validate_OD   %s   %s   %.2f h arc\n', SAT, DATE, TSPAN_S/3600);
fprintf('================================================================\n');
fprintf('  seed          %s[%d]  (the file''s own r,v: no derivation, no rotation)\n', SEED_PRODUCT, SEED_INDEX);
fprintf('  compare       RDO=%d KIN=%d ACC=%d DEN=%d TUDelft=%d  |  overlays TUD=%d TLE=%d\n', ...
        DO_RDO, DO_KIN, DO_ACC, DO_DENSITY, DO_TUDELFT_DENSITY, DO_TUDELFT, DO_TLE);
fprintf('  gravity       %s %s deg %d order %d   (field ''default'' SILENTLY caps at 6)\n', ...
        GRAV_FIELD, FORCES.gravity.model, FORCES.gravity.degree, FORCES.gravity.order);
fprintf('  drag          %s / %s   corotate=%d\n', DRAG_MODEL, ATMOS, FORCES.drag.corotate);
fprintf('  srp / erp     %s / %s\n', SRP_MODEL, ERP_MODEL);
fprintf('  attitude      %s   geometry %s   (only cannonball ignores both)\n', ATTITUDE, GEOMETRY);
fprintf('  other forces  thirdbody=%d srp=%d erp=%d relativity=%d solid=%d ocean=%d\n', ...
        FORCES.thirdbody.on, FORCES.srp.on, FORCES.erp.on, FORCES.relativity.on, ...
        FORCES.solidtides.on, FORCES.oceantides.on);
if isempty(SW_MANUAL)
    fprintf('  space weather MEASURED for the epoch (needs data_sources + network)\n');
else
    fprintf('  space weather MANUAL F107=%g F107a=%g ap=%g  <- NOT measured; you typed this\n', ...
            op_getf(SW_MANUAL,'F107',NaN), op_getf(SW_MANUAL,'F107a',NaN), op_getf(SW_MANUAL,'ap',NaN));
end
fprintf('  frame         %s   integrator %s (rtol=%g)   output every %g s\n', ...
        FRAME, INTEG_METHOD, op_getf(INTEG,'rtol',NaN), OUT_DT_S);
fprintf('  spacecraft    %s\n', tern_(USE_CATALOG,'from itsg_catalog.csv','TYPED IN BELOW (catalog off)'));
fprintf('  cache         refetch=%d\n', FORCE_REFETCH);
fprintf('----------------------------------------------------------------\n');

% ---- 9b. EPOCH + SATELLITE DATA (the seed and every reference) ----
RDO = data.itsg(SAT, DATE, SEED_PRODUCT, opts);
mjd0 = RDO.mjd(SEED_INDEX);
t_ref = (RDO.mjd - mjd0)*86400;
sel = t_ref >= 0 & t_ref <= TSPAN_S;
t_ref = t_ref(sel); r_ref = RDO.r(sel,:); v_ref = RDO.v(sel,:);
r0 = r_ref(1,:).'; v0 = v_ref(1,:).';                 % <-- the seed: file's own r,v
epoch = validation.mjd2utc(mjd0);
fprintf('\n[seed] %s %s from %s: |r0|=%.3f km |v0|=%.4f km/s (no derivation, no rotation)\n', ...
        SAT, DATE, SEED_PRODUCT, norm(r0)/1000, norm(v0)/1000);

% ---- 9b (cont). every OTHER satellite product for this epoch, fetched NOW ----
% All of it up front, before any physics: a missing product is a fact about the
% data, and you should learn it in seconds rather than after a propagation.
R = struct(); prov = {RDO.source};
REF = struct('RDO', RDO);
SC  = validation.itsg_catalog(SAT);

if DO_KIN
    try, REF.KIN = data.itsg(SAT, DATE, 'kinematicOrbit', opts); prov{end+1} = REF.KIN.source;
    catch ME, fprintf('[2] kinematicOrbit unavailable: %s\n', regexprep(ME.message,'\n.*','')); end
end
if DO_ACC && strcmpi(SC.has_acc,'yes')
    try
        REF.ACC = data.itsg(SAT, DATE, 'nonConservativeForces', opts); prov{end+1} = REF.ACC.source;
        REF.ATT = data.itsg(SAT, DATE, 'attitude', opts);              prov{end+1} = REF.ATT.source;
    catch ME, fprintf('[3] accelerometer/attitude unavailable: %s\n', regexprep(ME.message,'\n.*','')); end
elseif DO_ACC
    fprintf('[3] %s has no accelerometer product on ITSG -- skipped (see itsg.list).\n', SAT);
end
if DO_DENSITY && strcmpi(SC.has_density,'yes')
    % PRODUCT NAME. This used to be hardcoded to 'neutralDensity' for every
    % satellite. ITSG ships CHAMP's density as neutralDensity_ACC (release 1.0,
    % MJD+rho only); 'neutralDensity' does not exist for CHAMP, so the fetch 404'd,
    % a catch swallowed it, and the density comparison never ran on the one
    % satellite this file's own header recommends.
    switch lower(DENSITY_PRODUCT)
        case 'auto'
            % ---- THE PRODUCT NAME IS DATA, NOT A GUESS ------------------------
            % itsg_catalog.csv column 16 carries the REAL directory name:
            %     .../operational/<sat>/neutralDensity_1.0/
            % Versioned, and the same on every satellite. NO amount of guessing was
            % ever going to produce "_1.0" -- which is exactly why the old code tried
            % two spellings, 404'd, and announced "not published for this
            % satellite/epoch": a confident claim about the world manufactured from a
            % failure of our own spelling.
            %
            % There is NO fallback guess list here any more. A guess that only works
            % because a human already looked the name up is not a fallback -- it is
            % the lookup with extra steps, and it lets a wrong name survive by
            % producing plausible 404s instead of an error. If the catalog is wrong,
            % the error below says so and names the column to fix.
            dprod = {};
            if isfield(SC,'density_product') && ~isempty(SC.density_product)
                dprod = {SC.density_product};
            end
            % The server listing is a CHECK on the catalog, not a substitute for it.
            % If they disagree we want to know LOUDLY rather than silently prefer one.
            try
                avail_ = data.itsg_products(SC.dir);
                if ~isempty(avail_)
                    hit_ = avail_(~cellfun(@isempty, regexpi(avail_, 'density')));
                    if isempty(dprod) && ~isempty(hit_)
                        dprod = hit_;
                        fprintf('[4] catalog has no density_product; server offers: %s\n', ...
                                strjoin(hit_, ', '));
                    elseif ~isempty(dprod) && ~isempty(hit_) && ~any(strcmp(dprod{1}, hit_))
                        fprintf(['[4] WARNING: catalog says ''%s'' but the server lists %s.\n' ...
                                 '    Using the SERVER. Fix itsg_catalog.csv column 16.\n'], ...
                                dprod{1}, strjoin(hit_, ', '));
                        dprod = hit_;
                    end
                end
            catch
                % Offline: the catalog name stands. It was READ OFF the server, so it
                % is a record of what is there, not an assumption about what might be.
            end
            if isempty(dprod)
                error('validate_OD:noDensityProduct', ...
                     ['no density product for %s: itsg_catalog.csv column 16 is empty and ' ...
                      'the server could not be listed.\nThis is a CATALOG gap, not a missing ' ...
                      'dataset -- has_density=%s for this satellite.'], SAT, SC.has_density);
            end
        otherwise, dprod = DENSITY_PRODUCT;
    end
    if ischar(dprod), dprod = {dprod}; end
    den_err = '';
    for dk_ = 1:numel(dprod)
        try
            REF.DEN = data.itsg(SAT, DATE, dprod{dk_}, opts);
            prov{end+1} = REF.DEN.source;
            DENSITY_PRODUCT_USED = dprod{dk_};
            fprintf('[4] using ITSG product ''%s''\n', dprod{dk_});
            break
        catch e_
            den_err = e_.message;
            if dk_ < numel(dprod)
                fprintf('[4] %s not found -- trying ''%s''\n', dprod{dk_}, dprod{dk_+1});
            end
        end
    end
    % strjoin, not a bare dprod: `dprod` was a CHAR until round 41 made it a CELL of
    % candidate names, and this consumer was not updated -- so fprintf died with
    % "Function is not defined for 'cell' inputs" AFTER every download had succeeded.
    % My own "one name, two types" bug class, added three rounds after I wrote the
    % check for it. The static checker cannot see this one: the type only differs at
    % runtime, inside a format string.
    try, if ~isfield(REF,'DEN'), error('validate_OD:noDen','%s', den_err); end
    catch ME
        fprintf('[4] %s unavailable: %s\n', strjoin(dprod, ' / '), ...
                regexprep(ME.message,'\n.*',''));
    end
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
% ---- what is TU Delft actually giving us, and is it OURS? --------------------
% TU Delft's product is the same KIND of thing as ITSG's: a retrieval from the
% satellite's own accelerometer, on that satellite's own track. So it is only a
% comparison if it is the SAME SATELLITE and the SAME EPOCH. It is not a model to
% be evaluated anywhere -- asking it for a different satellite, or a date it does
% not cover, does not "degrade" the comparison, it voids it. GRACE-B silently
% receiving GRACE-A's file (fixed in round 7) is exactly what that looks like when
% nobody checks.
% So: check, and if it does not line up say DATA NOT AVAILABLE and move on.
if DO_TUDELFT_DENSITY && SC.has_tudelft
    try
        % DATE,DATE is a ZERO-WIDTH window: as datenums that is one instant
        % (midnight), so the fetcher returned 0 rows (CHAMP: "no rows in
        % [01-Jan-2007,01-Jan-2007]") or 1 row by luck (GRACE: a sample exactly at
        % 00:00:00) -- and metric [5] then reported a "ratio" computed from a SINGLE
        % point. Ask for the arc we actually propagate, rounded up to whole days.
        td_end = datestr(datenum(DATE) + max(1, ceil(TSPAN_S/86400)), 'yyyy-mm-dd');
        Ttd = data.tudelft_density(SC.tudelft_name, DATE, td_end);
        REF.TUD = validation.tudelft_to_ref(Ttd);
        % epoch overlap with the ARC we propagate -- not merely "the file exists"
        mjd0_ = datenum(DATE) - 678942;
        nIn   = sum(REF.TUD.mjd >= mjd0_ & REF.TUD.mjd <= mjd0_ + TSPAN_S/86400);
        fprintf('[5] TU Delft %s: %d samples, %d inside our %.1f h arc.\n', ...
                SC.tudelft_name, numel(REF.TUD.mjd), nIn, TSPAN_S/3600);
        if nIn < 2
            fprintf(['[5] DATA NOT AVAILABLE: TU Delft has %d sample(s) in the arc. ' ...
                     'A ratio from <2 points is a number, not a measurement -- skipping.\n'], nIn);
            REF = rmfield(REF,'TUD');
        end
        % 'retrieved' is NOT optional: the provenance writer below prints p.retrieved
        % for every entry. Omitting it made this line crash the whole script AFTER a
        % successful 3 h propagation -- and only for satellites where the TU Delft
        % fetch SUCCEEDED, which is why CHAMP (fetch failed -> no entry) looked fine
        % and GRACE did not. Every prov entry must carry the same fields.
        prov{end+1} = struct('product','TU Delft density','date',DATE, ...
                             'provider','TU Delft thermosphere.tudelft.nl','frame','geodetic', ...
                             'nEpochs',numel(REF.TUD.mjd),'retrieved',datestr(now,31), ...
                             'url','https://thermosphere.tudelft.nl/data/');
    catch ME
        fprintf('[5] TU Delft density unavailable: %s\n', regexprep(ME.message,'\n.*',''));
    end
elseif DO_TUDELFT_DENSITY
    fprintf('[5] %s has no TU Delft density (itsg_catalog.csv has_tudelft=no) -- skipped.\n', SAT);
end

if ~DO_RDO, REF = rmfield(REF,'RDO'); end


if ~DO_RDO, REF = rmfield(REF,'RDO'); end
fprintf('[data] fetched for %s %s: %s\n', SAT, DATE, strjoin(fieldnames(REF).', ', '));

% ---- 9c. CONFIG: satellite properties + force selection ----
cfg = config.defaultConfig();
cfg.epoch = epoch; cfg.r0 = r0; cfg.v0 = v0; cfg.tspan = TSPAN_S;
cfg.forces = FORCES;
cfg.gravityField = struct('field',GRAV_FIELD,'degree',FORCES.gravity.degree);
cfg.frame = struct('build',FRAME);
if USE_CATALOG
    % No translation: the catalog now speaks the canonical names (.mass/.Aref), the
    % same ones the forces read. This line used to be
    %     cfg.spacecraft.Aref = SC.area;
    % i.e. a rename in flight -- which works right up until someone writes a new
    % script and does not know it is needed. That is how the Cd bug happened.
    cfg.spacecraft.mass = SC.mass; cfg.spacecraft.Aref = SC.Aref;
    cfg.spacecraft.Cd = SC.Cd;     cfg.spacecraft.Cr  = SC.Cr;
end
if ~isempty(SC_MASS_KG), cfg.spacecraft.mass = SC_MASS_KG; end
if ~isempty(SC_AREA_M2), cfg.spacecraft.Aref = SC_AREA_M2; end
if ~isempty(SC_CD),      cfg.spacecraft.Cd   = SC_CD;      end
if ~isempty(SC_CR),      cfg.spacecraft.Cr   = SC_CR;      end

% ---- A(t) and Cd(t): hand the forces the MEASURED attitude + the geometry ------
% Until now the attitude product was fetched, used to rotate the accelerometer for
% metric [3], and then dropped. The forces never saw it, so every "attitude
% dependent" model ran on eye(3) -- a frozen attitude and therefore a frozen area.
% The model knobs are the SINGLE source of truth. If someone re-adds 'model' to the
% FORCES struct above, say so loudly rather than silently overwriting it -- a silent
% overwrite is how you spend a day concluding things about a model you never ran.
for fk_ = {'drag','srp','erp'}
    if isfield(FORCES, fk_{1}) && isfield(FORCES.(fk_{1}), 'model')
        error('validate_OD:twoSourcesOfTruth', ...
          ['FORCES.%s.model is set to ''%s'', but the model comes from the %s_MODEL ' ...
           'knob (currently ''%s''). Two places for one decision: the knob would ' ...
           'silently win and your FORCES setting would do nothing. Remove ' ...
           'FORCES.%s.model and use %s_MODEL.'], fk_{1}, FORCES.(fk_{1}).model, ...
           upper(fk_{1}), eval([upper(fk_{1}) '_MODEL']), fk_{1}, upper(fk_{1}));
    end
end
cfg.forces.drag.model = DRAG_MODEL;
cfg.forces.drag.atmos = ATMOS;
cfg.forces.drag.gsi   = GSI;
cfg.forces.srp.model  = SRP_MODEL;
cfg.forces.erp.model  = ERP_MODEL;

switch lower(ATTITUDE)
    case 'measured'
        if isfield(REF,'ATT') && ~isempty(REF.ATT)
            cfg.spacecraft.attitude = struct('mjd',REF.ATT.mjd, 'q',REF.ATT.q);
            fprintf('[attitude] A(t) and Cd(t) driven by the MEASURED ITSG attitude.\n');
        else
            % Do not quietly fall back to eye(3) and let the run LOOK attitude-driven.
            warning('validate_OD:noAttitude', ...
              ['ATTITUDE=''measured'' but no attitude product was fetched for %s %s. ' ...
               'Falling back to a FIXED attitude: A(t)=Aref and Cd(t)=Cd, i.e. a ' ...
               'cannonball in all but name.'], SAT, DATE);
            cfg.spacecraft.attitude = [];
        end
    case 'ram'
        cfg.spacecraft.attitude = 'ram';
    otherwise
        cfg.spacecraft.attitude = [];
        if ~strcmpi(DRAG_MODEL,'cannonball')
            error('validate_OD:attitudeTrap', ...
              ['ATTITUDE=''none'' with DRAG_MODEL=''%s'' is not a valid combination. ' ...
               'R_bi = eye(3) puts the body axes on the INERTIAL axes, so the +x plate ' ...
               'faces inertial +x instead of the flow and the drag collapses to ~zero ' ...
               'at most epochs -- it would look like a physics result and be an artefact. ' ...
               'Use ATTITUDE=''ram'' (modelled) or ''measured'' (real), or ' ...
               'DRAG_MODEL=''cannonball'' (which never reads R_bi).'], DRAG_MODEL);
        end
end

% ---- facets: only build a shape we actually have -------------------------------
switch lower(GEOMETRY)
    case 'aref_box'
        % Build a box whose RAM FACE area is exactly Aref, with the shape you chose.
        % Ly*Lz == Aref (the +x face), and Lx follows from the aspect ratio.
        ar = BOX_ASPECT(:).'/BOX_ASPECT(2);           % normalise on y
        Ly = sqrt(cfg.spacecraft.Aref * ar(2)/ar(3));
        Lz = cfg.spacecraft.Aref / Ly;
        Lx = Ly * ar(1)/ar(2);
        cfg.spacecraft.facets = srp.buildBox(Lx, Ly, Lz, OPTICS);
        GEOM_NOTE = sprintf(['ASSUMED box [%.3f %.3f %.3f] m from Aref=%.3f + aspect ' ...
                             '[%g %g %g] -- NOT %s''s geometry'], Lx, Ly, Lz, ...
                             cfg.spacecraft.Aref, BOX_ASPECT, SAT);
        fprintf('[geometry] %s\n', GEOM_NOTE);
        fprintf('           ram face %.4f m^2 == Aref by construction; every other\n', Ly*Lz);
        fprintf('           face is your assumption. Results are tagged ASSUMED.\n');
    case 'boxwing'
        if isempty(BOX_DIMS)
            error('validate_OD:noDims', ...
              ['GEOMETRY=''boxwing'' needs BOX_DIMS = [Lx Ly Lz]. There is no box-wing ' ...
               'geometry for %s in this toolbox, and inventing dimensions to make the ' ...
               'area move is not modelling -- it is fitting with extra steps. Use ' ...
               'GEOMETRY=''plate'' (A(t) = Aref*|n.v|, needs no invented shape) or supply ' ...
               'the real dimensions.'], SAT);
        end
        % srp.buildBox, NOT dgeom.buildBox: the SRP format carries .n/.A (all drag
        % needs) PLUS the optics SRP needs. One set, one geometry, both forces.
        cfg.spacecraft.facets = srp.buildBox(BOX_DIMS(1), BOX_DIMS(2), BOX_DIMS(3), OPTICS);
        fprintf('[geometry] box-wing: %d facets from [%g %g %g] m -- drives drag A(t) AND SRP\n', ...
                numel(cfg.spacecraft.facets), BOX_DIMS);
    otherwise
        % One plate of area Aref facing +x(body), in the SRP format so SRP can use
        % it too. With a measured attitude this is already A(t) = Aref*|n.v| -- the
        % honest minimum, no invented shape.
        cfg.spacecraft.facets = srp.facet('body', [1;0;0], cfg.spacecraft.Aref, ...
                                          OPTICS.alpha, OPTICS.rho_s, OPTICS.rho_d, [0;0;0], false);
        fprintf('[geometry] single plate A=%.3f m^2 (+x body) -- drives drag A(t) AND SRP\n', ...
                cfg.spacecraft.Aref);
end
if abs(OPTICS.alpha + OPTICS.rho_s + OPTICS.rho_d - 1) > 1e-9
    error('validate_OD:optics','OPTICS alpha+rho_s+rho_d must sum to 1 (got %.6f)', ...
          OPTICS.alpha+OPTICS.rho_s+OPTICS.rho_d);
end
cfg.forces.srp.Cr = cfg.spacecraft.Cr;
if ~isempty(SW_MANUAL), cfg.spaceweather.manual = SW_MANUAL; end
cfg.output = struct('times', (0:OUT_DT_S:TSPAN_S).');
cfg.integrator = INTEG; cfg.integrator.method = INTEG_METHOD;


% ---- 9d. DRIVER DATA + THE WORLD (space weather / EOP / gravity / ephemeris) --
% op.buildWorld is the ONLY place driver data is fetched, and it keys everything to
% cfg.epoch + cfg.tspan. Nothing below it touches the disk or the network.
W = op.buildWorld(cfg);
D = W.drivers;
fprintf('[drivers] atmos=%s window=%s..%s | swtable=%d swmanual=%d jbidx=%d | frame=%s\n', ...
        D.atmos, D.window{1}, D.window{2}, D.swtable, D.swmanual, D.jbidx, D.frame);
fprintf('[drivers] gravity=%s deg<=%d | DE440=%d | eop=%s\n', ...
        op_getf(W.grav,'name','default'), op_getf(W.grav,'nmax',0), ~isempty(W.eph), D.eop_dir);

% ---- 9e. PROPAGATE (everything is decided by now; this is just integration) ---
sol = op.propagate(cfg);
fprintf('[propagate] %s, rtol=%g: %d output epochs over %.2f h\n', ...
        INTEG_METHOD, op_getf(INTEG,'rtol',NaN), numel(sol.t), TSPAN_S/3600);

% ---- 9f. COMPARE our state against the data fetched in 9b --------------------
R = validation.od_metrics(sol, W, REF, struct( ...
        'mjd0',mjd0, 'tspan_s',TSPAN_S, 'epoch',epoch, ...
        'frame',FRAME, 'atmos',ATMOS, 'verbose',true));
if isfield(R,'refSpread')
    fprintf('    -> our residual is only meaningful ABOVE this floor.\n');
end
if isfield(R,'den') && isfield(R.den,'geoFromProduct') && ~R.den.geoFromProduct
    fprintf('    NOTE r1.0 product: sample point taken from OUR state, so our position error leaks in.\n');
end

% ---- 9g. (5) INDEPENDENT PRODUCTS, on the SAME epochs -------------------------
% These are NOT the reference. They are separate measurements of the same truth,
% plotted against the ITSG reduced-dynamic orbit so their quality is visible on one
% axis. TU Delft gives POSITION ONLY (and its own density); a TLE gives r,v but from
% MEAN elements (~1 km by construction). Both are interpolated onto OUR output
% epochs so nothing is compared at mismatched times.
R.ext = struct();
if DO_TUDELFT
    try
        TD = validation.track_reference(SAT, DATE, datestr(datenum(DATE)+1,'yyyy-mm-dd'), ...
                                        struct('frame',FRAME,'thin',1));
        tt = TD.t - (mjd0 - (datenum(DATE)-678942))*86400;   % align to our t=0
        st = tt >= 0 & tt <= TSPAN_S;
        if sum(st) > 5
            % position-only product: no v to Hermite with. spline, never pchip.
            rt = interp1(tt(st), TD.r(st,:), sol.t, 'spline', NaN);
            good = all(~isnan(rt),2);
            rr2 = validation.interp_state(t_ref, r_ref, v_ref, sol.t);
            d = rt(good,:) - rr2(good,:);
            R.ext.TUDelft = struct('t',sol.t(good),'d3',sqrt(sum(d.^2,2)));
            R.tudelft = struct('pos3D', sqrt(mean(sum(d.^2,2))));
            fprintf('[5] TU Delft track vs ITSG reduced-dynamic: %.2f m RMS (position only)\n', ...
                    R.tudelft.pos3D);
        end
    catch ME
        fprintf('[5] TU Delft overlay skipped: %s\n', regexprep(ME.message,'\n.*',''));
    end
end
if DO_TLE
    try
        % NORAD id comes from itsg_catalog.csv (the old sat_catalog is gone).
        sats = data.tle(SC.norad, DATE, datestr(datenum(DATE)+1,'yyyy-mm-dd'));
        tle  = validation.parseTLE(sats(1).l1, sats(1).l2, SAT);
        [rT,vT] = validation.tle2eci(tle, mu, epoch);
        cT = cfg; cT.r0 = rT; cT.v0 = vT;                  % same physics, TLE seed
        solT = op.propagate(cT);
        rr2 = validation.interp_state(t_ref, r_ref, v_ref, solT.t);
        d = solT.r - rr2;
        R.ext.TLE = struct('t',solT.t,'d3',sqrt(sum(d.^2,2)));
        R.tle = struct('pos3D', sqrt(mean(sum(d.^2,2))));
        fprintf('[5] TLE-seeded run vs ITSG reduced-dynamic: %.1f m RMS (mean elements ~km)\n', ...
                R.tle.pos3D);
    catch ME
        fprintf('[5] TLE overlay skipped: %s\n', regexprep(ME.message,'\n.*',''));
    end
end
if isempty(fieldnames(R.ext)), R = rmfield(R,'ext'); end

%% ---------------------------------------------------- 10. DECISIONS + PROVENANCE
outdir = '';
if SAVE, outdir = save_results(sprintf('OD_%s_%s',SAT,DATE)); end
if isempty(outdir), config.report(cfg, W, sol);
else,               config.report(cfg, W, sol, fullfile(outdir,'decisions.txt')); end

fprintf('\n---- PROVENANCE: every input, tagged with its source ----\n');
for i=1:numel(prov)
    p = prov{i};
    fprintf('  %-24s %-10s  %s\n', p.product, p.date, p.provider);
    fprintf('  %-24s frame=%s  epochs=%d\n', '', p.frame, p.nEpochs);
    fprintf('  %-24s %s\n', '', p.url);
end
% Same W.sw mistake as the density block, in its quiet form: guarded by isfield,
% so it never threw -- it just meant the provenance list SILENTLY omitted the space
% weather line on every run ever made. The indices live in W.swmanual / W.swtable.
if ~isempty(op_getf(W,'swmanual',[]))
    m = W.swmanual;
    fprintf('  %-24s F10.7=%g F10.7a=%g ap=%g   (MANUAL -- set in section 5, not measured)\n', ...
            'space weather', op_getf(m,'F107',NaN), op_getf(m,'F107a',NaN), op_getf(m,'ap',NaN));
elseif ~isempty(op_getf(W,'swtable',[]))
    T = W.swtable;
    src = 'table';
    if isfield(T,'source') && ~isempty(T.source), src = T.source; end
    fprintf('  %-24s measured F10.7/ap, source=%s  (see decisions.txt)\n','space weather', src);
else
    fprintf('  %-24s none loaded (atmos=%s needs no indices)\n','space weather', ATMOS);
end
if ~isempty(outdir)
    save(fullfile(outdir,'OD.mat'),'R','cfg','sol','prov','-v7');
    fid = fopen(fullfile(outdir,'provenance.txt'),'w');
    for i=1:numel(prov)
        p = prov{i};
        fprintf(fid,'%s | %s | %s | frame=%s | epochs=%d | retrieved=%s\n%s\n\n', ...
                p.provider, p.product, p.date, p.frame, p.nEpochs, p.retrieved, p.url);
    end
    fclose(fid);
    fprintf('  saved: %s\n', outdir);
end

%% ------------------------------------------- 11. PROVENANCE + PLOTS ----
% PROVENANCE IS NOT A PLOT. It prints whether or not PLOTS is on, because it is
% the thing that tells you what KIND of number you just read -- and a run with
% plots disabled (a batch sweep, a headless box) is exactly where a silently
% ASSUMED input does the most damage. Nesting it under `if PLOTS` would have
% dropped it precisely when it matters most.
%
% One provenance, not two: the fetched-product ledger built above knows WHERE each
% download came from (provider, frame, epochs, URL); the classifier knows WHAT KIND
% every input is, including the ASSUMED ones (Cd, Cr, GSI, optics, geometry) that
% are never downloaded and so never appear in the ledger. The ledger is passed in
% rather than re-derived -- two overlapping accounts of one question is the
% drag.force/panelCoeffs pattern, and they agree until someone edits one.
PROV = validation.provenance(cfg, W, REF, SAT, prov);
fprintf('\n  provenance: measured %d | fetched %d | ASSUMED %d | MISSING %d\n', ...
        PROV.n.measured, PROV.n.fetched, PROV.n.assumed, PROV.n.missing);
if PROV.n.assumed > PROV.n.measured
    fprintf(['  [!] more inputs are ASSUMED than MEASURED. Not automatically wrong --\n' ...
             '      it is the normal state for a satellite with no published geometry --\n' ...
             '      but the headline number is then a statement about your assumptions\n' ...
             '      as much as about the physics. Sweep them in compare_OD.\n']);
end

if PLOTS
    % ---- A FIGURE MUST NOT KILL THE RUN THAT PRODUCED IT ---------------------
    % Twice now a plotting bug has destroyed a completed validation: `DO_PLOTS`
    % (a name I invented) and `P = R.rdo.dv_rtn` (which clobbered the options
    % struct so `P.growth` died 18 lines later). Both fired AFTER a 20 s
    % propagation and every download. In each case the SCIENCE WAS FINE and a
    % diagnostic threw it away.
    %
    % So the plot calls are guarded and the errors are REPORTED, not swallowed:
    % file, line and message, then carry on. This is the opposite of the bare
    % `catch, end` this audit has been removing -- the test is whether you find
    % out, and here you do, loudly, while keeping the results.
    %
    % R and PROV are already computed and printed above; the figures are a view of
    % them, not the answer. If a view is broken, fix the view -- do not lose the run.
    plotFailed = {};
    try
        show_provenance(PROV, R, sprintf('%s %s', SAT, DATE), outdir);
    catch err_
        plotFailed{end+1} = sprintf('show_provenance: %s', err_.message);
        if ~isempty(err_.stack)
            plotFailed{end} = sprintf('%s  [%s line %d]', plotFailed{end}, ...
                                      err_.stack(1).name, err_.stack(1).line);
        end
    end
    try
        show_OD(R, sol, sprintf('%s  %s', SAT, DATE), outdir);
    catch err_
        plotFailed{end+1} = sprintf('show_OD: %s', err_.message);
        if ~isempty(err_.stack)
            plotFailed{end} = sprintf('%s  [%s line %d]', plotFailed{end}, ...
                                      err_.stack(1).name, err_.stack(1).line);
        end
    end
    if ~isempty(plotFailed)
        fprintf('\n  [!] %d FIGURE(S) FAILED -- the validation itself is unaffected:\n', ...
                numel(plotFailed));
        for q_ = 1:numel(plotFailed), fprintf('      %s\n', plotFailed{q_}); end
        fprintf('      The metrics above and %s/*.csv are complete and correct.\n', outdir);
        fprintf('      Fix the plot; do not re-run the propagation.\n');
    end
end

% ============================================================================
%  No local functions. They now live in 06_validation/+validation/ :
%     validation.rtn_stats     validation.mjd2utc
%     validation.quat2dcm      validation.itsg_catalog
%  and doy_ is retired in favour of the engine's own timeconv.doy.
%
%  WHY: MATLAB hoists a script's local functions; Octave does not -- it defines
%  them only when execution reaches them, so helpers at the end of a script are
%  never defined and every call fails with "'mjd2utc' undefined". Lifting them
%  makes this script run identically on both, and makes each helper testable and
%  reusable on its own.
% ============================================================================
