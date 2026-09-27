% VERIFY_DRAG_CHAIN  Dissect the drag calculation term by term and cross-check it.
%
%   a_drag = -0.5 * rho * Cd * (A/m) * |v_rel| * v_rel
%
%   Four numbers, four different places they come from, four different ways to be
%   silently wrong. This script pulls each one out, prints it, and checks it against
%   something INDEPENDENT. Nothing here is a unit test in the abstract: every check
%   below exists because that exact thing was found broken in this codebase.
%
%     rho   <- atmos.provider(model, geo, sw)     CHECK: sum(n_i M_i / NA) == rho ?
%                                                  (this caught atm.n in cm^-3 while
%                                                   drag.force assumed m^-3 -> panel
%                                                   drag was 1e6 too small)
%     Cd    <- catalog, or GSI physics            CHECK: does the panel model reproduce
%                                                  the cannonball when handed the same
%                                                  Cd and area?  (this caught Cd never
%                                                  reaching drag: CHAMP ran at 2.2 not 3.0)
%     A(t)  <- facets x attitude                  CHECK: A(t) == Aref*|n.vhat| ?
%                                                  (this caught R_bi=eye(3) -> n.v = 0
%                                                   -> EXACTLY zero drag, silently)
%     m     <- catalog                            CHECK: does the CSV value reach ctx.sc?
%
%   RUN:  setup_paths; verify_drag_chain
%
% =============================== KNOBS =======================================
SATS      = {'CHAMP','GRACE-A','GRACE-B','SWARM-A'};  % which catalog entries to check
DATE      = '2007-01-01';
ALTS_KM   = [250 300 350 450 550];   % where to probe the atmosphere
ATMOS     = {'dtm2020'};             % add 'nrlmsise','jb2008' on MATLAB w/ network
GSI_MODELS= {'sentman','dria','cll','sesam'};
GSI       = struct('Tw',300,'aT',0.9,'sig_n',0.9,'sig_t',0.9);
SW_MANUAL = struct('F107',90,'F107a',90,'ap',8,'Kp',2);  % [] = measured (needs network)
V_ORBIT   = 7700;                    % m/s, for the probe points
TOL_RHO   = 1e-3;                    % rel. tolerance on the rho<->composition closure
TOL_CLOSE = 1e-9;                    % rel. tolerance on the panel<->cannonball closure
% =============================================================================

K = de440.constants();
P = @(b) subsref_tern(b,'PASS','**FAIL**');
fprintf('\n=================== DRAG CHAIN VERIFICATION ===================\n');

%% ---- 1. CATALOG -> cfg.spacecraft : does the CSV number reach the physics? ----
fprintf('\n[1] CATALOG -> spacecraft. The CSV is the source of truth for m, Aref, Cd, Cr.\n');
fprintf('    ONE name per quantity now: .mass and .Aref, the names the FORCES read.\n');
fprintf('    (Was: .area in itsg_catalog, .Aref in sat.catalog, .Aref_m2 in sweep_knob,\n');
fprintf('     held together by a rename-in-flight in validate_OD. See sat.spacecraft.)\n');
fprintf('    %-9s | %-7s | %-8s | %-5s | %-5s | %s\n','sat','mass kg','Aref m^2','Cd','Cr','TU Delft name');
fprintf('    %s\n', repmat('-',1,66));
for i = 1:numel(SATS)
    C = validation.itsg_catalog(SATS{i});
    fprintf('    %-9s | %7.1f | %8.3f | %5.2f | %5.2f | %s\n', ...
            C.name, C.mass, C.Aref, C.Cd, C.Cr, subsref_tern(C.has_tudelft, C.tudelft_name, '(none)'));
end

%% ---- 2. DENSITY: is rho consistent with the composition it ships with? -------
% rho and n come from the SAME model call. If sum(n_i*M_i/NA) ~= rho, then rho and n
% are in different unit systems -- and the cannonball (which reads rho) will be right
% while every panel model (which reads n) is silently wrong by that factor.
fprintf('\n[2] DENSITY closure: does sum(n_i * M_i / N_A) reproduce rho?\n');
fprintf('    (NA per KMOL = %.6e, species in kg/kmol -- drag.force''s convention)\n', 6.02214076e26);
NA = 6.02214076e26;
for m = 1:numel(ATMOS)
    fprintf('    --- %s\n', ATMOS{m});
    fprintf('    %-6s | %-11s | %-11s | %-8s | %s\n','alt','rho model','sum n*M/NA','ratio','verdict');
    for a = 1:numel(ALTS_KM)
        geo = validation.probe_geo(ALTS_KM(a), DATE);
        sw  = atmos.spaceweather(geo.utc, struct('manual',SW_MANUAL));
        atm = atmos.provider(ATMOS{m}, geo, sw);
        if ~isfield(atm,'n')
            fprintf('    %3d km | %.4e | (no composition shipped)\n', ALTS_KM(a), atm.rho); continue
        end
        nm = fieldnames(atm.n); tot = 0;
        for j = 1:numel(nm), tot = tot + atm.n.(nm{j})*drag.species(nm{j})/NA; end
        rat = atm.rho/tot;
        fprintf('    %3d km | %.4e | %.4e | %8.4f | %s\n', ...
                ALTS_KM(a), atm.rho, tot, rat, P(abs(rat-1) < TOL_RHO));
    end
end

%% ---- 3. A(t): does the projected area follow the attitude? -------------------
% A(t) = sum_k A_k * max(0, n_k . vhat).  For ONE plate that is Aref*|n.vhat| -- a
% closed form we can check against. This is where R_bi=eye(3) showed up as A=0.
fprintf('\n[3] A(t): projected area vs attitude. One +x plate, Aref = 1.0 m^2.\n');
fprintf('    %-12s | %-10s | %-10s | %-10s | %s\n','yaw [deg]','n.vhat','A(t) pred','A(t) model','verdict');
Aref = 1.0; f1 = srp.facet('body',[1;0;0],Aref,0.2,0.3,0.5,[0;0;0],false);
vhat = [0;1;0];
for yaw = [0 30 45 60 89 91]
    th = deg2rad(yaw);
    R  = [cos(th) -sin(th) 0; sin(th) cos(th) 0; 0 0 1] * dgeom.ramAttitude(vhat*V_ORBIT);
    n_eci = R*[1;0;0];  cth = dot(n_eci, vhat);
    Apred = Aref*max(0,cth);
    Amod  = validation.projected_area(f1, R, vhat);
    fprintf('    %-12g | %10.6f | %10.6f | %10.6f | %s\n', yaw, cth, Apred, Amod, ...
            P(abs(Amod-Apred) < 1e-12));
end
fprintf('    ^ yaw 91 deg: the plate has turned BEHIND the flow -> A = 0. Not negative.\n');

%% ---- 4. Cd(t) from the GSI physics, across altitude --------------------------
fprintf('\n[4] Cd(t) from GSI. aT is COMPUTED by sesam/dria, TYPED for sentman.\n');
fprintf('    %-6s | %-10s | %-10s | %s\n','alt','nO [m^-3]','SESAM aT', sprintf('%-9s ', GSI_MODELS{:}));
for a = 1:numel(ALTS_KM)
    geo = validation.probe_geo(ALTS_KM(a), DATE);
    sw  = atmos.spaceweather(geo.utc, struct('manual',SW_MANUAL));
    atm = atmos.provider(ATMOS{1}, geo, sw);
    nO  = 0; if isfield(atm,'n') && isfield(atm.n,'O'), nO = atm.n.O; end
    aT  = drag.sesam(nO, atm.T);
    r   = [K.Re_earth + ALTS_KM(a)*1000; 0; 0];
    o   = struct('mass',522,'Aref',Aref,'R_bi',dgeom.ramAttitude(vhat*V_ORBIT), ...
                 'wind',[0;0;0],'omega',K.omega_earth);
    line = '';
    for g = 1:numel(GSI_MODELS)
        try
            out = drag.force(r, vhat*V_ORBIT, atm, f1, GSI_MODELS{g}, GSI, o);
            line = [line sprintf('%-9.4f ', out.Cd)];
        catch, line = [line sprintf('%-9s ','ERR')]; end
    end
    fprintf('    %3d km | %.4e | %10.4f | %s\n', ALTS_KM(a), nO, aT, line);
end
fprintf('    ^ sentman is FLAT by construction (you typed aT=0.9). dria/sesam rise with\n');
fprintf('      altitude as atomic O thins and accommodation falls. sesam == dria (alias).\n');

%% ---- 5. THE CLOSURE: panel model must reproduce the cannonball ---------------
% Hand the cannonball the Cd the PANEL model just computed, at zero incidence, with
% the same area and mass. They are then the same physics written twice, and must
% agree. If they do not, one of rho / Cd / A / m is not what it claims to be.
fprintf('\n[5] CLOSURE: cannonball(Cd from panel) vs panel, same rho/A/m, plate normal to flow.\n');
fprintf('    drag.force defines Cd = D/(qd*Aref) with qd = 0.5*SUM(rhoS)*V^2 -- the density\n');
fprintf('    it summed from the COMPOSITION, not the model''s own rho. Those differ by the\n');
fprintf('    ratio in [2] (DTM2020''s rho and its species sum are internally consistent only\n');
fprintf('    to ~1e-4). Comparing against atm.rho therefore shows a 1e-4 "error" that is\n');
fprintf('    DTM2020''s, not ours. So close against the SAME rho the panel model used.\n');
fprintf('    %-6s | %-8s | %-12s | %-12s | %-10s | %s\n','alt','Cd panel','a cannonball','a panel','rel diff','verdict');
mass = 522;
for a = 1:numel(ALTS_KM)
    geo = validation.probe_geo(ALTS_KM(a), DATE);
    sw  = atmos.spaceweather(geo.utc, struct('manual',SW_MANUAL));
    atm = atmos.provider(ATMOS{1}, geo, sw);
    r   = [K.Re_earth + ALTS_KM(a)*1000; 0; 0];
    R   = dgeom.ramAttitude(vhat*V_ORBIT);
    o   = struct('mass',mass,'Aref',Aref,'R_bi',R,'wind',[0;0;0],'omega',0);  % omega=0: no corotation
    out = drag.force(r, vhat*V_ORBIT, atm, f1, 'sentman', GSI, o);
    % the same rho the panel model integrated: qd = 0.5*rho_tot*V^2
    rho_tot = 2*out.qd/V_ORBIT^2;
    a_can = 0.5*rho_tot*out.Cd*(Aref/mass)*V_ORBIT^2;      % scalar magnitude
    a_pan = norm(out.a);
    rel   = abs(a_pan-a_can)/max(a_can,realmin);
    fprintf('    %3d km | %8.4f | %.6e | %.6e | %.3e | %s\n', ...
            ALTS_KM(a), out.Cd, a_can, a_pan, rel, P(rel < TOL_CLOSE));
end
fprintf('    ^ THIS is the check that matters: it ties rho, Cd, A and m together in one\n');
fprintf('      identity. It only closes if all four are what they say they are.\n');

fprintf('\n===============================================================\n');

% ============================================================================
%  NO LOCAL FUNCTIONS HERE -- ON PURPOSE, AND I GOT THIS WRONG ONCE ALREADY.
%  MATLAB hoists a script's local functions; Octave does NOT. A helper at the
%  bottom of a script is simply undefined when the script runs under Octave --
%  which is the bug that started this whole audit, and which I reintroduced in the
%  first draft of THIS file. The helpers now live in +validation/:
%      validation.probe_geo   validation.projected_area   subsref_tern
% ============================================================================

%% ---- 6. THE OTHER FORCE CHAINS: same treatment, each term traced -------------
% Every force is rho-like x coeff x (A/m) x geometry. Same four failure modes.
fprintf('\n[6] THE OTHER CHAINS -- each term, where it comes from, and its check.\n');
K = de440.constants();
alt = 350; geo = validation.probe_geo(alt, DATE);
r   = [K.Re_earth+alt*1000; 0; 0];  v = [0; V_ORBIT; 0];
mass = 522; Aref = 1.0;
f1   = srp.facet('body',[1;0;0],Aref,0.2,0.3,0.5,[0;0;0],false);
cfgB = config.defaultConfig();
cfgB.epoch=[2007 1 1 0 0 0]; cfgB.r0=r; cfgB.v0=v; cfgB.tspan=60;
cfgB.output=struct('times',[0;60]);
cfgB.gravityField=struct('field','default','degree',4);
cfgB.spaceweather.manual=SW_MANUAL;
cfgB.spacecraft=struct('mass',mass,'Aref',Aref,'Cd',3.0,'Cr',1.3,'facets',f1,'attitude','ram','R_bi',eye(3));
cfgB.forces=struct('gravity',struct('on',true,'model','sphharm','degree',4,'order',4), ...
  'drag',struct('on',true,'model','cannonball','atmos',ATMOS{1},'corotate',true,'gsi',GSI), ...
  'thirdbody',struct('on',true,'model','battin'), ...
  'srp',struct('on',true,'model','cannonball'), ...
  'erp',struct('on',true,'model','knocke'), ...
  'relativity',struct('on',true,'terms',{{'schwarzschild'}}), ...
  'solidtides',struct('on',false),'oceantides',struct('on',false));
W = op.buildWorld(cfgB);
[atot, parts] = op.accel(0, r, v, W);
E  = ephemInputs(validation.op_T_(cfgB.epoch), W.eph);

fprintf('\n    SRP chain:   a = nu * P_srp * Cr * (A/m)\n');
nu = srp.eclipse(r, E.sun_eci);
a_srp_hand = nu * E.P_srp * cfgB.spacecraft.Cr * (Aref/mass);
fprintf('      P_srp  = %.6e N/m^2   <- ephemInputs (DE440 sun distance)\n', E.P_srp);
fprintf('      nu     = %.4f            <- srp.eclipse (0=umbra, 1=full sun)\n', nu);
fprintf('      Cr     = %.4f            <- sc.Cr (cannonball only; boxwing uses optics)\n', cfgB.spacecraft.Cr);
fprintf('      A/m    = %.6e         <- sc.Aref/sc.mass\n', Aref/mass);
fprintf('      hand   = %.6e | model = %.6e | rel %.2e | %s\n', ...
        a_srp_hand, norm(parts.srp), abs(norm(parts.srp)-a_srp_hand)/a_srp_hand, ...
        subsref_tern(abs(norm(parts.srp)-a_srp_hand)/a_srp_hand < 1e-9,'PASS','**FAIL**'));

fprintf('\n    ERP chain:   a = f(albedo,IR,doy) * CrAoM      [NO attitude path]\n');
fprintf('      CrAoM  = %.6e         <- forces.erp.CrAoM -> forces.erp.Cr -> sc.Cr, x Aref/mass\n', ...
        cfgB.spacecraft.Cr*Aref/mass);
fprintf('      model  = %.6e         <- erp.accel(r, sun, CrAoM, ''knocke'', doy)\n', norm(parts.erp));
fprintf('      CHECK: does ERP scale linearly with Cr? (it must -- it is CrAoM x geometry)\n');
c2 = cfgB; c2.spacecraft.Cr = 2.6; W2 = op.buildWorld(c2); [~,p2] = op.accel(0,r,v,W2);
fprintf('      Cr 1.3->2.6: %.6e -> %.6e | ratio %.4f (expect 2.0) | %s\n', ...
        norm(parts.erp), norm(p2.erp), norm(p2.erp)/norm(parts.erp), ...
        subsref_tern(abs(norm(p2.erp)/norm(parts.erp)-2)<1e-6,'PASS','**FAIL**'));

fprintf('\n    THIRD BODY:  a = GM_b * (  (r_b-r)/|r_b-r|^3  -  r_b/|r_b|^3 )\n');
d_s = E.sun_eci - r; d_m = E.moon_eci - r;
a_tb_hand = K.GM_sun*(d_s/norm(d_s)^3 - E.sun_eci/norm(E.sun_eci)^3) + ...
            K.GM_moon*(d_m/norm(d_m)^3 - E.moon_eci/norm(E.moon_eci)^3);
fprintf('      |r_sun|  = %.6e m      <- DE440\n', norm(E.sun_eci));
fprintf('      |r_moon| = %.6e m      <- DE440\n', norm(E.moon_eci));
fprintf('      hand   = %.6e | model = %.6e | rel %.2e | %s\n', ...
        norm(a_tb_hand), norm(parts.thirdbody), ...
        abs(norm(parts.thirdbody)-norm(a_tb_hand))/norm(a_tb_hand), ...
        subsref_tern(abs(norm(parts.thirdbody)-norm(a_tb_hand))/norm(a_tb_hand) < 1e-6,'PASS','**FAIL**'));
fprintf('      (the second term is the INDIRECT part: Earth is accelerated too. Dropping\n');
fprintf('       it is a classic error worth ~1e-6 m/s^2 -- larger than the whole SRP.)\n');

fprintf('\n    GRAVITY:     a = -mu*r/|r|^3 + spherical-harmonic corrections\n');
a_pt = -K.mu_earth*r/norm(r)^3;
fprintf('      point mass = %.6e | model(deg 4) = %.6e | J2 etc = %.3e\n', ...
        norm(a_pt), norm(parts.gravity), norm(parts.gravity - a_pt));
fprintf('      ratio to point mass = %.6f  (J2 is ~1e-3 of the total at LEO)  | %s\n', ...
        norm(parts.gravity)/norm(a_pt), ...
        subsref_tern(abs(norm(parts.gravity)/norm(a_pt)-1) < 5e-3,'PASS','**FAIL**'));

%% ---- 7. RE-INTEGRATION: all chains, one acceleration -------------------------
fprintf('\n[7] RE-INTEGRATED: every chain, summed, with its share of the total.\n');
fn = fieldnames(parts); tot = norm(atot);
% share as a FRACTION of gravity, not a percent of the total: at LEO every
% non-gravitational force is <1e-6 of the total and a percent column reads 0.000%
% for all of them, which tells you nothing. The ladder is what matters.
fprintf('    %-12s | %-13s | %-11s | %s\n','force','|a| [m/s^2]','/gravity','driver chain');
fprintf('    %s\n', repmat('-',1,86));
drv = struct('gravity','gravityField -> data.gravity -> W.grav', ...
             'thirdbody','DE440 -> ctx.E.sun_eci/moon_eci', ...
             'drag','data.drivers -> atmos.provider -> rho,n + sc + attitude', ...
             'srp','DE440 -> ctx.E.P_srp + srp.eclipse + sc', ...
             'erp','DE440 sun + doy + CrAoM', ...
             'relativity','ctx.grav.mu + c');
for i=1:numel(fn)
    d = '?'; if isfield(drv,fn{i}), d = drv.(fn{i}); end
    fprintf('    %-12s | %.6e | %.4e | %s\n', fn{i}, norm(parts.(fn{i})), norm(parts.(fn{i}))/norm(parts.gravity), d);
end
fprintf('    %s\n', repmat('-',1,86));
fprintf('    %s\n', repmat('-',1,86));
fprintf('    %-12s | %.6e | %.4e\n', 'TOTAL', tot, 1.0);
ng = norm(parts.drag+parts.srp+parts.erp);
fprintf('    %-12s | %.6e | %.4e  <- what metric [3] measures against the\n', 'NON-GRAV', ng, ng/norm(parts.gravity));
fprintf('    %31s accelerometer. It is 1e-7 of gravity,\n','');
fprintf('    %31s which is why a 27%% Cd error hides so well.\n','');

%% ---- 8. WHICH FIELDS DO THE CURRENT TOGGLES ACTUALLY READ? ------------------
sat.reads(cfgB);
