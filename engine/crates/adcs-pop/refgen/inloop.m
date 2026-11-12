% In-loop orbit reference: asils.orbit (POP inside) for the scenario configs of both cases.
% Writes tests/data/inloop.txt, one "key v1 v2 ..." line per record, %.17g.
% The state, the Sun, the Moon, the density and the frame are asils.orbit's (the twin's generated models); the per-force parts at the
% start are the vendored Octave POP's own op.accel on the same force set and Earth frame (the referent).
% Run from matlab_sils:  ADCS_POP=<crate dir> octave-cli --eval "startup_asils; addpath(fullfile(pwd,'pop')); evalc('setup_paths();'); run('<this file>')"
1;
function cfg = pop_cfg_(P, y0)
    % the run's force set, start and Earth frame in POP's words (as matlab_sils/tests/run_all_tests.m pop_cfg_)
    cr = asils.models.forcemodel.sils_cr(P.sc.refl);
    fs = asils.models.forcemodel.sils_forces(cr);
    cfg = config.defaultConfig();
    cfg.epoch = P.epoch_utc;
    cfg.spacecraft = struct('mass', P.sc.mass_kg, 'Aref', P.sc.aref_m2, 'Cd', P.sc.cd, 'Cr', cr, 'R_bi', eye(3));
    cfg.gravityField = struct('field', 'default', 'degree', fs.grav_degree);
    gm = {'twobody', 'zonal', 'sphharm'}; tb = {'battin', 'direct', 'tidal', 'legendre'};
    at = {'exponential', 'nrlmsise', 'jb2008', 'dtm2020', 'dtm2020_research'}; ec = {'cylindrical', 'conical', 'fine'};
    dm = {'cannonball', 'panel'}; sm = {'cannonball', 'boxwing'};
    cfg.forces.gravity   = struct('on', true, 'model', gm{fs.grav_model + 1}, 'degree', fs.grav_degree, 'order', fs.grav_order);
    cfg.forces.thirdbody = struct('on', fs.tb_on, 'model', tb{fs.tb_model + 1});
    cfg.forces.drag      = struct('on', fs.drag_on, 'model', dm{fs.drag_model + 1}, 'atmos', at{fs.drag_atmos + 1}, 'corotate', fs.drag_corotate);
    cfg.forces.srp       = struct('on', fs.srp_on, 'model', sm{fs.srp_model + 1}, 'eclipse', ec{fs.srp_eclipse + 1}, 'Cr', fs.srp_cr);
    cfg.spaceweather.manual = struct('F107', P.env.F107, 'F107a', P.env.F107a, 'Kp', P.env.Kp, 'ap', P.env.ap);
    if P.env.frame_build == 1
        cfg.frame = struct('build', 'A', 'dUT1', 0.0, 'data_dir', fullfile(asils.util.root(), '..', 'engine', 'crates', 'adcs-pop', 'data', 'time_frames'));
    else
        cfg.frame = struct('build', 'gmst', 'dUT1', 0.0, 'data_dir', '');
    end
    cfg.r0 = y0(1:3); cfg.v0 = y0(4:6); cfg.tspan = P.sim.duration_s;
end

f = fopen(fullfile(getenv('ADCS_POP'), 'tests', 'data', 'inloop.txt'), 'w');
w = @(k, x) fprintf(f, '%s%s\n', k, sprintf(' %.17g', x));
cases = {{'detumble_ais', 'cases/ais_3u.csv'}, {'fine_hold_img', 'cases/ais_img_3u.csv'}};
for c = 1:numel(cases)
  P = asils.config(cases{c}{1}, cases{c}{2});
  O = asils.orbit.init(P);
  fprintf(f, 'case %s\n', cases{c}{1});
  w('epoch', P.epoch_utc); w('alt_km', P.orbit.alt_km); w('inc_deg', P.orbit.inc_deg); w('ecc', P.orbit.ecc);
  w('ltan_h', P.orbit.ltan_h); w('u0_deg', P.orbit.u0_deg); w('argp_deg', P.orbit.argp_deg);
  w('mass', P.sc.mass_kg); w('aref', P.sc.aref_m2); w('cd', P.sc.cd); w('cr', 1 + P.sc.refl);
  w('sw', [P.env.F107 P.env.F107a P.env.Kp P.env.ap]); w('step', P.orbit.step_s);
  w('frame_build', P.env.frame_build);   % env_earth_frame_build: 0 gmst, 1 build A with the EOP snapshot of the design
  w('raan', O.raan_rad); w('y0', O.y0); w('a0', O.a0); w('omega_eci', O.W.omega_eci);
  pcfg = pop_cfg_(P, O.y0);
  evalc('PW = op.buildWorld(pcfg);');
  [a, parts] = op.accel(0, O.y0(1:3), O.y0(4:6), PW);
  w('p_gravity', parts.gravity); w('p_thirdbody', parts.thirdbody); w('p_drag', parts.drag); w('p_srp', parts.srp);
  ts = [0 3.7 10 600 1800 3000.1 5739];
  for k = 1:numel(ts)
    [r, v, O] = asils.orbit.state(O, ts(k));
    X = asils.orbit.context(O, ts(k));
    w('sample', [ts(k) r(:)' v(:)' X.sun_eci(:)' X.moon_eci(:)' X.P_srp X.rho reshape(X.C', 1, 9)]);
  end
end
fclose(f);
