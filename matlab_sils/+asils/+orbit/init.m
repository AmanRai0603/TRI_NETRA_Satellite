function O = init(P)
%ASILS.ORBIT.INIT  Initialise the in-loop PRECISION ORBIT (POP v51) for one run.
%
%   The SILS truth orbit is no longer read from data (the Standard Code's
%   frozen Orekit CSV / pop_cache ephemeris is gone). It is PROPAGATED here,
%   in the same loop as the attitude, by the Precision Orbit Propagator's own
%   force model (op.buildWorld / op.accel / op.rhs): spherical-harmonic
%   gravity, Sun + Moon third body from DE440, drag with the DTM2020 density
%   and co-rotating atmosphere, cannonball/box-wing SRP with conical eclipse.
%
%   Initial state from the case: altitude, inclination, eccentricity, LTAN
%   (RAAN placed from the DE440 Sun right ascension at the epoch).
%
%   Returns O with the POP world W, the two bracketing nodes (t0,y0,a0,x0) and
%   (t1,y1,a1,x1) used for dense output, and the per-node context x (Sun, SRP
%   pressure, ECI->ECEF rotation, density) that the environment reads.
    o = P.orbit;
    K = de440.constants();
    a = K.Re_earth + o.alt_km*1e3;
    inc = o.inc_deg*pi/180;

    % ---- POP configuration ------------------------------------------------
    cfg = config.defaultConfig();
    cfg.epoch = P.epoch_utc;
    cfg.spacecraft = struct('mass', P.sc.mass_kg, 'Aref', P.sc.aref_m2, ...
        'Cd', P.sc.cd, 'Cr', 1 + P.sc.refl, 'R_bi', eye(3));
    cfg.gravityField = struct('field', 'default', 'degree', o.grav_degree);
    cfg.forces.gravity   = struct('on', true, 'model', 'sphharm', 'degree', o.grav_degree, 'order', o.grav_degree);
    cfg.forces.thirdbody = struct('on', o.thirdbody, 'model', 'battin');
    cfg.forces.drag      = struct('on', o.drag, 'model', 'cannonball', 'atmos', o.atmos, 'corotate', true);
    cfg.forces.srp       = struct('on', o.srp, 'model', 'cannonball', 'eclipse', 'conical', 'Cr', 1 + P.sc.refl);
    cfg.spaceweather.manual = struct('F107', P.env.F107, 'F107a', P.env.F107a, 'Kp', P.env.Kp, 'ap', P.env.ap);
    cfg.frame = struct('build', 'gmst', 'dUT1', 0.0, 'data_dir', '');

    % ---- RAAN from LTAN: Omega = RA_sun + 15 deg/h * (LTAN - 12) -------------
    T  = timeconv.convertUTC(P.epoch_utc(1), P.epoch_utc(2), P.epoch_utc(3), ...
                             P.epoch_utc(4), P.epoch_utc(5), P.epoch_utc(6), 0);
    E  = ephemInputs(T.tdb_jd);
    raSun = atan2(E.sun_unit(2), E.sun_unit(1));
    raan = mod(raSun + (o.ltan_h - 12)*15*pi/180, 2*pi);

    [r0, v0] = op.coe2rv(a, o.ecc, inc, raan, o.argp_deg*pi/180, o.u0_deg*pi/180, K.mu_earth);
    cfg.r0 = r0(:); cfg.v0 = v0(:); cfg.tspan = P.sim.duration_s;

    evalc('W = op.buildWorld(cfg);');
    O.W = W; O.cfg = cfg; O.h = o.step_s;
    O.raan_rad = raan; O.inc_rad = inc; O.a_m = a;
    O.omega_e = K.omega_earth;
    O.t0 = 0; O.y0 = [r0(:); v0(:)];
    [O.a0, O.x0] = asils.orbit.node(0, O.y0, O.W);
    O = asils.orbit.advance(O);        % fills node 1
    O.n_accel = 4;
end
