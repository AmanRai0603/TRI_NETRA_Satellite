% ATMOS_DRAG_GOLDEN  Cross-check inputs for force_data/16U_dria_dtm2020_srp-boxwing_erp-boxwing.
%   Rebuilds the TEMPLATE_16U world (sgeom.sat16u, ram attitude, DRIA on DTM2020,
%   manual space weather F10.7 = F10.7a = 90, ap = 8, Kp = 2, epoch 2007-01-01) and,
%   for every row of the golden state.csv, runs op.accel and records the DragInput the
%   Rust port needs (r, v, op.geodetic(r_ecef), utc, doy, omega_eci, Sun, R_bi) with
%   what the CURRENT MATLAB code returns (a, rho, T, nO, Cd, A_proj) next to the
%   golden CSV values. Writes tests/data/atmos_drag_golden.json.
%   Run from matlab_sils/pop after setup_paths:
%     octave-cli --no-gui -q --eval "setup_paths; run('<abs>/refgen/atmos_drag_golden.m')"
1;
function s = jn(x)
    if isnan(x), s = 'null'; else, s = sprintf('%.17g', x); end
end
function s = jv(v)
    v = v(:).'; p = cell(1, numel(v));
    for k = 1:numel(v), p{k} = jn(v(k)); end
    s = ['[' strjoin(p, ',') ']'];
end
function s = jm(M)
    s = ['[' jv(M(1,:)) ',' jv(M(2,:)) ',' jv(M(3,:)) ']'];
end

warning('off', 'all');
here = fileparts(mfilename('fullpath'));
gdir = fullfile(here, '..', '..', '..', '..', 'matlab_sils', 'pop', 'force_data', '16U_dria_dtm2020_srp-boxwing_erp-boxwing');
SC16 = sgeom.sat16u();
SC   = struct('mass', SC16.mass, 'Aref', 0.20*0.20, 'Cd', 2.2, 'Cr', 1.3, 'facets', SC16.facets, 'R_bi', eye(3));
cfg = config.defaultConfig();
cfg.epoch = [2007 1 1 0 0 0];
cfg.tspan = 1800;
cfg.gravityField = struct('field', 'default', 'degree', 20);
cfg.spacecraft = SC; cfg.spacecraft.attitude = 'ram';
cfg.spaceweather.manual = struct('F107', 90, 'F107a', 90, 'ap', 8, 'Kp', 2);
GSI = struct('Tw', 300, 'aT', 0.9, 'sig_n', 0.9, 'sig_t', 0.9);
cfg.forces = struct( ...
  'gravity',    struct('on', true, 'model', 'sphharm', 'degree', 20, 'order', 20), ...
  'drag',       struct('on', true, 'model', 'dria', 'atmos', 'dtm2020', 'corotate', true, 'gsi', GSI), ...
  'thirdbody',  struct('on', true, 'model', 'battin'), ...
  'srp',        struct('on', false), 'erp', struct('on', false), 'relativity', struct('on', false), ...
  'solidtides', struct('on', false), 'oceantides', struct('on', false));
evalc('W = op.buildWorld(cfg);');
S  = csvread(fullfile(gdir, 'state.csv'), 1, 0);
Dg = csvread(fullfile(gdir, 'drag.csv'), 1, 0);
Mg = csvread(fullfile(gdir, 'model_inputs.csv'), 1, 0);
outf = fullfile(here, '..', 'tests', 'data', 'atmos_drag_golden.json');
fid = fopen(outf, 'w');
fprintf(fid, '{"manual":{"F107":90,"F107a":90,"ap":8,"Kp":2},"mass":%s,"Aref":%s,"rows":[\n', jn(SC.mass), jn(SC.Aref));
for k = 1:size(S, 1)
    [aa, parts, ctx, info] = op.accel(S(k,1), S(k,2:4).', S(k,5:7).', W);
    [lat, lon, alt] = op.geodetic(ctx.r_ecef);
    D = info.drag;
    s = ['{"t":' jn(S(k,1)) ',"r":' jv(ctx.r_eci) ',"v":' jv(ctx.v_eci) ',"lat":' jn(lat) ',"lon":' jn(lon) ',"alt":' jn(alt) ...
         ',"utc":' jv(ctx.utc) ',"doy":' jn(ctx.T.doy) ',"omega":' jv(ctx.omega_eci) ',"sun":' jv(ctx.E.sun_eci) ...
         ',"R_bi":' jm(ctx.sc.R_bi) ',"a":' jv(parts.drag) ',"rho":' jn(D.atm.rho) ',"T":' jn(D.atm.T) ...
         ',"nO":' jn(D.atm.n.O) ',"Cd":' jn(D.out.Cd) ',"A_proj":' jn(D.out.A_proj) ...
         ',"golden":{"a":' jv(Dg(k,2:4)) ',"alt_km":' jn(Mg(k,2)) ',"rho":' jn(Mg(k,3)) ',"T":' jn(Mg(k,4)) ...
         ',"nO":' jn(Mg(k,5)) ',"Cd":' jn(Mg(k,6)) ',"A_proj":' jn(Mg(k,7)) '}}'];
    if k < size(S, 1), fprintf(fid, '%s,\n', s); else, fprintf(fid, '%s\n', s); end
end
fprintf(fid, ']}\n');
fclose(fid);
printf('wrote %s\n', outf);
