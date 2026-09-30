function cfg = defaultConfig()
%CONFIG.DEFAULTCONFIG  Baseline configuration for op.propagate.
%   cfg = config.defaultConfig()
%   A sensible LEO default: spherical-harmonic gravity (degree 6 zonal
%   default field), Sun+Moon third body, cannonball drag on the exponential
%   atmosphere, cannonball SRP with conical eclipse, RK78 adaptive integrator,
%   offline GMST frame.  Edit fields, or start from config.forcePresets(name).
%
%   Every force has an .on toggle and a .model selector so you can flip terms and
%   swap models to study their impact (see examples/ex04_force_impact_study.m).

    % ---- epoch & initial ECI state (edit these) ----
    cfg.epoch = [2025 6 1 0 0 0];              % UTC [Y Mo D H Mi S]
    cfg.r0 = [6778137.0; 0; 0];                % m  (~400 km circular, equatorial)
    cfg.v0 = [0; 7668.6; 0];                   % m/s
    cfg.tspan = 6000;                          % s  (duration) or [t0 tf]

    % ---- spacecraft (16U-class low LEO default) ----
    cfg.spacecraft = struct( ...
        'mass', 24.0, ...                      % kg
        'Aref', 0.04, ...                      % m^2 frontal (0.2 x 0.2)
        'Cd',   2.2, ...                       % cannonball drag coeff
        'Cr',   1.3, ...                       % cannonball SRP coeff
        'R_bi', eye(3));                       % body->ECI attitude DCM (ram-fixed)

    % ---- gravity field ----
    cfg.gravityField = struct('field','default','degree',6);

    % ---- frame (offline default; switch build to 'B'/'C' for precise EOP) ----
    cfg.frame = struct('build','gmst','dUT1',0.0);

    % ---- integrator ----
    cfg.integrator = struct('method','rk78','rtol',1e-10,'atol',1e-12);

    % ---- output sampling ----
    cfg.output = struct('dt',30);              % report every 30 s

    % ---- space weather (only used by real density models: nrlmsise/jb2008/dtm2020)
    % source 'celestrak' auto-fetches + caches F10.7/Kp/ap; set .manual to bypass
    % the fetch and supply your own indices (see docs/SPACEWEATHER.md).
    cfg.spaceweather = struct('source','auto','manual',[]);  % auto: historical(OMNI2+GFZ) or forecast(NOAA) by epoch

    % ---- force model stack ----
    cfg.forces = struct();
    cfg.forces.gravity    = struct('on',true,'model','sphharm','degree',6,'order',6);
    cfg.forces.thirdbody  = struct('on',true,'model','battin');
    % NOTE: no 'Cd' here, deliberately. forces.drag.Cd is an OVERRIDE, and drag.m
    % resolves Cd as forces.drag.Cd -> spacecraft.Cd -> 2.2. Pre-seeding the
    % override with 2.2 meant the override was ALWAYS present, so the
    % spacecraft.Cd fallback could never fire for anyone who built a config the
    % natural incremental way:
    %       cfg = config.defaultConfig(); cfg.spacecraft.Cd = 3.0;
    % That silently propagated CHAMP at 2.2 instead of 3.0 -- a 27% drag error --
    % while config.report happily printed "Cd : 3", because report reads
    % spacecraft and the physics read forces.drag. It only LOOKED fixed because
    % the three master scripts (EXAMPLE_16U, compare_OD, validate_OD) replace
    % cfg.forces wholesale with a FORCES struct that has no Cd, so the fallback
    % fired for them. Every other path was still wrong. The default value lives in
    % drag.m's fallback, which is the one place that should own it.
    cfg.forces.drag       = struct('on',true,'model','cannonball','atmos','exponential', ...
                                   'corotate',true);
    cfg.forces.srp        = struct('on',true,'model','cannonball','eclipse','conical','Cr',1.3);
    cfg.forces.erp        = struct('on',false,'model','knocke');
    cfg.forces.relativity = struct('on',false,'terms',{{'schwarzschild'}});
    cfg.forces.solidtides = struct('on',false);
    cfg.forces.oceantides = struct('on',false);
    cfg.forces.empirical  = struct('on',false,'acc',[0 0 0]);
end
