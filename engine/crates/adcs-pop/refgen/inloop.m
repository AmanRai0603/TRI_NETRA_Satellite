% In-loop orbit reference: asils.orbit (POP inside) for the scenario configs of both cases.
% Writes tests/data/inloop.txt, one "key v1 v2 ..." line per record, %.17g.
% Run from matlab_sils:  ADCS_POP=<crate dir> octave-cli --eval "startup_asils; run('<this file>')"
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
  w('raan', O.raan_rad); w('y0', O.y0); w('a0', O.a0); w('omega_eci', O.W.omega_eci);
  [a, parts] = op.accel(0, O.y0(1:3), O.y0(4:6), O.W);
  w('p_gravity', parts.gravity); w('p_thirdbody', parts.thirdbody); w('p_drag', parts.drag); w('p_srp', parts.srp);
  ts = [0 3.7 10 600 1800 3000.1 5739];
  for k = 1:numel(ts)
    [r, v, O] = asils.orbit.state(O, ts(k));
    X = asils.orbit.context(O, ts(k));
    w('sample', [ts(k) r(:)' v(:)' X.sun_eci(:)' X.moon_eci(:)' X.P_srp X.rho reshape(X.C', 1, 9)]);
  end
end
fclose(f);
