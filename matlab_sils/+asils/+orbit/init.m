function O = init(P)
%ASILS.ORBIT.INIT  Initialise the in-loop PRECISION ORBIT (POP v51) for one run: the twin's truth orbit, as the engine's
%   adcs-sim run.rs Truth::new steps adcs-pop's port of the same propagator.
%
%   The force set is env's (asils.models.forcemodel.sils_forces with the reflectivity's Cr, sils_cr): gravity, the third
%   bodies, drag and solar pressure as the design states them, here only put in POP's words (op.buildWorld's config). The
%   orbit's start is env's too (forcemodel's sso_initial: a = Re + alt, the RAAN from the LTAN and the DE440 Sun's right
%   ascension at the epoch, the argument of latitude u0). POP itself (matlab_sils/pop: its integrator, its readers and
%   its published models) is the vendored reference the engine's port is held to.
%
%   Returns O with the POP world W, the two bracketing nodes (t0,y0,a0,x0) and (t1,y1,a1,x1) used for dense output, and
%   the per-node context x (Sun, SRP pressure, ECI->ECEF rotation, density) that the environment reads.
%   Owner: Agastya. Copyright (c) 2026 Agastya. All rights reserved.
    o = P.orbit;
    cr = asils.models.forcemodel.sils_cr(P.sc.refl);              % env_force_model: Cr = 1 + the reflectivity
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
    cfg.frame = struct('build', 'gmst', 'dUT1', 0.0, 'data_dir', '');

    % the start: the DE440 Sun at the epoch (POP's reader), the geometry env's
    T  = timeconv.convertUTC(P.epoch_utc(1), P.epoch_utc(2), P.epoch_utc(3), P.epoch_utc(4), P.epoch_utc(5), P.epoch_utc(6), 0);
    E  = ephemInputs(T.tdb_jd);
    [r0, v0, raan] = asils.models.forcemodel.sso_initial(E.sun_unit(:), o.alt_km, o.ecc, o.inc_deg, o.ltan_h, o.argp_deg, o.u0_deg);
    cfg.r0 = r0(:); cfg.v0 = v0(:); cfg.tspan = P.sim.duration_s;

    evalc('W = op.buildWorld(cfg);');
    K = de440.constants();
    O.W = W; O.cfg = cfg; O.h = o.step_s;
    O.raan_rad = raan; O.inc_rad = o.inc_deg*pi/180; O.a_m = K.Re_earth + o.alt_km*1e3;
    O.omega_e = K.omega_earth;
    O.t0 = 0; O.y0 = [r0(:); v0(:)];
    [O.a0, O.x0] = asils.orbit.node(0, O.y0, O.W);
    O = asils.orbit.advance(O);        % fills node 1
    O.n_accel = 4;
end
