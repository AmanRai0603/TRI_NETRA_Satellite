% RUN_ALL_TESTS  The regression suite. ONE file. Run after any change.
%
%   RUN:  setup_paths; run_all_tests
%
%   ---------------------------------------------------------------------------
%   WHAT CHANGED AND WHY
%   ---------------------------------------------------------------------------
%   This was five files: run_all_tests + test_gravity + test_integrators +
%   test_energy + test_seed, each a script calling the next. Consolidated because
%   the split bought nothing (nobody ran them individually) and cost the usual
%   things: five places for setup_paths to be missed, five files to keep in sync,
%   and a suite whose coverage you could only learn by opening all five.
%
%   The physics checks below are unchanged. Added: the drag chain and the force
%   chains, which are where every real bug in this codebase actually lived --
%   the old suite checked gravity and integrators, which were never wrong.
%
%   Each block is INDEPENDENT: comment one out and the rest still run.
%% ============================================================================
setup_paths;
fprintf('\n==================== REGRESSION SUITE ====================\n');
nfail = 0;

%% ---- 1. GRAVITY: analytic J2 + gradient consistency ------------------------
fprintf('\n[1] gravity\n');
try
    % Build ctx through op.accel rather than by hand: a hand-made ctx is a SECOND
    % definition of the interface and drifts from the real one (this test first
    % failed with "no member 'r_eci'" for exactly that reason).
    K = de440.constants(); a = K.Re_earth+500e3;
    r = [a;0;0]; v = [0;sqrt(K.mu_earth/a);0];
    cfg = config.defaultConfig();
    cfg.epoch=[2007 1 1 0 0 0]; cfg.r0=r; cfg.v0=v; cfg.tspan=60;
    cfg.output=struct('times',[0;60]);
    cfg.gravityField=struct('field','default','degree',2);
    cfg.forces=struct('gravity',struct('on',true,'model','twobody'), ...
      'drag',struct('on',false),'thirdbody',struct('on',false),'srp',struct('on',false), ...
      'erp',struct('on',false),'relativity',struct('on',false), ...
      'solidtides',struct('on',false),'oceantides',struct('on',false));
    W = op.buildWorld(cfg);
    [~, parts] = op.accel(0, r, v, W);
    % Compare against the GM the FIELD carries, not de440's. They differ at ~1e-9
    % relative -- DE440's mu_earth and a gravity model's GM come from different
    % fits and are not the same number. Testing against de440's gives a 7.5e-10
    % "failure" that is really a units-of-truth mismatch, not a code error. This is
    % the same class as Cd-without-its-reference-area: a number is meaningless
    % without saying which convention produced it.
    mu_field = subsref_default(W.grav,'mu',K.mu_earth);
    a_an = -mu_field*r/norm(r)^3;
    e = norm(parts.gravity - a_an)/norm(a_an);
    fprintf('    two-body vs analytic (field GM): rel %.3e   %s\n', e, subsref_tern(e<1e-12,'PASS','FAIL'));
    fprintf('    field GM vs de440 mu_earth: rel %.3e (different sources, not an error)\n', ...
            abs(mu_field-K.mu_earth)/K.mu_earth);
    nfail = nfail + (e>=1e-12);
catch err
    fprintf('    ERROR: %s\n', err.message); nfail = nfail+1;
end

%% ---- 2. INTEGRATORS: Kepler return + method invariance ---------------------
fprintf('\n[2] integrators (a correct setup must not depend on the method)\n');
try
    K = de440.constants(); a = K.Re_earth+450e3; Tp = 2*pi*sqrt(a^3/K.mu_earth);
    cfg = config.defaultConfig();
    cfg.epoch=[2007 1 1 0 0 0]; cfg.r0=[a;0;0]; cfg.v0=[0;sqrt(K.mu_earth/a);0];
    cfg.tspan=Tp; cfg.output=struct('times',[0;Tp]);
    cfg.gravityField=struct('field','default','degree',2);
    cfg.forces=struct('gravity',struct('on',true,'model','twobody'), ...
      'drag',struct('on',false),'thirdbody',struct('on',false),'srp',struct('on',false), ...
      'erp',struct('on',false),'relativity',struct('on',false), ...
      'solidtides',struct('on',false),'oceantides',struct('on',false));
    ref = [];
    for m = {'rk78','rk45','ode45'}
        c = cfg; c.integrator = struct('method',m{1},'rtol',1e-12,'atol',1e-9);
        s = op.propagate(c);
        clo = norm(s.r(end,:).' - cfg.r0);
        if isempty(ref), ref = s.r(end,:); d = 0; else, d = norm(s.r(end,:)-ref); end
        fprintf('    %-6s closure %.3e m | vs rk78 %.3e m   %s\n', m{1}, clo, d, ...
                subsref_tern(clo<5,'PASS','FAIL'));
        nfail = nfail + (clo>=5);
    end
catch err
    fprintf('    ERROR: %s\n', err.message); nfail = nfail+1;
end

%% ---- 3. ENERGY: two-body drift ---------------------------------------------
fprintf('\n[3] energy conservation\n');
try
    K = de440.constants(); a = K.Re_earth+450e3; Tp = 2*pi*sqrt(a^3/K.mu_earth);
    cfg = config.defaultConfig();
    cfg.epoch=[2007 1 1 0 0 0]; cfg.r0=[a;0;0]; cfg.v0=[0;sqrt(K.mu_earth/a);0];
    cfg.tspan=5*Tp; cfg.output=struct('times',[0;5*Tp]);
    cfg.integrator=struct('method','rk78','rtol',1e-13,'atol',1e-8);
    cfg.gravityField=struct('field','default','degree',2);
    cfg.forces=struct('gravity',struct('on',true,'model','twobody'), ...
      'drag',struct('on',false),'thirdbody',struct('on',false),'srp',struct('on',false), ...
      'erp',struct('on',false),'relativity',struct('on',false), ...
      'solidtides',struct('on',false),'oceantides',struct('on',false));
    s = op.propagate(cfg);
    E0 = 0.5*norm(cfg.v0)^2 - K.mu_earth/norm(cfg.r0);
    E1 = 0.5*norm(s.v(end,:))^2 - K.mu_earth/norm(s.r(end,:));
    d = abs((E1-E0)/E0);
    fprintf('    |dE/E| over 5 orbits: %.3e   %s\n', d, subsref_tern(d<1e-10,'PASS','FAIL'));
    nfail = nfail + (d>=1e-10);
catch err
    fprintf('    ERROR: %s\n', err.message); nfail = nfail+1;
end

%% ---- 4. INTERPOLANT: exact on a cubic, and O(h^4) --------------------------
% Was not tested, and a pchip interpolant was silently costing 72 m in validate_OD.
fprintf('\n[4] validation.interp_state\n');
try
    t=[0;1;2;3]; f=@(x) 2*x.^3-5*x.^2+3*x+7; df=@(x) 6*x.^2-10*x+3;
    [rq,vq] = validation.interp_state(t,[f(t) 0*t 0*t],[df(t) 0*t 0*t],linspace(0,3,51).');
    e = max(abs(rq(:,1)-f(linspace(0,3,51).')));
    fprintf('    exact on a cubic: %.3e   %s\n', e, subsref_tern(e<1e-10,'PASS','FAIL'));
    nfail = nfail + (e>=1e-10);
catch err
    fprintf('    ERROR: %s\n', err.message); nfail = nfail+1;
end

%% ---- 5. THE DRAG CHAIN: rho, Cd, A(t), m ----------------------------------
% Every real bug lived here, not in gravity. verify_drag_chain is the detailed
% version; this is the pass/fail gate.
fprintf('\n[5] drag chain (density closure + A(t) + panel<->cannonball)\n');
try
    NA = 6.02214076e26;
    geo = validation.probe_geo(350,'2007-01-01');
    sw  = atmos.spaceweather(geo.utc, struct('manual',struct('F107',90,'F107a',90,'ap',8,'Kp',2)));
    atm = atmos.provider('dtm2020', geo, sw);
    nm = fieldnames(atm.n); tot = 0;
    for j=1:numel(nm), tot = tot + atm.n.(nm{j})*drag.species(nm{j})/NA; end
    rr = atm.rho/tot;
    fprintf('    density closure sum(n*M/NA)/rho: %.4f   %s   (1e6 here = the cm^-3 bug)\n', ...
            rr, subsref_tern(abs(rr-1)<1e-3,'PASS','FAIL'));
    nfail = nfail + (abs(rr-1)>=1e-3);

    K = de440.constants();
    f1 = srp.facet('body',[1;0;0],1.0,0.2,0.3,0.5,[0;0;0],false);
    R  = dgeom.ramAttitude([0;7700;0]);
    A  = validation.projected_area(f1, R, [0;1;0]);
    fprintf('    A(t) of a ram plate: %.6f (expect 1.0)   %s\n', A, subsref_tern(abs(A-1)<1e-12,'PASS','FAIL'));
    nfail = nfail + (abs(A-1)>=1e-12);

    o = struct('mass',522,'Aref',1.0,'R_bi',R,'wind',[0;0;0],'omega',0);
    out = drag.force([K.Re_earth+350e3;0;0],[0;7700;0],atm,f1,'sentman', ...
                     struct('Tw',300,'aT',0.9,'sig_n',0.9,'sig_t',0.9),o);
    rho_used = 2*out.qd/7700^2;
    a_can = 0.5*rho_used*out.Cd*(1.0/522)*7700^2;
    rel = abs(norm(out.a)-a_can)/a_can;
    fprintf('    cannonball(Cd from panel) == panel: rel %.3e   %s\n', rel, ...
            subsref_tern(rel<1e-9,'PASS','FAIL'));
    nfail = nfail + (rel>=1e-9);
catch err
    fprintf('    ERROR: %s\n', err.message); nfail = nfail+1;
end

%% ---- 6. EVERY FORCE FIRES, FINITE, AND ATTITUDE REACHES THE PANEL MODELS ---
fprintf('\n[6] all forces finite + attitude actually reaches them\n');
try
    K = de440.constants(); a = K.Re_earth+300e3;
    r = [a;0;0]; v = [0;sqrt(K.mu_earth/a);0];
    S = sgeom.vleo16u();
    cfg = config.defaultConfig();
    cfg.epoch=[2007 1 1 0 0 0]; cfg.r0=r; cfg.v0=v; cfg.tspan=60;
    cfg.output=struct('times',[0;60]);
    cfg.gravityField=struct('field','default','degree',6);
    cfg.spaceweather.manual=struct('F107',90,'F107a',90,'ap',8,'Kp',2);
    cfg.spacecraft=struct('mass',S.mass,'Aref',0.04,'Cd',2.2,'Cr',1.3, ...
                          'facets',S.facets,'attitude','ram','R_bi',eye(3));
    cfg.forces=struct('gravity',struct('on',true,'model','sphharm','degree',6,'order',6), ...
      'drag',struct('on',true,'model','dria','atmos','dtm2020','corotate',true, ...
                    'gsi',struct('Tw',300,'aT',0.9,'sig_n',0.9,'sig_t',0.9)), ...
      'thirdbody',struct('on',true,'model','battin'), ...
      'srp',struct('on',true,'model','boxwing'), ...
      'erp',struct('on',true,'model','boxwing','nrings',4,'nseg',8), ...
      'relativity',struct('on',true,'terms',{{'schwarzschild'}}), ...
      'solidtides',struct('on',false),'oceantides',struct('on',false));
    W = op.buildWorld(cfg);
    [~, parts] = op.accel(0, r, v, W);
    fn = fieldnames(parts); ok = true;
    for k=1:numel(fn)
        f = all(isfinite(parts.(fn{k})));
        if ~f, fprintf('    %s NOT FINITE\n', fn{k}); ok = false; end
    end
    fprintf('    %d forces, all finite: %s   (NaN here = the solar-array n=[0;0;0] bug)\n', ...
            numel(fn), subsref_tern(ok,'PASS','FAIL'));
    nfail = nfail + ~ok;

    % attitude MUST change the panel drag, or R_bi is not reaching the force
    c2 = cfg; c2.spacecraft.attitude = [];
    c2.forces.srp.model='cannonball'; c2.forces.erp.model='knocke';
    c2.forces.drag.model='dria';
    W2 = op.buildWorld(c2); [~,p2] = op.accel(0,r,v,W2);
    moved = abs(norm(p2.drag) - norm(parts.drag))/max(norm(parts.drag),realmin) > 1e-6;
    fprintf('    panel drag responds to attitude: %s   (identical = R_bi frozen at eye(3))\n', ...
            subsref_tern(moved,'PASS','FAIL'));
    nfail = nfail + ~moved;
catch err
    fprintf('    ERROR: %s\n', err.message); nfail = nfail+1;
end

%% ---- 7. SEED RULE: propagation starts EXACTLY at the given state -----------
fprintf('\n[7] seed rule\n');
try
    K = de440.constants(); a = K.Re_earth+400e3;
    cfg = config.defaultConfig();
    cfg.epoch=[2007 1 1 0 0 0]; cfg.r0=[a;100;-50]; cfg.v0=[10;sqrt(K.mu_earth/a);5];
    cfg.tspan=600; cfg.output=struct('times',(0:60:600).');
    cfg.gravityField=struct('field','default','degree',4);
    cfg.forces=struct('gravity',struct('on',true,'model','twobody'), ...
      'drag',struct('on',false),'thirdbody',struct('on',false),'srp',struct('on',false), ...
      'erp',struct('on',false),'relativity',struct('on',false), ...
      'solidtides',struct('on',false),'oceantides',struct('on',false));
    s = op.propagate(cfg);
    dr = norm(s.r(1,:).'-cfg.r0); dv = norm(s.v(1,:).'-cfg.v0);
    fprintf('    |r(0)-r0| = %.3e m, |v(0)-v0| = %.3e m/s   %s\n', dr, dv, ...
            subsref_tern(dr<1e-9 && dv<1e-12,'PASS','FAIL'));
    nfail = nfail + ~(dr<1e-9 && dv<1e-12);
catch err
    fprintf('    ERROR: %s\n', err.message); nfail = nfail+1;
end

%% ---- 8. STRUCT CONTRACTS: run them, do not read them ----------------------
% The rms3d/acc.ratio/DO_PLOTS class. A static checker gets the variable case; the
% FIELD case needs execution, because a struct's shape depends on which branch ran.
fprintf('\n[8] struct contracts (test_contracts)\n');
try
    nf_ = test_contracts();
    if isempty(nf_), nf_ = 0; end
    nfail = nfail + nf_;
catch err
    fprintf('    ERROR: %s\n', err.message); nfail = nfail+1;
end

fprintf('\n=========================================================\n');
if nfail == 0
    fprintf(' ALL PASS.  (Octave + synthetic inputs: this proves WIRING,\n');
    fprintf(' units, frames and dispatch -- NOT the numbers. For those you\n');
    fprintf(' need validate_OD on real MATLAB with real data.)\n');
else
    fprintf(' %d FAILURE(S) -- investigate before trusting any result.\n', nfail);
end
fprintf('=========================================================\n');
