function O = init(P)
%ASILS.ORBIT.INIT  Initialise the in-loop PRECISION ORBIT for one run: the twin's truth orbit, as the engine's adcs-sim
%   run.rs Truth::new steps adcs-pop's.
%
%   The orbit is env's: the force set (asils.models.forcemodel.sils_forces with the reflectivity's Cr, sils_cr), every
%   force and what it reads (the time scales, DE440, the Earth's frame and rate, the field, the atmosphere and its
%   indices: asils.orbit.accel) and the start (forcemodel's sso_initial: a = Re + alt, the RAAN from the LTAN and the
%   DE440 Sun's right ascension at the epoch, the argument of latitude u0), all generated into +asils/+models. What is
%   written here is the world those calls are handed (the engine's adcs-pop World::new): the epoch, the spacecraft, the
%   run's space weather, the gravity workspace, the Earth's rate. The Octave POP (matlab_sils/pop) is not called: it is
%   the vendored referent the generated models are held to (tests/run_all_tests.m t_orbit_vs_pop).
%
%   Returns O with the world W, the two bracketing nodes (t0,y0,a0,x0) and (t1,y1,a1,x1) used for dense output, and
%   the per-node context x (Sun, SRP pressure, ECI->ECEF rotation, density) that the environment reads.
%   Owner: Agastya. Copyright (c) 2026 Agastya. All rights reserved.
    o = P.orbit;
    cr = asils.models.forcemodel.sils_cr(P.sc.refl);              % env_force_model: Cr = 1 + the reflectivity
    W.fs = asils.models.forcemodel.sils_forces(cr);
    W.epoch = reshape(P.epoch_utc, 6, 1);
    W.sc = struct('mass', P.sc.mass_kg, 'aref', P.sc.aref_m2, 'cd', P.sc.cd, 'cr', cr, 'R_bi', eye(3));
    W = asils.orbit.world(W, P.env);
    K = asils.models.de440.de440_constants();

    % the start: the DE440 Sun at the epoch, the geometry env's
    T  = asils.models.timescales.convert_utc(W.epoch(1), W.epoch(2), W.epoch(3), W.epoch(4), W.epoch(5), W.epoch(6), 0);
    E  = asils.orbit.ephem(T.tdb_jd);
    [r0, v0, raan] = asils.models.forcemodel.sso_initial(E.sun_unit, o.alt_km, o.ecc, o.inc_deg, o.ltan_h, o.argp_deg, o.u0_deg);

    O.W = W; O.h = o.step_s;
    O.raan_rad = raan; O.inc_rad = o.inc_deg*pi/180; O.a_m = asils.relations.orbit.radius(o.alt_km*1e3);
    O.omega_e = K.omega_earth;
    O.t0 = 0; O.y0 = [r0(:); v0(:)];
    [O.a0, O.x0] = asils.orbit.node(0, O.y0, O.W);
    O = asils.orbit.advance(O);        % fills node 1
    O.n_accel = 4;
end
