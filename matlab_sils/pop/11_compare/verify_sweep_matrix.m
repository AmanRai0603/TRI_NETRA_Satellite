% VERIFY_SWEEP_MATRIX  Every sweep knob: is it accepted, and does it MOVE the answer?
%
%   RUN:  setup_paths; verify_sweep_matrix
%
%   ---------------------------------------------------------------------------
%   WHAT THIS CATCHES THAT NOTHING ELSE DOES
%   ---------------------------------------------------------------------------
%   A flat column in a sweep table is ambiguous, and the two meanings are opposite:
%     - the model does not CONSULT that knob  -> correct, expected, informative
%     - the knob is set and then DROPPED      -> a bug that looks like a result
%   EXAMPLE_16U documented an 'SC.Cd' knob that was a no-op for as long as nobody
%   checked (Cd lived in cfg.spacecraft, the physics read cfg.forces.drag). Every
%   model knob was unreachable from compare_OD until round 16. Both were invisible.
%
%   So: sweep every knob twice against a config where it SHOULD matter, and print
%   whether the non-gravitational acceleration moved. The EXPECT column is what the
%   documentation claims -- a mismatch is a finding, in either direction.
%
%   'ignored (by design)' is a PASS: 'dria'/'sesam' derive the accommodation
%   coefficient from the local atomic oxygen and genuinely do not read gsi.aT.
%   sat.reads(cfg) says which fields your toggles consult before you spend arcs.
%
% =============================== KNOBS =======================================
ALT_KM = 350;
EPOCH  = [2007 1 1 0 0 0];
SW     = struct('F107',90,'F107a',90,'ap',8,'Kp',2);
% =============================================================================
K = de440.constants();

% a REALISTIC 16U: real geometry (sgeom.vleo16u), real-ish mass, VLEO altitude
S16 = sgeom.vleo16u();
plate = srp.facet('body',[1;0;0],0.04,0.2,0.3,0.5,[0;0;0],false);

base = struct();
base.epoch = EPOCH; base.alt_km = ALT_KM; base.sw = SW;
base.IM = 'rk78'; base.IN = struct('method','rk78','rtol',1e-10,'atol',1e-6);
base.SC = struct('mass_kg',S16.mass,'Aref_m2',0.04,'Cd',2.2,'Cr',1.3, ...
                 'facets',S16.facets,'attitude','ram');
base.F  = struct( ...
  'gravity',    struct('on',true,'model','sphharm','degree',6,'order',6), ...
  'drag',       struct('on',true,'model','dria','atmos','dtm2020','corotate',true, ...
                       'gsi',struct('Tw',300,'aT',0.9,'sig_n',0.9,'sig_t',0.9)), ...
  'thirdbody',  struct('on',true,'model','battin'), ...
  'srp',        struct('on',true,'model','boxwing'), ...
  'erp',        struct('on',true,'model','boxwing','nrings',4,'nseg',8), ...
  'relativity', struct('on',true,'terms',{{'schwarzschild'}}), ...
  'solidtides', struct('on',false), 'oceantides', struct('on',false));

% knob | v1 | v2 | what the docs claim
T = {
 'FORCES.drag.atmos',      'exponential','dtm2020',   'moves'
 'FORCES.drag.model',      'cannonball','dria',       'moves'
 'FORCES.drag.corotate',   true,        false,        'moves'
 'FORCES.drag.gsi.aT',     0.7,         1.0,          'ignored by dria (it DERIVES aT)'
 'FORCES.drag.gsi.Tw',     250,         400,          'moves'
 'FORCES.srp.model',       'cannonball','boxwing',    'moves'
 'FORCES.erp.model',       'knocke',    'boxwing',    'moves'
 'FORCES.erp.nrings',      4,           16,           'moves (slightly: cap resolution)'
 'FORCES.gravity.degree',  2,           6,            'moves'
 'FORCES.thirdbody.on',    true,        false,        'ignored (not in the non-grav sum)'
 'FORCES.srp.on',          true,        false,        'moves'
 'FORCES.erp.on',          true,        false,        'moves'
 'FORCES.relativity.on',   true,        false,        'ignored (not in the non-grav sum)'
 'SC.Cd',                  2.2,         3.5,          'ignored by dria (it DERIVES Cd)'
 'SC.Cr',                  1.3,         2.0,          'ignored by srp boxwing (facet optics)'
 'SC.Aref',                0.04,        0.08,         'ignored by panel+boxwing (facets rule)'
 'SC.mass',                24,          48,           'moves'
 'SC.attitude',            'ram',       [],           'moves'
 'INTEG_METHOD',           'rk78',      'rk45',       'ignored (single accel eval)'
};

fprintf('\n============== SWEEP KNOB MATRIX ==============\n');
fprintf('  base: 16U (sgeom.vleo16u, REAL geometry), %d km, drag=dria/dtm2020,\n', ALT_KM);
fprintf('        srp=boxwing, erp=boxwing, attitude=ram\n');
fprintf('  "moves" is measured on the NON-GRAVITATIONAL sum: gravity is 1e7 times\n');
fprintf('  larger and would hide a 100%% drag change inside its own rounding.\n\n');
fprintf('  %-24s | %-9s | %-11s | %s\n','knob','moved?','rel change','documented expectation');
fprintf('  %s\n', repmat('-',1,92));
nsurprise = 0;
for i = 1:size(T,1)
    [mv, rel, note] = validation.try_knob(T{i,1}, T{i,2}, T{i,3}, base, K);
    exp_moves = ~isempty(strfind(T{i,4},'moves'));
    if rel < 0
        tag = 'ERROR'; nsurprise = nsurprise+1;
        fprintf('  %-24s | %-9s | %-11s | %s\n', T{i,1}, tag, '-', note);
        continue
    end
    agree = (mv == exp_moves);
    tag = subsref_tern(mv,'YES','no');
    fprintf('  %-24s | %-9s | %-11.3e | %s%s\n', T{i,1}, tag, rel, T{i,4}, ...
            subsref_tern(agree,'','   <== DISAGREES WITH THE DOC'));
    nsurprise = nsurprise + ~agree;
end
fprintf('\n');
if nsurprise==0
    fprintf('  Every knob behaves as documented. A "no" against "ignored" is a PASS:\n');
    fprintf('  the model derives that quantity instead of accepting it, which is why\n');
    fprintf('  you selected it. sat.reads(cfg) says which before you spend the arcs.\n');
else
    fprintf('  %d knob(s) disagree with the documentation -- investigate: either the\n', nsurprise);
    fprintf('  wiring drops the knob, or the doc is stale. Both have happened here.\n');
end
fprintf('===============================================\n');
