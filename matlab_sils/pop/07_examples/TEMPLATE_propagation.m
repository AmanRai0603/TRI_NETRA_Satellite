%% ============================================================================
%  TEMPLATE_propagation.m  --  copy this for any propagation you want to run.
%  ----------------------------------------------------------------------------
%  Shows the full pattern:
%    (A) set epoch / state / spacecraft,
%    (B) choose the force stack (toggle + model per force),
%    (C) obtain SPACE WEATHER: try to auto-download it; if it is NOT available
%        for your epoch, the script tells you and falls back to MANUAL values
%        that YOU supply (no silent defaults),
%    (D) propagate and read out r,v at any time.
%  Everything downloaded (space weather, EOP, gravity, TLE) is cached under
%  data.root(), so later runs are offline.
%% ============================================================================
clear; setup_paths;

% (optional) put all downloads in a folder you choose:
% data.root('set', 'C:\orbit\data_cache');

% ---------------------------------------------------------------- (A) STATE ----
mu    = 3.986004418e14;
epoch = [2025 6 1 0 0 0];                 % UTC [Y Mo D H Mi S]
alt   = 400e3;  a = 6378137 + alt;  inc = deg2rad(51.6);
r0    = [a;0;0];
v0    = sqrt(mu/a)*[0;cos(inc);sin(inc)];
tspan = 86400;                            % seconds

cfg = config.defaultConfig();
cfg.epoch=epoch; cfg.r0=r0; cfg.v0=v0; cfg.tspan=tspan;
cfg.spacecraft.mass = 24;                 % kg
cfg.spacecraft.Aref = 0.04;               % m^2

% ------------------------------------------------------------- (B) FORCES ------
cfg.gravityField = struct('field','default','degree',12);   % or a .gfc path/name
cfg.forces.gravity   = struct('on',true,'model','sphharm','degree',12,'order',12);
cfg.forces.thirdbody = struct('on',true,'model','battin');
cfg.forces.srp       = struct('on',true,'model','cannonball','eclipse','conical','Cr',1.3);
cfg.forces.relativity= struct('on',true,'terms',{{'schwarzschild'}});
% choose a density model: 'exponential' (offline, no SW) OR a real one below
DENSITY_MODEL = 'nrlmsise';               % 'exponential'|'nrlmsise'|'jb2008'|'dtm2020'
cfg.forces.drag = struct('on',true,'model','dria','atmos',DENSITY_MODEL,'corotate',true);

% ------------------------------------------------------ (C) SPACE WEATHER ------
% Only real density models need space weather. Try to auto-fetch it for the
% propagation window; if that fails, tell the user and use MANUAL values.
if ~strcmpi(DENSITY_MODEL,'exponential')
    swAvailable = false;
    try
        d0 = datestr(datenum(epoch(1),epoch(2),epoch(3))-2,'yyyy-mm-dd');
        d1 = datestr(datenum(epoch(1),epoch(2),epoch(3))+ceil(tspan/86400)+2,'yyyy-mm-dd');
        SWtable = data.spaceweather(d0, d1);          % OMNI2 (F10.7) + GFZ (Kp/ap)
        % probe the epoch to confirm coverage
        atmos.spaceweather(epoch, struct('table',SWtable));
        swAvailable = true;
        fprintf('[SW] space weather downloaded & covers the epoch.\n');
    catch e
        fprintf(2,'[SW] space weather NOT available: %s\n', regexprep(e.message,'\n.*',''));
    end

    if ~swAvailable
        % -------- MANUAL SCENARIO: fill these in from any source you trust ------
        fprintf('[SW] using MANUAL space-weather values (edit them below).\n');
        cfg.spaceweather.manual = struct( ...
            'F107',  120, ...     % daily F10.7 [sfu]
            'F107a', 120, ...     % 81-day centred F10.7 [sfu]
            'Kp',    2.0, ...     % daily Kp
            'ap',    7);          % daily ap
    end
end

% ------------------------------------------------------- (D) PROPAGATE ---------
% For a precise inertial<->fixed frame with real EOP (auto-cached), use:
%   cfg.frame.build='B';  cfg.frame.data_dir = data.eop_dir();
cfg.integrator = struct('method','rk78','rtol',1e-10,'atol',1e-10);
cfg.output     = struct('dt', 300);

sol = op.propagate(cfg);
alt = (vecnorm(sol.r,2,2)-6378137)/1000;
fprintf('Propagated %.1f h: altitude %.2f -> %.2f km\n', tspan/3600, alt(1), alt(end));

% state at an arbitrary time after epoch:
rv = sol.stateAt(1234.5);
fprintf('r(1234.5 s) = [%.1f %.1f %.1f] km\n', rv(1:3)/1000);
