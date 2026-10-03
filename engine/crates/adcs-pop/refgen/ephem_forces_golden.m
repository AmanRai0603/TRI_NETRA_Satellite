% EPHEM_FORCES_GOLDEN  Inputs + MATLAB re-evaluation for the golden POP dataset
%   force_data/16U_dria_dtm2020_srp-boxwing_erp-boxwing (TEMPLATE_16U: epoch
%   2007-01-01, 300 km / 96.5 deg, sgeom.sat16u, ram attitude, srp boxwing,
%   erp boxwing 8x16, relativity {'schwarzschild'}, thirdbody battin).
%   For every row of state.csv this rebuilds the op.accel context with the SAME
%   world (op.buildWorld, drag off -- it is not ported here) and prints what the
%   Rust ports need (TDB JD, doy, R_bi, field mu) plus the MATLAB forces from the
%   CSV state and the golden CSV forces themselves.
%   Run from matlab_sils/pop:
%     octave-cli --no-gui -q --eval "setup_paths; run('<abs>/refgen/ephem_forces_golden.m')"
%   Owner: Agastya. Copyright (c) 2026 Agastya. All rights reserved.

here = fileparts(mfilename('fullpath'));
if isempty(here), here = pwd; end
outf = fullfile(here, '..', 'tests', 'data', 'ephem_forces_golden.json');
gdir = fullfile(here, '..', '..', '..', '..', 'matlab_sils', 'pop', 'force_data', '16U_dria_dtm2020_srp-boxwing_erp-boxwing');

J = @(v) ['[' strjoin(arrayfun(@(x) sprintf('%.17g', x), v(:).', 'UniformOutput', false), ',') ']'];
N = @(x) sprintf('%.17g', x);
rd = @(f) dlmread(fullfile(gdir, f), ',', 1, 0);

S  = rd('state.csv');  TB = rd('thirdbody.csv');  SR = rd('srp.csv');
ER = rd('erp.csv');    RL = rd('relativity.csv');

SC16 = sgeom.sat16u();
SC   = struct('mass', SC16.mass, 'Aref', 0.20*0.20, 'Cd', 2.2, 'Cr', 1.3, ...
              'facets', SC16.facets, 'R_bi', eye(3));
cfg = config.defaultConfig();
cfg.epoch = [2007 1 1 0 0 0];
cfg.tspan = 1800;
cfg.gravityField = struct('field', 'default', 'degree', 20);
cfg.spacecraft = SC;  cfg.spacecraft.attitude = 'ram';
cfg.forces = struct( ...
  'gravity',    struct('on',true,  'model','sphharm','degree',20,'order',20), ...
  'drag',       struct('on',false), ...
  'thirdbody',  struct('on',true,  'model','battin'), ...
  'srp',        struct('on',true,  'model','boxwing'), ...
  'erp',        struct('on',true,  'model','boxwing','nrings',8,'nseg',16), ...
  'relativity', struct('on',true,  'terms',{{'schwarzschild'}}), ...
  'solidtides', struct('on',false), 'oceantides', struct('on',false));
W = op.buildWorld(cfg);

fid = fopen(outf, 'w');
fprintf(fid, '{"mu_field":%s,"omega_eci":%s,"rows":[\n', N(W.grav.mu), J(W.omega_eci));
for k = 1:size(S,1)
  t = S(k,1); r = S(k,2:4).'; v = S(k,5:7).';
  [~, parts, ctx] = op.accel(t, r, v, W);
  if k > 1, fprintf(fid, ',\n'); end
  fprintf(fid, ['{"t":%s,"jd_tdb":%s,"doy":%s,"r":%s,"v":%s,"R_bi":%s,' ...
                '"m_thirdbody":%s,"m_srp":%s,"m_erp":%s,"m_relativity":%s,' ...
                '"g_thirdbody":%s,"g_srp":%s,"g_erp":%s,"g_relativity":%s}'], ...
    N(t), N(ctx.T.tdb_jd), N(ctx.T.doy), J(r), J(v), J(ctx.sc.R_bi.'), ...
    J(parts.thirdbody), J(parts.srp), J(parts.erp), J(parts.relativity), ...
    J(TB(k,2:4)), J(SR(k,2:4)), J(ER(k,2:4)), J(RL(k,2:4)));
  printf('row %d t=%g\n', k, t); fflush(stdout);
end
fprintf(fid, '\n]}\n');
fclose(fid);
printf('wrote %s\n', outf);
