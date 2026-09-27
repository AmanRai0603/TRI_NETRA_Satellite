function nfail = test_contracts()
%TEST_CONTRACTS  Exercise every struct contract by RUNNING it, not by reading it.
%
%   RUN:  setup_paths; test_contracts
%
%   ---------------------------------------------------------------------------
%   WHY THIS EXISTS AND check_undefined.py DOES NOT COVER IT
%   ---------------------------------------------------------------------------
%   `R.rdo.rms3d` when rtn_stats returns `pos3D`. `R.acc.ratio` when od_metrics
%   never assigns one. `DO_PLOTS` when the toggle is `PLOTS`. Three crashes, all
%   found by the user AFTER a 20 s propagation and every download, because MATLAB
%   does not check a field until the line runs.
%
%   check_undefined.py catches the VARIABLE case cleanly. It cannot catch the FIELD
%   case: a struct's shape depends on which branch ran, so judging it statically
%   needs type inference, and the attempt produced 19 hits that were nearly all
%   noise. A checker you cannot trust gets switched off.
%
%   So the field case is tested by EXECUTION instead: build the structs that really
%   occur -- including the degenerate ones (empty R, R with only some metrics, a
%   satellite with no accelerometer) -- and see what throws. The degenerate cases
%   are the point: R's contents legitimately vary with which products exist, so a
%   reader that assumes a shape is wrong even when today's data happens to fit.
%
%   Add a case here whenever a struct crosses a function boundary.
setup_paths; addpath('/tmp/octshim'); warning('off','all');
nfail = 0;
K = de440.constants();

% ---- 1. validation.provenance across every model combination ----------------
printf('[1] validation.provenance over model combos\n');
S = sgeom.vleo16u();
combos = {'cannonball','cannonball','knocke'; 'dria','boxwing','boxwing'; ...
          'sentman','cannonball','ceres'; 'cll','boxwing','simple'; 'sesam','cannonball','knocke'};
for i = 1:size(combos,1)
  cfg = config.defaultConfig();
  cfg.spacecraft = struct('mass',522,'Aref',0.767,'Cd',3,'Cr',1.3,'facets',S.facets,'attitude','ram');
  cfg.forces = struct('drag',struct('on',true,'model',combos{i,1},'atmos','dtm2020', ...
                        'gsi',struct('Tw',300,'aT',0.9,'sig_n',0.9,'sig_t',0.9)), ...
                      'srp',struct('on',true,'model',combos{i,2}), ...
                      'erp',struct('on',true,'model',combos{i,3}));
  cfg.gravityField = struct('field','default','degree',6);
  for refcase = 1:3
    REF = struct();
    if refcase >= 1, REF.RDO = struct('mjd',(1:10).'); end
    if refcase >= 2, REF.ACC = struct('mjd',(1:10).'); REF.ATT = struct('mjd',(1:10).'); end
    if refcase >= 3, REF.DEN = struct('mjd',(1:10).'); REF.TUD = struct('mjd',(1:10).'); end
    try
      P = validation.provenance(cfg, struct(), REF, 'CHAMP');
      assert(isfield(P,'items') && isfield(P,'n'));
    catch e
      printf('    FAIL %s/%s/%s ref%d: %s\n', combos{i,1}, combos{i,2}, combos{i,3}, refcase, e.message); nfail=nfail+1;
    end
  end
end
printf('    %d combos x 3 ref cases\n', size(combos,1));

% ---- 2. show_provenance with PARTIAL R (the real failure mode) --------------
printf('[2] show_provenance with partial/odd R\n');
graphics_toolkit('gnuplot');
cfg = config.defaultConfig();
cfg.spacecraft = struct('mass',522,'Aref',0.767,'Cd',3,'Cr',1.3);
cfg.forces = struct('drag',struct('on',true,'model','cannonball','atmos','dtm2020'), ...
                    'srp',struct('on',true,'model','cannonball'),'erp',struct('on',true,'model','knocke'));
P = validation.provenance(cfg, struct(), struct('RDO',struct('mjd',(1:5).')), 'CHAMP');
Rs = { [], struct(), ...
       struct('rdo',struct('pos3D',4.5)), ...
       struct('rdo',struct('pos3D',4.5),'kin',struct('pos3D',4.4),'refSpread',0.01), ...
       struct('acc',struct('rms_mod',2e-7,'rms_meas',2e-6)), ...
       struct('acc',struct('rms_mod',2e-7,'rms_meas',0)), ...
       struct('den',struct('ratio',0.94),'tud',struct('ratio',1.14)), ...
       struct('rdo',struct()), ...
       struct('acc',struct()) };
for i=1:numel(Rs)
  try
    f = show_provenance(P, Rs{i}, 'fieldcheck', '');
    close(f);
  catch e
    printf('    FAIL R case %d: %s\n', i, e.message); nfail=nfail+1;
  end
end
printf('    %d R shapes\n', numel(Rs));

% ---- 3. validation.capability over every catalog satellite ------------------
printf('[3] validation.capability over all 24 satellites\n');
T = fileread('05_data/sat_data/itsg_catalog.csv');
L = strsplit(strtrim(T), sprintf('\n'));
for i=2:numel(L)
  c = strsplit(L{i},','); sat = strtrim(c{1});
  try
    CAP = validation.capability(sat, false);
    assert(isfield(CAP,'drag') && isfield(CAP,'srp') && isfield(CAP,'erp') && isfield(CAP,'has'));
  catch e
    printf('    FAIL %s: %s\n', sat, e.message); nfail=nfail+1;
  end
end
printf('    %d satellites\n', numel(L)-1);

% ---- 4. drag.force / drag.cannonball diagnostic contract --------------------
printf('[4] drag out-struct contract (Cd_A, A_proj, Aref_used, qd)\n');
geo = validation.probe_geo(350,'2007-01-01');
sw  = atmos.spaceweather(geo.utc, struct('manual',struct('F107',90,'F107a',90,'ap',8,'Kp',2)));
atm = atmos.provider('dtm2020', geo, sw);
R_  = dgeom.ramAttitude([0;7700;0]);
o   = struct('mass',522,'Aref',0.767,'R_bi',R_,'wind',[0;0;0],'omega',0,'sunHat_eci',[0;0;1e11]);
need = {'Cd','Cd_A','A_proj','Aref_used','qd','a'};
for m = {'sentman','dria','cll','sesam'}
  try
    out = drag.force([K.Re_earth+350e3;0;0],[0;7700;0],atm,S.facets,m{1}, ...
                     struct('Tw',300,'aT',0.9,'sig_n',0.9,'sig_t',0.9),o);
    for j=1:numel(need)
      if ~isfield(out,need{j}), printf('    FAIL %s: no .%s\n', m{1}, need{j}); nfail=nfail+1; end
    end
  catch e
    printf('    FAIL %s: %s\n', m{1}, e.message); nfail=nfail+1;
  end
end
out = drag.cannonball([K.Re_earth+350e3;0;0],[0;7700;0],atm.rho,3.0,0.767,struct('mass',522,'omega',0));
for j=1:numel(need)
  if ~isfield(out,need{j}), printf('    FAIL cannonball: no .%s\n', need{j}); nfail=nfail+1; end
end
printf('    5 drag models share one contract\n');

% ---- 5. THE BIAS FIX, on a case where the answer is KNOWN -------------------
% A model that is EXACTLY the truth, plus a realistic instrument bias. The raw ratio
% must call it a failure and the corrected one must return 1.0. If this ever breaks,
% metric [3] is lying again -- and every compare_OD sweep against it goes flat, which
% reads as "this knob does not matter" and is a confident wrong answer.
printf('[5] acc_ratio recovers a known-perfect model from under a bias\n');
nn = 200; tt = (0:nn-1).'*30;
sig  = 2e-7*[cos(2*pi*tt/5400), 0.3*sin(2*pi*tt/5400), 0.1*ones(nn,1)];
bias = repmat([1.9e-6 0.4e-6 0.2e-6], nn, 1);
Rb = struct('acc', struct('t',tt, 'meas', sig+bias, 'mod', sig, ...
      'rms_meas',sqrt(mean(sum((sig+bias).^2,2))), 'rms_mod',sqrt(mean(sum(sig.^2,2)))));
raw = Rb.acc.rms_mod/Rb.acc.rms_meas;
[rc, okc] = validation.acc_ratio(Rb);
if abs(rc-1) > 1e-9 || ~okc
    printf('    FAIL: corrected ratio %.6f (expect 1.0), corrected=%d\n', rc, okc); nfail=nfail+1;
end
if raw > 0.2
    printf('    FAIL: raw %.4f -- the case no longer reproduces the bug\n', raw); nfail=nfail+1;
end
printf('    raw %.4f (calls a PERFECT model a 10x failure) -> corrected %.4f   PASS\n', raw, rc);

% ---- 6. CLOSURE: metrics [1] and [3] must be cross-checked ------------------
% The CHAMP case: metric [3] claimed a deficit demanding 104 m of along-track drift
% while metric [1] measured 2.2 m. They contradicted each other by 48x for twelve
% rounds because nothing compared them. This locks the comparison in.
printf('[6] closure catches metrics that contradict each other\n');
tc = (0:30:10800).';
Rbad  = struct('acc',struct('t',tc,'rms_ac_meas',1.994e-6,'rms_ac_mod',2.081e-7), 'rdo',struct('along',2.16));
Rgood = struct('acc',struct('t',tc,'rms_ac_meas',3.0e-8,  'rms_ac_mod',1.59e-8),  'rdo',struct('along',2.091));
Cb = validation.closure(Rbad,  false);
Cg = validation.closure(Rgood, false);
if Cb.ok
    printf('    FAIL: the CHAMP raw case (48x contradiction) reported CONSISTENT\n'); nfail=nfail+1;
end
if ~Cg.ok
    printf('    FAIL: the GRACE-A case (2.5x) reported INCONSISTENT\n'); nfail=nfail+1;
end
if abs(Cb.dx_pred - 104.15) > 1
    printf('    FAIL: dx_pred %.2f, expected ~104 m\n', Cb.dx_pred); nfail=nfail+1;
end
printf('    CHAMP raw: %.0f m predicted vs 2.2 m measured -> INCONSISTENT (correct)\n', Cb.dx_pred);
printf('    GRACE-A  : %.2f m predicted vs 2.09 m measured -> CONSISTENT (correct)   PASS\n', Cg.dx_pred);

% ---- 7. ECLIPSE SPLIT: separate drag from SRP, on KNOWN injected errors ------
% At 480 km in solar minimum SRP is 0.81x the drag, so metric [3]'s deficit could be
% either -- an RMS over the arc cannot tell. SRP is exactly zero in eclipse and drag
% is not, so the orbit runs the experiment for free. This injects a KNOWN error in
% one force at a time and checks acc_split names the right one.
%
% The first version of acc_split compared the eclipse and sunlit RATIOS and called a
% pure drag error "SRP". Correct SRP DILUTES a drag error -- the sunlit ratio is
% (r_d*D+S)/(D+S), which moves toward 1 as S grows -- so the ratios differ for a
% reason that has nothing to do with SRP. This test killed that version.
printf('[7] eclipse split names the right force\n');
Ks = de440.constants(); ns = 720; ts = (0:ns-1).'*15;
oms = sqrt(Ks.mu_earth/(Ks.Re_earth+480e3)^3);
sun = mod(ts, 2*pi/oms) < 0.6*(2*pi/oms);
dT = 1.4e-8*[cos(oms*ts), 0.3*sin(oms*ts)+0.5, 0.2*cos(2*oms*ts)];
sT = 1.2e-8*[0.5*ones(ns,1), cos(oms*ts), 0.3*sin(oms*ts)] .* sun;
mkA = @(ds,ss) struct('t',ts,'meas',dT+sT,'mod',ds*dT+ss*sT, ...
    'ac_meas',(dT+sT)-mean(dT+sT,1),'ac_mod',(ds*dT+ss*sT)-mean(ds*dT+ss*sT,1), ...
    'cmp',struct('drag',ds*dT,'srp',ss*sT,'erp',zeros(ns,3)));
Sd = validation.acc_split(struct('acc',mkA(0.4,1.0)), false);   % DRAG is wrong
Ss = validation.acc_split(struct('acc',mkA(1.0,0.4)), false);   % SRP is wrong
Sb = validation.acc_split(struct('acc',mkA(1.0,1.0)), false);   % both fine
if isempty(strfind(Sd.verdict,'DRAG'))
    printf('    FAIL: injected a DRAG error, verdict said: %s\n', Sd.verdict); nfail=nfail+1;
end
if isempty(strfind(Ss.verdict,'SRP')) || abs(Ss.ratio_drag-1) > 0.15
    printf('    FAIL: injected an SRP error, drag ratio %.3f (want ~1), verdict: %s\n', ...
           Ss.ratio_drag, Ss.verdict); nfail=nfail+1;
end
if abs(Sb.ratio_drag-1) > 0.05
    printf('    FAIL: nothing injected, drag ratio %.3f\n', Sb.ratio_drag); nfail=nfail+1;
end
printf('    drag err -> drag %.3f / srp %.3f | srp err -> drag %.3f / srp %.3f   PASS\n', ...
       Sd.ratio_drag, Sd.ratio_srp, Ss.ratio_drag, Ss.ratio_srp);

% ---- 8. EVERY FIGURE CAN BE SWITCHED OFF -----------------------------------
% By RUNNING show_OD, not by reading it. The previous version parsed the source and
% compared the defaults struct to `tags{end+1} = 'literal'` lines: it broke as soon
% as a tag came from a variable, and it counted sampling knobs as figure switches.
% It was measuring the text, not the behaviour.
printf('[8] every figure can be switched off (behavioural)\n');
nfail = nfail + test_figure_toggles(true);

% ---- 9. SCALE FACTOR knows when it is meaningless ---------------------------
% A scale factor ALWAYS exists -- the formula returns something for any two series,
% including unrelated ones. The number that keeps you honest is r2. If this test
% breaks, a fitted k could be quoted as a Cd correction when the model does not even
% track the measurement.
printf('[9] scale_factor reports shape, not just size\n');
tt2 = (0:0.05:6*pi).'; b2 = sin(tt2) + 0.3*cos(3*tt2);
S1 = validation.scale_factor(b2,        0.53*b2);      % right shape, wrong size
S2 = validation.scale_factor(b2,        cos(2.3*tt2)); % wrong shape
S3 = validation.scale_factor(b2,        -b2);          % anti-correlated
if abs(S1.factor - 1/0.53) > 1e-6 || S1.r2 < 0.99
    printf('    FAIL: 0.53x truth -> k %.4f r2 %.3f (expect 1.887 / 1.0)\n', S1.factor, S1.r2); nfail=nfail+1;
end
if S2.r2 > 0.1
    printf('    FAIL: unrelated model reported r2 %.3f\n', S2.r2); nfail=nfail+1;
end
if S3.factor > 0 || isempty(strfind(S3.note,'ANTI'))
    printf('    FAIL: anti-correlated model not flagged (k %.3f)\n', S3.factor); nfail=nfail+1;
end
printf('    0.53x truth -> k=%.3f r2=%.2f | unrelated -> r2=%.2f | anti -> k=%.1f flagged   PASS\n', ...
       S1.factor, S1.r2, S2.r2, S3.factor);

% ---- 10. NO KNOB CONTROLS NOTHING ------------------------------------------
% `ratio_zero_frac` survived a figure rewrite that deleted its only consumer, and
% sat in the defaults advertising a capability that no longer existed. A knob that
% controls nothing is worse than no knob: someone sets it, sees no change, and
% concludes the QUANTITY does not matter -- when in fact the knob was never read.
% That is the same failure as FORCES.drag.corotate (round ~20), which was
% documented, sweepable, and silently ignored.
printf('[10] every knob has a consumer, and no knob is a figure\n');
% ASK show_OD what its knobs are; do not parse its source. The previous version
% regexed the defaults struct out of the file, and when defaults() was restructured
% the regex matched nothing, dtx{1} threw "out of bound 0" -- and the suite still
% printed ALL PASS. A test that ERRORS without failing is worse than one that cannot
% fail: it looks like it ran.
P10    = show_OD();                     % defaults, draws nothing
figs10 = figureToggles_probe();         % the ONE declared figure list
knobs  = setdiff(fieldnames(P10), figs10);
src2   = fileread(fullfile('06_validation','realsat','show_OD.m'));
cut_   = strfind(src2, 'function names = figureToggles');
body   = src2(1:cut_(1));
deadk  = {};
for k = 1:numel(knobs)
    if isempty(regexp(body, ['P\.' knobs{k} '\>'], 'once'))
        deadk{end+1} = knobs{k};
    end
end
if ~isempty(deadk)
    printf('    FAIL: knob(s) that control nothing: %s\n', strjoin(deadk, ', ')); nfail=nfail+1;
else
    printf('    %d knobs (%s), every one has a consumer   PASS\n', ...
           numel(knobs), strjoin(knobs, ', '));
end

% ---- 11. VERSIONED PRODUCT DIRECTORIES -------------------------------------
% The server holds the density under .../CHAMP/neutralDensity_1.0/2003/ . Our own
% whitelist rejected that name -- "unknown product neutralDensity_1.0" -- before a
% single URL was tried, because productInfo matched on lower(strrep(p,'_','')) which
% turns it into "neutraldensity1.0". A name the CATALOG had read off the server was
% refused by us and reported as missing data.
%
% This could not be tested at all until the product table and URL builder were
% exposed: the only way to exercise them was a live request and a 404. A rule that
% decides whether data is reachable must be checkable without the network.
printf('[11] versioned product directories (neutralDensity_1.0)\n');
Pv = data.itsg_productinfo('neutralDensity_1.0');
if ~strcmp(Pv.name, 'neutralDensity')
    printf('    FAIL: layout resolved to ''%s'', expected ''neutralDensity''\n', Pv.name); nfail=nfail+1;
end
if ~strcmp(Pv.dir, 'neutralDensity_1.0')
    printf('    FAIL: directory resolved to ''%s'', expected ''neutralDensity_1.0''\n', Pv.dir); nfail=nfail+1;
end
% the unversioned name must still work, and _ACC must NOT be treated as a version
Pa = data.itsg_productinfo('neutralDensity_ACC');
if ~strcmp(Pa.name, 'neutralDensity_ACC')
    printf('    FAIL: neutralDensity_ACC resolved to ''%s'' -- _ACC is not a version\n', Pa.name); nfail=nfail+1;
end
uv = data.itsg_urls('CHAMP','neutralDensity_1.0','neutralDensity','2003-01-01');
want = 'CHAMP/neutralDensity_1.0/2003/CHAMP_neutralDensity_2003-01-01.txt.gz';
if isempty(strfind(uv{1}, want))
    printf('    FAIL: first URL is\n      %s\n    expected to end with\n      %s\n', uv{1}, want); nfail=nfail+1;
else
    printf('    _1.0 -> layout %s, dir %s, URL #1 %s   PASS\n', Pv.name, Pv.dir, want);
end

if nfail==0
    printf('    all contracts hold (5 model combos x 3 ref shapes, 9 R shapes,\n');
    printf('    24 satellites, 5 drag models on one out-struct)   PASS\n');
else
    printf('    %d contract FAILURE(S)\n', nfail);
end
end
