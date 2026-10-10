function ok = run_all_tests()
%RUN_ALL_TESTS  Unit and integration tests of the TRI-NETRA ADCS SILS.
%   >> startup_asils; addpath tests; run_all_tests
%   Each test prints PASS/FAIL with the number it checked. Each runs from its own seed
%   (rng(1000 + its position)), so a random test draws the same numbers every time.
%   The models and the flight software are the design's (+asils/+models, +asils/+alg, generated):
%   these tests hold the twin's runner around them (the tick, the bus, the units, the set-up, the
%   metrics, the campaigns) and the physics they give, and the twin against the engine value for value.
%   Copyright (c) 2026 Agastya. All rights reserved.
    T = {@t_quat, @t_kinematics, @t_sso_start, @t_case, @t_igrf, @t_shadow, ...
         @t_torques, @t_plant_conservation, @t_cmg_plant, @t_mekf, @t_quest, @t_lqr_blob, ...
         @t_fmr_spin_down, @t_cmg_steering, @t_hal_link, @t_hal_codec, @t_select, @t_gen_bdot, @t_sun_spin_law, @t_sun_guidance, ...
         @t_sun_model, @t_st_chain, @t_sun_chain, @t_blob_decode, ...
         @t_sizing, @t_modes_table, @t_orbit_vs_pop, @t_short_runs, @t_campaign_draw, @t_metrics_evaluate, @t_flex_plant, ...
         @t_solution_scenario, @t_coil_lag, @t_wheel_motor, @t_st_moon_blind, @t_gps_latency, @t_earth_radiation, ...
         @t_fidelity_refused, @t_chains_from_part, @t_streams, @t_checkpoint, @t_engine_parity, @t_physics_vectors, ...
         @test_trinetra_open};
    n = 0;
    for i = 1:numel(T)
        name = func2str(T{i});
        rng(1000 + i);                       % every test seeded: a failure replays exactly
        try
            msg = T{i}();
            fprintf('PASS  %-24s %s\n', name, msg); n = n + 1;
        catch e
            fprintf('FAIL  %-24s %s\n', name, e.message);
        end
    end
    ok = n == numel(T);
    fprintf('%d / %d tests passed\n', n, numel(T));
end

function m = t_quat()
    a = asils.quat.norm(randn(4,1)); b = asils.quat.norm(randn(4,1));
    e = norm(asils.quat.dcm(asils.quat.mult(a,b)) - asils.quat.dcm(b)*asils.quat.dcm(a));
    q = asils.quat.fromdcm(asils.quat.dcm(a)); e2 = min(norm(q - a), norm(q + a));
    assert(e < 1e-12 && e2 < 1e-12, 'composition %.2e, dcm round trip %.2e', e, e2);
    e3 = norm(asils.la.dcm(a) - asils.alg.math.dcm(a));
    assert(e3 == 0, 'the toolbox''s dcm is the flight algorithms'' (%.2e)', e3);
    m = sprintf('composition %.1e, round trip %.1e; la = alg dcm', e, e2);
end

function B = body_(I, geo)
    if nargin < 2, geo = asils.models.rotors.rotor_geometry(zeros(8, 3), zeros(4, 3), zeros(8, 1), 0, 0); end
    B = asils.plant.body(I, geo, []);
end

function x = state_(q, w)
    x = asils.models.rigidbody.PlantState_zero(); x.q = q; x.w = w;
end

function m = t_kinematics()
    % inertially fixed vector seen from a body spinning at w: v_B' = -w x v_B
    q = asils.quat.norm(randn(4,1)); w = [0.01; -0.02; 0.03]; dt = 1e-3; v = [1; 2; 3];
    x1 = asils.plant.step(state_(q, w), dt, body_(eye(3)), zeros(3,1), zeros(8,1), zeros(4,1));
    d_num = (asils.la.dcm(x1.q)*v - asils.la.dcm(q)*v)/dt;
    d_ana = -cross(w, asils.la.dcm(q)*v);
    assert(norm(d_num - d_ana) < 1e-4, 'mismatch %.2e', norm(d_num - d_ana));
    m = sprintf('dv/dt error %.1e', norm(d_num - d_ana));
end

function m = t_sso_start()
    % env's sso_initial: the ascending node sits at the LTAN's angle from the Sun's right ascension
    s = [cosd(30); sind(30); 0];
    [r, v, raan] = asils.models.forcemodel.sso_initial(s, 550, 0, 97.6, 10.5, 0, 0);
    want = mod(30 + (10.5 - 12)*15, 360);
    assert(abs(raan*180/pi - want) < 1e-9, 'RAAN %.6f deg vs %.6f', raan*180/pi, want);
    assert(abs(norm(r) - 6378137 - 550e3) < 1e-3 && abs(acosd(dot(cross(r, v), [0;0;1])/norm(cross(r, v))) - 97.6) < 1e-9, 'a, i');
    m = sprintf('RAAN %.2f deg at LTAN 10:30, a and i as asked', raan*180/pi);
end

function m = t_case()
    c1 = asils.case.read(fullfile(asils.util.root(), 'cases', 'ais_3u.csv'));
    c2 = asils.case.read(fullfile(asils.util.root(), 'cases', 'ais_img_3u.csv'));
    assert(c1.v.orbit_alt == 550 && c1.v.orbit_ltan == 6 && c2.v.orbit_ltan == 10, 'orbit keys');
    assert(c1.v.req_ape == 10 && abs(c2.v.req_ape - 0.01) < 1e-12, 'requirement keys');
    f = [tempname '.csv']; fid = fopen(f, 'w');
    fprintf(fid, 'section,key,label,unit,value,lo,hi,level,note\nmeta,meta.schema,,,adcs-case/9,,,,\n'); fclose(fid);
    refused = false; try, asils.case.read(f); catch, refused = true; end
    delete(f); assert(refused, 'a wrong schema was not refused');
    m = 'both cases read; wrong schema refused';
end

function m = t_igrf()
    gh = asils.models.frames.igrf_gh(2025.0);
    B0 = norm(asils.models.frames.igrf_ned(gh, 0, 0, 0, 13));
    Bp = norm(asils.models.frames.igrf_ned(gh, 80*pi/180, 0, 0, 13));
    B5 = norm(asils.models.frames.igrf_ned(gh, 0, 0, 550, 13));
    assert(B0 > 25000 && B0 < 40000 && Bp > 50000 && Bp < 62000 && B5 < B0, ...
        '|B| equator %.0f, 80N %.0f, 550 km %.0f nT', B0, Bp, B5);
    g2 = asils.alg.frames.igrf_gh(2025.0);
    assert(isequal(gh, g2), 'the truth''s and the flight software''s IGRF coefficients are one table');
    m = sprintf('|B| equator %.0f nT, 80N %.0f nT, 550 km %.0f nT', B0, Bp, B5);
end

function m = t_shadow()
    s = [1.496e11; 0; 0]; f = @asils.models.shadow.shadow_fraction;
    a = f([7e6; 0; 0], s); b = f([-7e6; 0; 0], s); c = f([0; 7e6; 0], s);
    assert(a == 1 && b == 0 && c == 1, 'nu = %g %g %g', a, b, c);
    m = 'sunlit 1, umbra 0, terminator side 1';
end

function G = facets_(box, cm)
    G = asils.models.facets.facets_box(box(:), cm(:), 0.8, 0.8, 0.05, 0.6, 0.5);
end

function m = t_torques()
    G = facets_([0.34 0.1 0.1], [0;0;0]); I = diag([0.0067 0.042 0.042]); q = [0;0;0;1];
    p = asils.env.torques(q, [7e6;0;0], [0;7500;0], [2e-5;0;0], [1.5e11;0;0], 1, 4.5e-6, 1e-12, I, G, [0;0;0], 3.986e14, [1 1 1 1]);
    assert(norm(p(:,1)) < 1e-15, 'GG on a principal axis %.2e', norm(p(:,1)));
    assert(norm(p(:,2)) < 1e-15 && norm(p(:,3)) < 1e-15, 'symmetric box about its CM must see no aero/SRP torque');
    G = facets_([0.34 0.1 0.1], [0; 0; 0.02]);
    p = asils.env.torques(q, [7e6;0;0], [0;7500;0], [2e-5;0;0], [1.5e11;0;0], 1, 4.5e-6, 1e-12, I, G, [0;0;0], 3.986e14, [1 1 1 1]);
    assert(norm(p(:,2)) > 0, 'offset CM must give aero torque');
    m = sprintf('GG/aero/SRP symmetry held; aero with 2 cm offset %.2e N m', norm(p(:,2)));
end

function [geo, x] = wheels_()
    a0 = zeros(8, 3); a0(1:3, :) = eye(3);
    geo = asils.models.rotors.rotor_geometry(a0, zeros(4, 3), zeros(8, 1), 3, 0);
    x = asils.models.rigidbody.PlantState_zero();
end

function m = t_plant_conservation()
    I = diag([0.0067 0.042 0.043]); [geo, x] = wheels_(); B = body_(I, geo);
    x.q = asils.quat.norm(randn(4,1)); x.w = 0.1*randn(3,1); x.h(1:3) = 1e-3*randn(3,1);
    Hi = @(x) asils.la.dcm(x.q).'*asils.models.momentum.total_momentum(x, I, geo, B.flex);
    H0 = Hi(x);
    for k = 1:1000, x = asils.plant.step(x, 0.1, B, zeros(3,1), zeros(8,1), zeros(4,1)); end
    e = norm(Hi(x) - H0)/norm(H0);
    assert(e < 1e-6, 'inertial momentum drift %.2e', e);
    m = sprintf('100 s torque-free: inertial |dH|/|H| = %.1e', e);
end

function m = t_cmg_plant()
    % gimballing a CMG pyramid exchanges momentum with the body: the INERTIAL total momentum stays constant
    pr = asils.product.load('TRN-P-3U-CMG'); X = pr.mex.rec;
    geo = asils.models.rotors.rotor_geometry(X.a0, X.g, X.gi, X.n, X.ng); I = diag([0.0067 0.042 0.043]); B = body_(I, geo);
    x = asils.models.rigidbody.PlantState_zero(); x.q = asils.quat.norm(randn(4,1)); x.h = X.h0;
    Hi = @(x) asils.la.dcm(x.q).'*asils.models.momentum.total_momentum(x, I, geo, B.flex);
    H0 = Hi(x);
    for k = 1:200, x = asils.plant.step(x, 0.05, B, zeros(3,1), zeros(8,1), [0.3; -0.2; 0.1; 0.4]); end
    e = norm(Hi(x) - H0); w = norm(x.w);
    assert(e < 1e-9 && w > 1e-3, 'dH %.2e, body rate %.2e', e, w);
    m = sprintf('10 s of gimballing: |dH_inertial| = %.1e, body rate %.3f deg/s', e, w*180/pi);
end

function m = t_mekf()
    % the flight software's MEKF (fsw_estimation: mekf_init, mekf_predict, mekf_vector) on two noisy vectors
    q = asils.quat.norm(randn(4,1)); [qk, b, P] = asils.alg.estimation.mekf_init(asils.quat.norm(q + [0.05;0;0;0]), 0.1, 1e-3);
    r1 = [1;0;0]; r2 = [0;0.6;0.8];
    for k = 1:200
        [qk, P] = asils.alg.estimation.mekf_predict(qk, b, P, 1e-5, 1e-8, [0;0;0], 0.1);
        [qk, b, P] = asils.alg.estimation.mekf_vector(qk, b, P, asils.la.dcm(q)*r1 + 1e-3*randn(3,1), r1, 1e-3, 0);
        [qk, b, P] = asils.alg.estimation.mekf_vector(qk, b, P, asils.la.dcm(q)*r2 + 1e-3*randn(3,1), r2, 1e-3, 0);
    end
    e = asils.quat.angle(qk, q)*180/pi;
    assert(e < 0.05, 'MEKF error %.3f deg', e);
    m = sprintf('two-vector MEKF converged to %.4f deg', e);
end

function m = t_quest()
    q = asils.quat.norm(randn(4,1)); r = randn(8, 3); r = r./sqrt(sum(r.^2, 2));
    b = (asils.la.dcm(q)*r.').';
    bv = zeros(16, 3); rv = bv; bv(1:8, :) = b; rv(1:8, :) = r; w = zeros(16, 1); w(1:8) = 1;
    qe = asils.alg.estimation.quest(bv, rv, w, 8); e = asils.quat.angle(q, qe);
    assert(e < 1e-9, 'QUEST error %.2e rad', e);
    m = sprintf('q-method on 8 exact stars: %.1e rad', e);
end

function m = t_lqr_blob()
    % the LQR gains the engine puts in the blob (its Riccati solve of fswrw's weights) stabilise each axis
    P = asils.config('fine_hold_img_lqr', 'cases/ais_img_3u.csv', struct()); p = P.fsw; I = diag(p.J);
    worst = -Inf;
    for ax = 1:3
        A = [0 1 0; 0 0 1; 0 0 0] - [0; 0; 1/I(ax)]*p.rw_Klqr(ax, :);
        worst = max(worst, max(real(eig(A))));
    end
    assert(p.rw_law == 1 && worst < 0, 'law %d, worst closed-loop pole %.3g', p.rw_law, worst);
    m = sprintf('blob LQR law, every axis stable (slowest pole %.3g 1/s)', worst);
end

function m = t_fmr_spin_down()
    pr = asils.product.load('TRN-P-3U-FMR'); X = pr.mex.rec;
    assert(abs(X.t_sd(1) - 0.755) < 0.01, 'T_sd %.3f s', X.t_sd(1));
    assert(abs(X.h_max(1) - 1.0e-3) < 5e-5, 'h_max %.2e', X.h_max(1));
    m = sprintf('ring spin-down %.3f s (IDMAS 0.75 s), h_max %.2e N m s', X.t_sd(1), X.h_max(1));
end

function m = t_cmg_steering()
    pr = asils.product.load('TRN-P-3U-CMG'); X = pr.mex.rec; d = [0.1; -0.2; 0.3; 0.05];
    a = asils.alg.allocation.rotor_axes(X.n, X.a0, X.gi, X.g, d);
    tau = [1e-4; -2e-4; 5e-5]; h = zeros(8, 1); h(1:4) = X.h0(1:4);
    [gd, ~] = asils.alg.allocation.steer_sr(tau, a, h, X.n, X.ng, X.gi, X.g, 1.5, 1e-12, 10, false);
    got = zeros(3,1);
    for i = 1:4, got = got - h(i)*gd(i)*cross(X.g(X.gi(i), :).', a(:, i)); end
    e = norm(got - tau)/norm(tau);
    assert(e < 1e-3, 'steering torque error %.2e', e);
    m = sprintf('SR steering delivers the torque to %.1e relative', e);
end

function m = t_hal_link()
    % the OILS/HILS link's frames carry the bus whole: sensors (registers, UART bytes, CAN frames) and commands
    B = asils.hal.bus(); B.now_ns = 123456789012; B.mag = (1:7)'; B.gyro = (11:23)'; B.sun = (31:37)'; B.es = [];
    B.uart{2} = 1:40; B.uart{3} = 50:70; B.can_rx = [512, 8, 1:8; 513, 8, 9:16];
    B2 = asils.hal.link('unpack_sensors', asils.hal.link('pack_sensors', B), asils.hal.bus());
    assert(B2.now_ns == B.now_ns && isequal(B2.mag, B.mag) && isequal(B2.gyro, B.gyro) && isequal(B2.sun, B.sun) && isempty(B2.es), 'registers');
    assert(isequal(B2.uart{2}, B.uart{2}) && isequal(B2.uart{3}, B.uart{3}) && isequal(B2.can_rx, B.can_rx), 'UART and CAN');
    B.pwm = [-32767; 5; 32767; 0; 0; 0; 0; 1]; B.can_tx = [256, 2, 255, 127, 0 0 0 0 0 0; 768, 8, 1:6, 0, 0];
    B3 = asils.hal.link('unpack_commands', asils.hal.link('pack_commands', B), asils.hal.bus());
    assert(isequal(B3.pwm, B.pwm) && isequal(B3.can_tx, B.can_tx), 'commands');
    m = 'sensor and command frames round-trip byte for byte';
end

function m = t_hal_codec()
    % the emulators' bytes are what the flight drivers read: a value through emucodec, the framing and drv_read
    s = asils.models.emucodec.emu_scale();
    b = [2e-5; -1.234e-5; 3.3e-5]; w = [0.01; -0.02; 0.003]; q = asils.quat.norm([0.1; -0.2; 0.3; 0.9]);
    r = [6.9e6; 1.2e5; -3e4]; v = [10; 7600; 5];
    st = zeros(96, 1); f = asils.hal.st_frame([true; false], [q.'; 0 0 0 1], 2, s); st(1:numel(f)) = f;
    gp = zeros(96, 1); f2 = asils.hal.gps_frame(true, r, v, s); gp(1:numel(f2)) = f2;
    tm = asils.hal.rotor_tm(0, 1.234e-3, 0, s);
    [mok, bm, gok, wm, ~, ~, ~, ~, sok, sv, qs, pok, rr, vv, h] = asils.alg.drivers.drv_read( ...
        asils.hal.regs(true, asils.models.emucodec.mag_counts(b, s)), asils.hal.gyro_resp(true, w, s), zeros(7,1), zeros(7,1), ...
        st, numel(f), gp, numel(f2), [tm(1); zeros(7,1)], [8; zeros(7,1)], [tm(3:10); zeros(7,8)], 1, true, false, false, true, true, 1, zeros(8,1));
    assert(mok && gok && sok && pok && sv(1) && ~sv(2), 'flags');
    assert(max(abs(bm - b)) <= s.mag && max(abs(wm - w)) <= s.gyro && max(abs(qs(1, :).' - q)) <= s.q, 'values within one count');
    assert(max(abs(rr - r)) <= s.pos && max(abs(vv - v)) <= s.vel && abs(h(1) - 1.234e-3) <= s.h, 'GNSS and rotor');
    pl = double(f(4:end-2)); c = asils.hal.crc16(pl); pad = zeros(64, 1); pad(1:numel(pl)) = pl;
    assert(c == asils.alg.drivers.crc16(pad, numel(pl)), 'the rig''s CRC is the drivers''');
    m = 'every value within one count through the drivers; the CRCs agree';
end

function m = t_select()
%T_SELECT  The registry resolves every slot and refuses hardware it cannot fly.
    P = asils.config('fine_hold_img', 'cases/ais_img_3u.csv', struct('seed', 1));
    a = P.fsw.alg;
    assert(strcmp(a.pointing, 'pid') && strcmp(a.allocation, 'rotor_pinv') && strcmp(a.detumble, 'bdot_gyro'), 'defaults');
    P = asils.config('fine_hold_cmg', 'cases/ais_img_3u.csv', struct('seed', 1));
    assert(strcmp(P.fsw.alg.allocation, 'cmg_sr'), 'CMG allocation %s', P.fsw.alg.allocation);
    refused = false;
    try
        asils.config('nadir_hold_ais', 'cases/ais_3u.csv', struct('seed', 1, 'set', struct('fsw__algorithms__pointing', 'pid')));
    catch e
        refused = ~isempty(strfind(e.message, 'cannot fly'));
    end
    assert(refused, 'a wheel law on a coils-only bus was not refused');
    m = 'defaults per product, CMG steering picked, wheel law refused on coils-only bus';
end

function m = t_gen_bdot()
%T_GEN_BDOT  L1 with w_d = 0 is plain B-dot; with w_d it drives the rate to w_d.
    B = [2e-5; -1e-5; 3e-5]; w = [0.05; -0.02; 0.1];
    m0 = asils.alg.control.gen_bdot(B, -cross(w, B), zeros(3,1), 1e6);
    assert(dot(cross(m0, B), w) < 0, 'plain B-dot does not remove energy');
    wd = [0; 0; 0.1];
    m1 = asils.alg.control.gen_bdot(B, -cross(w, B), wd, 1e6);
    assert(dot(cross(m1, B), w - wd) < 0, 'L1 does not drive toward w_d');
    m = sprintf('dV/dt < 0 for w_d = 0 and w_d = %.2f rad/s e_z', wd(3));
end

function m = t_sun_spin_law()
%T_SUN_SPIN_LAW  L2 (He et al.): the commanded torque never raises the Lyapunov value; E1 turns the coils off in eclipse.
    J = diag([0.0067 0.042 0.042]); k1 = 0.01; k2 = 0.05; spin = 6;
    worst = -Inf;
    for k = 1:200
        B = 3e-5*randn(3,1); w = 0.1*randn(3,1); s = randn(3,1); s = s/norm(s);
        m0 = asils.alg.control.sun_spin(B, w, s, false, J, spin, k1, k2, 0);
        ws = -spin*pi/180; sg = sign(w(3));
        A = cross(B, k1*(J*w - sg*J(3,3)*ws*s) + k2*diag([J(3,3)-J(1,1), J(3,3)-J(2,2), 0])*w);
        worst = max(worst, dot(A, m0));
    end
    assert(worst <= 1e-20, 'A.m0 = %.2e > 0', worst);
    assert(~any(asils.alg.control.sun_spin([1e-5;0;0], [0;0;0.1], [0;0;-1], true, J, spin, k1, k2, 0)), 'E1 eclipse');
    m = sprintf('max A.m0 = %.1e over 200 draws; E1 coils off', worst);
end

function m = t_sun_guidance()
%T_SUN_GUIDANCE  Sun referencing puts the power face on the Sun and the roll axis on the orbit normal across the Sun line.
    se = [0.3;0.9;0.2]; r = [7e6;0;0]; v = [0;7.5e3;0];
    q = asils.alg.guidance.guidance(4, r, v, 0, [0;0;0;1], 0, 0, 1, zeros(3,1), [0;0;0;1], [0;0;-1], [0;1;0], se, false);
    R = asils.la.dcm(q);
    s = se/norm(se); n = cross(r, v); n = n/norm(n); e = n - (n'*s)*s; e = e/norm(e);
    ang = @(a, b) atan2d(norm(cross(a, b)), a'*b);           % the angle, well conditioned near zero
    e1 = ang([0; 0; -1], R*s); e2 = ang([0; 1; 0], R*e);
    assert(e1 < 1e-6 && e2 < 1e-6, 'Sun %.2e deg, roll %.2e deg', e1, e2);
    m = sprintf('power face on the Sun %.0e deg, roll axis %.0e deg', e1, e2);
end

function m = t_sun_model()
%T_SUN_MODEL  The onboard Sun (precessed to J2000) agrees with DE440 to 0.01 deg.
    P = asils.config('fine_hold_img', 'cases/ais_img_3u.csv', struct()); O = asils.orbit.init(P);
    X = asils.orbit.context(O, 0); [r, ~] = asils.orbit.state(O, 0);
    s = X.sun_eci - r; e = acosd(asils.alg.frames.sun_model(P.jd0)'*s/norm(s));
    assert(e < 0.01, 'onboard Sun off by %.4f deg', e);
    m = sprintf('onboard Sun vs DE440 %.4f deg', e);
end

function [cr, cm, cam, W] = image_chain_(npx)
    ns = 4000; cr = zeros(ns, 3); cm = zeros(ns, 1);
    [~, cr, cm] = asils.models.starcat.star_catalogue(cr, cm);
    cam = asils.models.strender.StCamera_zero();
    cam.n = npx; cam.fov = 0.17; cam.psf_px = 1.2; cam.flux0 = 3000; cam.bg = 50; cam.read_noise = 8; cam.k_sigma = 5;
    cam.max_spots = 20; cam.id_tol = 2e-4; cam.mag_tol = 0.25; cam.fit_tol = 1e-4;
    [cam.f, cam.c] = asils.models.strender.st_focal(cam.fov, npx);
    [np, cr] = asils.models.stidentify.st_pair_count(cr, ns, cam.fov);
    [~, cr, pi_, pj, pa] = asils.models.stidentify.st_pairs(cr, ns, cam.fov, zeros(np, 1), zeros(np, 1), zeros(np, 1), zeros(np, 1), zeros(np, 1), zeros(np, 1));
    nn = npx*npx;
    W = struct('img', zeros(nn, 1), 'work', zeros(nn, 1), 'ia', zeros(nn, 1), 'ib', zeros(nn, 1), 'pi', pi_, 'pj', pj, 'pa', pa, 'np', np);
end

function m = t_st_chain()
%T_ST_CHAIN  The star tracker's chain on rendered frames (sens's st_chain): every frame solved within 60 arcsec.
    [cr, cm, cam, W] = image_chain_(1024);
    assert(W.np == 452404, 'pair table %d', W.np);
    g = asils.devices.stream(3, 'test'); E = zeros(1, 4);
    for k = 1:4
        q = asils.quat.norm(randn(4,1));
        [qm, ok, ~, ~, ~, cr, cm, W.img, W.work, W.ia, W.ib, W.pi, W.pj, W.pa, g] = asils.models.stimage.st_chain(q, eye(3), eye(3), ...
            cr, cm, 4000, cam, W.img, W.work, W.ia, W.ib, W.pi, W.pj, W.pa, true, g);
        assert(ok, 'frame %d not solved', k);
        E(k) = asils.quat.angle(q, qm)*180/pi*3600;
    end
    assert(max(E) < 60, 'worst frame %.1f arcsec', max(E));
    m = sprintf('4/4 frames solved, worst %.1f arcsec', max(E));
end

function m = t_sun_chain()
%T_SUN_CHAIN  Quadrant Sun sensor (sens's quad_currents, quad_angles): currents -> angles within 0.5 deg over +/-30 deg.
    p = asils.models.sunquad.SunHead_zero(); p.a = 1e-3; p.h = 0.6e-3; p.noise = 0; p.min_frac = 0.05;
    g = asils.devices.stream(1, 'test'); E = [];
    for a = -30:10:30
        for b = -30:15:30
            s = [tand(a); tand(b); 1]; s = s/norm(s);
            [i, g] = asils.models.sunquad.quad_currents(s, p, g);
            [sh, ok] = asils.models.sunangles.quad_angles(i, p);
            assert(ok, 'no Sun at %g, %g', a, b); E(end+1) = acosd(min(1, s'*sh)); %#ok<AGROW>
        end
    end
    assert(max(E) < 0.5, 'worst %.3f deg', max(E));
    m = sprintf('noise-free worst %.2e deg over +/-30 deg', max(E));
end

function m = t_blob_decode()
%T_BLOB_DECODE  The engine's blob (adcs params) decodes field by field; a changed byte fails its CRC.
    P = asils.config('fine_hold_img', 'cases/ais_img_3u.csv', struct()); b = P.blob; p = P.fsw;
    assert(numel(b) == 2249 && p.dt == 0.1 && p.nr == 3 && abs(p.jd0 - P.jd0) < 1e-9 && isequal(p.rot_a0(1:3, :), eye(3)), 'fields');
    b(100) = bitxor(b(100), 1); refused = false;
    try, asils.fsw.params_decode(b); catch e, refused = ~isempty(strfind(e.message, 'CRC')); end
    assert(refused, 'a changed byte was not refused');
    m = sprintf('%d-byte blob decoded (dt %.1f s, %d rotors); a flipped bit refused', numel(b), p.dt, p.nr);
end

function m = t_sizing()
%T_SIZING  The sizing (the engine's adcs-design over the design's laws): the catalogue's lightest rotor that meets the
%   need, an authority knob raising the need, the largest fitted with its gap when none meets it, the VSCMG's rotor at
%   half momentum, the spare ring holding any one ring's momentum on its axis, a skewed ring's section.
    k = asils.sizing.knobs();
    Dm = struct('case', 't', 'h_req', 1.5e-3, 'tau_req', 5e-5);
    p = asils.sizing.rotor(Dm, k, 'rw');
    assert(strcmp(p.part_number, 'CAT-CUBESPACE-CUBEWHEEL-CW0017') && isempty(p.sizing.gap), 'rw pick %s', p.part_number);
    Dm.h_req = 5e-3;
    p = asils.sizing.rotor(Dm, k, 'rw');
    assert(strcmp(p.part_number, 'CAT-CUBESPACE-CUBEWHEEL-CW0057'), 'rw pick at 5 mNms %s', p.part_number);
    Dm.h_req = 1.5e-3;
    k4 = asils.sizing.knobs(struct('scale', struct('rw', 4)));
    p = asils.sizing.rotor(Dm, k4, 'rw');
    assert(strcmp(p.part_number, 'CAT-ROCKET-LAB-RW-0-01'), 'rw pick with scale 4 %s', p.part_number);
    Dm.h_req = 10;
    c = asils.sizing.rotor(Dm, k, 'cmg');
    assert(ischar(c.sizing.gap), 'no cmg meets 10 N m s: the gap is said');
    v = asils.sizing.rotor(struct('case', 't', 'h_req', 1e-3, 'tau_req', 5e-5), k, 'vscmg');
    assert(abs(v.nominal.rotor_momentum_Nms - v.nominal.h_max_Nms/2) < 1e-12 && ~isempty(strfind(v.part_number, '-VSCMG')), 'vscmg');
    Dm = struct('case', 't', 'h_req', 2e-3, 'tau_req', 6e-5);
    r = asils.sizing.fmr(Dm, asils.sizing.knobs(struct('fmr_spare', true)), [0.34 0.10 0.10]);
    assert(numel(r) == 4, 'the spare ring');
    ax = asils.models.sizering.spare_axis();
    for i = 1:3
        assert(abs(r{4}.sizing.h_ring_Nms*ax(i) - r{i}.sizing.h_ring_Nms) < 1e-12, 'the spare holds ring %d on its axis', i);
    end
    assert(numel(asils.sizing.fmr(Dm, k, [0.34 0.10 0.10])) == 3, 'no spare unless the knob asks');
    [a, pe] = asils.models.sizering.ring_section([1; 1; 1], ax);
    s = 1/sqrt(2);
    assert(abs(a - 1.5*sqrt(3)*s*s) < 1e-9 && abs(pe - 6*s) < 1e-9, 'a cube cut normal to its diagonal: the regular hexagon');
    m = sprintf('rw CW0017 / CW0057 / RL RW-0.01 (x4), cmg gap, vscmg half, spare ring %.3g N m s', r{4}.sizing.h_ring_Nms);
end

function m = t_modes_table()
%T_MODES_TABLE  Every mission mode option names a controller state the FSW has.
    T = asils.fsw.modes(); n = 0;
    for id = asils.solution.mode()
        M = asils.solution.mode(id{1});
        for i = 1:numel(M.options)
            assert(any(strcmp(T.state, M.options{i}.fsw_mode)), '%s/%s: unknown state %s', id{1}, M.options{i}.id, M.options{i}.fsw_mode);
            n = n + 1;
        end
    end
    m = sprintf('%d options over %d modes map to FSW states', n, numel(asils.solution.mode()));
end

function m = t_orbit_vs_pop()
    % The in-loop precision orbit (env's generated models, asils.orbit.accel) against the vendored POP, its referent:
    % the force model at the run's states (to the last bits: POP takes the Sun's distance by norm, the design by the
    % square root of the dot product), and RK4 + Hermite against POP's own batch propagator (rk78).
    pop_path_();
    P = asils.config('nadir_hold_ais', fullfile(asils.util.root(), 'cases', 'ais_3u.csv'), struct());
    P.sim.duration_s = 600;
    O = asils.orbit.init(P);
    cfg = pop_cfg_(P, O.y0);
    evalc('W = op.buildWorld(cfg);');
    da = 0;
    for tt = [0 7 300 555 600]
        [r, v, O] = asils.orbit.state(O, tt);
        a1 = asils.orbit.accel(O.W, tt, r, v); a2 = op.accel(tt, r, v, W);
        da = max(da, norm(a1 - a2)/norm(a2));
    end
    assert(da < 1e-14, 'the generated force model vs POP''s op.accel: %.2e relative', da);
    O = asils.orbit.init(P);
    cfg.tspan = 600; cfg.output = struct('times', [0; 300; 555; 600]);
    cfg.integrator = struct('method', 'rk78', 'rtol', 1e-12, 'atol', 1e-12);
    evalc('sol = op.propagate(cfg);');
    e = 0;
    for tt = [300 555 600]
        [r, ~, O] = asils.orbit.state(O, tt);
        rp = sol.stateAt(tt); e = max(e, norm(r - rp(1:3)));
    end
    assert(e < 1.0, 'in-loop vs POP rk78 differ by %.3f m', e);
    m = sprintf('force model vs POP %.1e relative; in-loop orbit vs POP op.propagate rk78: max %.3f m over 600 s', da, e);
end

function pop_path_()
    % the vendored POP on the path: the referent only (the twin's runs do not call it)
    if exist('ephemInputs', 'file') == 2, return, end
    addpath(fullfile(asils.util.root(), 'pop'));
    evalc('setup_paths();');
end

function cfg = pop_cfg_(P, y0)
    % the run's force set and start in POP's words (op.buildWorld's config), as the twin flew POP until S7.19b
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
    cfg.frame = struct('build', 'gmst', 'dUT1', 0.0, 'data_dir', '');
    cfg.r0 = y0(1:3); cfg.v0 = y0(4:6); cfg.tspan = P.sim.duration_s;
end

function m = t_short_runs()
    r1 = asils.run('nadir_hold_ais', 'cases/ais_3u.csv', 'set', struct('engine__duration_s', 20), 'quiet', true);
    r2 = asils.run('fine_hold_img', 'cases/ais_img_3u.csv', 'set', struct('engine__duration_s', 10), 'quiet', true);
    assert(all(isfinite(r1.q(:))) && all(isfinite(r2.q(:))), 'non-finite state');
    m = sprintf('runs: AIS 20 s in %.1f s wall, IMG 10 s in %.1f s wall', r1.wall_s, r2.wall_s);
end

function m = t_campaign_draw()
    % a Monte Carlo run is the same run wherever it is drawn, inside its bounds; an edge run puts one dispersion at a
    % bound and leaves the rest nominal; the draws are the engine's words
    C = asils.util.readjson(fullfile(asils.util.root(), 'data', 'campaigns', 'mc_nadir_ais.json'));
    f = intersect({'case', 'xCase', 'x_case', 'case_'}, fieldnames(C));
    P0 = asils.config(C.scenario, fullfile('cases', [C.(f{1}) '.csv']), struct('seed', 1));
    [a, da] = asils.campaign.draw(C, P0, 3); [b, db] = asils.campaign.draw(C, P0, 3); [~, dc] = asils.campaign.draw(C, P0, 4);
    assert(isequal(a, b) && isequal(da, db), 'the same run draws the same values');
    assert(~isequal(da, dc), 'another run draws other values');
    [sc, eng] = asils.util.overrides(a);
    assert(any(strcmp(eng(:, 1), 'engine.f107')) && any(strcmp(sc(:, 1), 'initial.arg_lat_deg')), 'the engine''s words');
    for k = 1:40
        [~, d] = asils.campaign.draw(C, P0, k);
        if isfield(d, 'F107'), assert(d.F107 >= 70 && d.F107 <= 220, 'solar flux inside its bounds'); end
    end
    E = asils.util.readjson(fullfile(asils.util.root(), 'data', 'campaigns', 'edge_nadir_ais.json'));
    ds = E.dispersions; if ~iscell(ds), ds = num2cell(ds); end
    j = find(cellfun(@(s) strcmp(s.kind, 'solar_flux'), ds), 1);
    [~, lo] = asils.campaign.draw(E, P0, 2*j - 1); [~, hi] = asils.campaign.draw(E, P0, 2*j);
    assert(lo.F107 == ds{j}.lo && hi.F107 == ds{j}.hi, 'edge runs %d and %d put the solar flux at its bounds', 2*j - 1, 2*j);
    P1 = asils.config(C.scenario, fullfile('cases', [C.(f{1}) '.csv']), struct('seed', 1, 'set', a));
    assert(P1.env.F107 == da.F107 && norm(P1.sc.m_res) > 0, 'the draws reach the truth');
    m = sprintf('mc draws reproducible, engine words, in bounds over 40 runs; edge runs %d/%d at the flux bounds', 2*j - 1, 2*j);
end

function m = t_metrics_evaluate()
    % the statistics and the verdict on a known channel: APE rising 0..10 deg over the window
    rec = asils.run('nadir_hold_ais', 'cases/ais_3u.csv', 'set', struct('engine__duration_s', 20), 'quiet', true);
    n = numel(rec.t); rec.ape_3ax = linspace(0, 10, n);
    req = rec.P.case.v.req_ape;
    mk = @(stat) struct('id', 'a', 'kind', 'ape', 'window', 'all', 'statistic', stat, 'requirement', 'req.ape');
    rec.P.scenario.metrics = {mk('max'), mk('mean'), mk('rms')};
    M = asils.metrics.evaluate(rec);
    assert(abs(M(1).value - 10) < 1e-12, 'max is the largest sample');
    assert(abs(M(2).value - 5) < 1e-9, 'mean of a uniform ramp is its middle');
    assert(abs(M(3).value - sqrt(mean(rec.ape_3ax.^2))) < 1e-12, 'rms');
    assert(M(1).req == req && M(1).pass == double(10 <= req), 'the verdict is value <= requirement');
    m = sprintf('max/mean/rms on a 0..10 deg ramp, verdict against req.ape = %g', req);
end

function m = t_flex_plant()
% one flexible mode: momentum and energy kept, the free-free frequency Omega/sqrt(1 - p)
    p = 0.3; f = 0.5; I = diag([0.02 0.03 0.05]);
    fl = struct('on', true, 'delta', [0; 0; sqrt(p*0.05)], 'omega', 2*pi*f, 'zeta', 0);
    geo = asils.models.rotors.rotor_geometry(zeros(8, 3), zeros(4, 3), zeros(8, 1), 0, 0);
    B = asils.plant.body(I, geo, fl);
    x = state_([0; 0; 0; 1], [0.01; -0.02; 0]); x.eta = 0.01;
    dl = fl.delta;
    Hi = @(x) asils.la.dcm(x.q)'*(I*x.w + dl*x.etad);
    E = @(x) 0.5*x.w'*I*x.w + x.w'*dl*x.etad + 0.5*x.etad^2 + 0.5*fl.omega^2*x.eta^2;
    H0 = Hi(x); E0 = E(x); dt = 0.01; t = []; last = x.eta;
    for k = 1:20000
        x = asils.plant.step(x, dt, B, zeros(3,1), zeros(8,1), zeros(4,1));
        if last > 0 && x.eta <= 0, t(end+1) = k*dt; end %#ok<AGROW>
        last = x.eta;
    end
    assert(norm(Hi(x) - H0) < 1e-8*(1 + norm(H0)), 'inertial momentum %g -> %g', norm(H0), norm(Hi(x)));
    assert(abs(E(x) - E0) < 1e-8*(1 + abs(E0)), 'energy %g -> %g', E0, E(x));
    T = (t(end) - t(1))/(numel(t) - 1); want = sqrt(1 - p)/f;
    assert(abs(T - want) < 0.01*want, 'period %.4f s vs %.4f s', T, want);
    m = sprintf('momentum and energy kept; period %.3f s (free-free %.3f s)', T, want);
end

function m = t_solution_scenario()
    % every option of a mission mode builds a scenario the engine's schema accepts
    n = 0;
    C = asils.case.read(fullfile(asils.util.root(), 'cases', 'ais_3u.csv'));
    for mode = {'detumble', 'nadir_pointing'}
        M = asils.solution.mode(mode{1});
        for i = 1:numel(M.options)
            S = asils.solution.scenario('ais_3u', mode{1}, M.options{i}.id);
            asils.scenario.check(S, C);
            n = n + 1;
        end
    end
    m = sprintf('%d mode options build schema-valid scenarios', n);
end

function m = t_coil_lag()
%T_COIL_LAG  A coil's dipole lags its command by tau (act's coil_lag: 63.2 % at one tau, its mean 1/e of it).
    [e, avg] = asils.models.coillag.coil_lag(0, 0.2, 0.01, 0.01);
    assert(abs(e - 0.2*(1 - exp(-1))) < 1e-15 && abs(avg - 0.2*exp(-1)) < 1e-15, 'end %.6f, mean %.6f', e, avg);
    [e2, ~] = asils.models.coillag.coil_lag(e, 0.2, 0.01, 0.01);
    assert(abs(e2 - 0.2*(1 - exp(-2))) < 1e-15, 'second step %.6f', e2);
    m = sprintf('%.1f %% at one tau, mean 1/e', 100*(1 - exp(-1)));
end

function m = t_wheel_motor()
%T_WHEEL_MOTOR  The wheel torque follows its back-EMF line T_s (1 - |w|/w_nl) above the knee (act's motor_line) and
%   stops at the speed limit (speed_limited).
    ts = 4e-3*5/12; wn = 1250;
    hd = @(cmd, w) asils.models.wheelspeed.speed_limited(asils.models.wheelmotor.motor_line(cmd, w, ts, wn), w, 600);
    assert(abs(hd(1e-3, 300) - 1e-3) < 1e-15, 'below the knee');
    for w = [520 580 -550]
        want = ts*(1 - abs(w)/wn)*sign(w);
        assert(abs(hd(1e-3*sign(w), w) - want) < 1e-15, 'w %g: %.6e vs %.6e', w, hd(1e-3*sign(w), w), want);
    end
    assert(abs(hd(-1e-3, 580) + 1e-3) < 1e-15, 'braking');
    assert(hd(1e-3, 600) == 0 && abs(hd(-1e-3, 600) + 1e-3) < 1e-15, 'speed limit');
    m = sprintf('on the line at 580 rad/s: %.3f mN m (stall %.3f, no-load 1250 rad/s)', 1e3*hd(1e-3, 580), 1e3*ts);
end

function [u, d, ht, hq] = tracker_(blind)
    d = asils.models.sttracker.StDesc_zero();
    d.nh = 1; d.bs(1, :) = [0 0 1]; d.noise_cross = 1e-4; d.noise_roll = 1e-3; d.max_rate = 1; d.sun_excl = 0.5;
    d.earth_excl = 0.3; d.fov = 0.17; d.model = 0; d.moon_excl = 0.26; d.blind_s = blind; d.noise_rate_ref = 0.01;
    [u, ~] = asils.models.sttracker.st_new(d, asils.devices.stream(1, 'disp'), asils.devices.stream(1, 'st'));
    ht = zeros(64, 1); hq = zeros(64, 4);
end

function [ok, u, ht, hq, hn] = see_(u, d, ht, hq, hn, t, sun, moon)
    q = [0;0;0;1];
    [hn, ht, hq] = asils.models.sttracker.st_history(ht, hq, hn, t, q);
    [okv, ~, ~, ~, ~, u, ht, hq] = asils.models.sttracker.st_sample(u, d, ht, hq, hn, zeros(4000, 3), zeros(4000, 1), q, t, ...
        zeros(3,1), sun, moon, [0;0;-1], 1);
    ok = okv(1);
end

function m = t_st_moon_blind()
%T_ST_MOON_BLIND  No attitude inside the Moon's exclusion cone; blind for blind_s after the Moon leaves it.
    [u, d, ht, hq] = tracker_(5); hn = 0; sun = [1;0;0];
    mo = @(deg) [sind(deg); 0; cosd(deg)];
    [v, u, ht, hq, hn] = see_(u, d, ht, hq, hn, 0, sun, mo(40)); assert(v, 'clear sky');
    [v, u, ht, hq, hn] = see_(u, d, ht, hq, hn, 1, sun, mo(14)); assert(~v, 'Moon in the cone');
    [v, u, ht, hq, hn] = see_(u, d, ht, hq, hn, 1.2, sun, mo(16)); assert(~v, 'still blind');
    [v, u, ht, hq, hn] = see_(u, d, ht, hq, hn, 5.9, sun, mo(20)); assert(~v, 'blind until 6 s');
    v = see_(u, d, ht, hq, hn, 6.0, sun, mo(20)); assert(v, 'recovered at 6 s');
    m = 'Moon cone 15 deg refused; recovered 5 s after';
end

function m = t_gps_latency()
%T_GPS_LATENCY  A fix holds the truth of latency_s ago, linear between ticks (sens's gps_history, gps_delayed).
    L = 0.25; ht = zeros(256, 1); hr = zeros(256, 3); hv = hr; hn = 0;
    rf = @(t) [7e6 + 100*t; -3*t; 2*t]; vf = @(t) [100; -3; 2 + t];
    for k = 0:30, t = k*0.1; [hn, ht, hr, hv] = asils.models.gnss.gps_history(ht, hr, hv, hn, L, t, rf(t), vf(t)); end
    [te, r, v] = asils.models.gnss.gps_delayed(ht, hr, hv, hn, L, 3.0);
    assert(abs(te - 2.75) < 1e-12 && norm(r - rf(2.75)) < 1e-6 && norm(v - vf(2.75)) < 1e-12, 'delayed state');
    r3 = rf(3.0); lagm = r3(1) - r(1);
    assert(abs(lagm - 25) < 1e-6, 'along-track lag %.3f m', lagm);
    m = sprintf('the fix at 3 s is the state of 2.75 s: %.1f m behind at 100 m/s', lagm);
end

function m = t_earth_radiation()
%T_EARTH_RADIATION  Albedo and Earth infrared pressures in closed form (env's albedo_pressure, earth_ir_pressure).
    rn = 6378137 + 500e3; vf = (6378137/rn)^2;
    pa = asils.models.albedo.albedo_pressure([rn;0;0], [1.5e11;0;0], 4.56e-6);
    pi_ = asils.models.earthir.earth_ir_pressure([rn;0;0]);
    assert(abs(pa - 0.30*4.56e-6*vf) < 1e-20 && abs(pi_ - 237/299792458*vf) < 1e-20, 'pressures');
    assert(asils.models.albedo.albedo_pressure([rn;0;0], [-1.5e11;0;0], 4.56e-6) == 0, 'night side');
    G = facets_([0.1 0.1 0.34], [0; 0.01; 0]);
    p = asils.env.torques([0;0;0;1], [rn;0;0], [1;0;0], [0;0;0], [-1.5e11;0;0], 0, 4.56e-6, 0, eye(3), G, [0;0;0], 3.986e14, [0 0 1 0]);
    F = pi_*0.1*0.34*(1 + 0.3 + 2*0.3/3);
    assert(abs(p(3,3) - 0.01*F) < 1e-12*0.01*F && abs(p(1,3)) < 1e-25 && abs(p(2,3)) < 1e-25, 'IR torque %.4e vs %.4e', p(3,3), 0.01*F);
    m = sprintf('albedo %.2e Pa, IR %.2e Pa at 500 km; IR torque %.2e N m', pa, pi_, p(3,3));
end

function m = t_fidelity_refused()
%T_FIDELITY_REFUSED  A part that leaves out a value its device reads is refused by name.
    R = asils.util.root(); dd = fullfile(R, 'store', 'sized', sprintf('zz_test_%06d', randi(1e6)));
    mkdir(fullfile(dd, 'parts')); mkdir(fullfile(dd, 'products'));
    c = onCleanup(@() rmdir(dd, 's'));
    base = asils.util.readjson(fullfile(R, 'data', 'products', 'TRN-P-3U-RW-RCS.json'));
    fills = base.fill; if ~iscell(fills), fills = num2cell(fills); end
    cases = {'coils', 'time_constant_s'; 'wheels', 'motor_kt_Nm_per_A'; 'wheels', 'friction_static_Nm'; 'wheels', 'speed_max_rad_s'; ...
             'star_tracker', 'moon_exclusion_rad'; 'star_tracker', 'blind_recovery_s'; 'star_tracker', 'noise_doubling_rate_rad_s'; 'gnss', 'latency_s'};
    for k = 1:size(cases, 1)
        slot = cases{k, 1}; key = cases{k, 2};
        i = find(cellfun(@(f) strcmp(f.slot, slot), fills));
        p = asils.util.readjson(fullfile(R, 'data', 'parts', [fills{i}.part '.json']));
        p.nominal = rmfield(p.nominal, key); p.part_number = 'T-EDIT';
        put_(fullfile(dd, 'parts', 'T-EDIT.json'), p);
        pr = base; f2 = fills; f2{i}.part = 'T-EDIT'; pr.fill = f2; pr.id = 'T-EDIT';
        put_(fullfile(dd, 'products', 'T-EDIT.json'), pr);
        try
            asils.product.load('T-EDIT'); error('not refused');
        catch e
            assert(~isempty(strfind(e.message, key)), '%s %s: %s', slot, key, e.message);
        end
    end
    m = sprintf('%d missing values refused by name', size(cases, 1));
end

function put_(f, s)
    fid = fopen(f, 'w'); fwrite(fid, jsonencode(s)); fclose(fid);
end

function m = t_chains_from_part()
%T_CHAINS_FROM_PART  model = 'image' and level = 'chain' read every value from the part: one left out is refused by name,
%   as is a model or level there is none of; the image model and the Sun chain answer through the run.
    R = asils.util.root(); dd = fullfile(R, 'store', 'sized', sprintf('zz_test_%06d', randi(1e6)));
    mkdir(fullfile(dd, 'parts')); mkdir(fullfile(dd, 'products'));
    c = onCleanup(@() rmdir(dd, 's'));
    base = asils.util.readjson(fullfile(R, 'data', 'products', 'TRN-P-3U-IMG.json'));
    fills = base.fill; if ~iscell(fills), fills = num2cell(fills); end
    ist = find(cellfun(@(f) strcmp(f.slot, 'star_tracker'), fills)); isun = find(cellfun(@(f) strcmp(f.slot, 'sun_sensors'), fills));
    cam = struct('detector_px', 256, 'psf_sigma_px', 1.2, 'flux_mag6_e', 3000, 'background_e', 50, 'read_noise_e', 8, ...
        'centroid_k_sigma', 5, 'max_spots', 20, 'id_tol_rad', 2e-4, 'id_mag_tol', 0.25, 'fit_tol_rad', 1e-4);
    hd = struct('aperture_side_m', 1e-3, 'aperture_height_m', 0.6e-3, 'current_noise_frac', 0.005, 'current_min_frac', 0.05);
    chain_parts_(R, dd, cam, hd, ''); chain_product_(dd, base, fills, ist, isun, 'image', 'chain');
    dv = asils.product.load('T-CHAINS');
    assert(dv.st.model == 2 && dv.st.cam.n == 256 && dv.sun.rec.chain && dv.sun.rec.head.h == 0.6e-3, 'values from the part');
    keys = [fieldnames(cam); fieldnames(hd)];
    for k = 1:numel(keys)
        chain_parts_(R, dd, cam, hd, keys{k});
        try, asils.product.load('T-CHAINS'); error('not refused'); catch e
            assert(~isempty(strfind(e.message, keys{k})), '%s: %s', keys{k}, e.message);
        end
    end
    chain_parts_(R, dd, cam, hd, '');
    for bad = {{'pinhole', 'chain', 'model pinhole'}, {'image', 'currents', 'level currents'}}
        chain_product_(dd, base, fills, ist, isun, bad{1}{1}, bad{1}{2});
        try, asils.product.load('T-CHAINS'); error('not refused'); catch e
            assert(~isempty(strfind(e.message, bad{1}{3})), '%s', e.message);
        end
    end
    m = sprintf('%d values refused by name; unknown model and level refused', numel(keys));
end

function chain_parts_(R, dd, cam, hd, drop)
    p = asils.util.readjson(fullfile(R, 'data', 'parts', 'SYN-ST-1.json'));
    for k = fieldnames(cam)', if ~strcmp(k{1}, drop), p.nominal.(k{1}) = cam.(k{1}); end, end
    p.part_number = 'T-ST'; put_(fullfile(dd, 'parts', 'T-ST.json'), p);
    p = asils.util.readjson(fullfile(R, 'data', 'parts', 'SYN-SUN-1.json'));
    for k = fieldnames(hd)', if ~strcmp(k{1}, drop), p.nominal.(k{1}) = hd.(k{1}); end, end
    p.part_number = 'T-SUN'; put_(fullfile(dd, 'parts', 'T-SUN.json'), p);
end

function chain_product_(dd, base, fills, ist, isun, model, level)
    pr = base; f2 = fills; f2{ist}.part = 'T-ST'; f2{ist}.model = model; f2{isun}.part = 'T-SUN'; f2{isun}.level = level;
    pr.fill = f2; pr.id = 'T-CHAINS'; put_(fullfile(dd, 'products', 'T-CHAINS.json'), pr);
end

function m = t_streams()
%T_STREAMS  The run's named streams are the engine's (adcs-sim-core rng.rs): FNV-1a of the name, SplitMix64; the kept
%   blocks of the runtime give the draws one at a time would.
    s = asils.devices.stream(1, 'gyro'); s2 = s; x = zeros(1, 600); y = x;
    for k = 1:600, [x(k), s] = asils.pc.stream_normal(s); end
    for k = 1:600
        [u1, s2] = asils.pc.stream_uniform(s2); %#ok<ASGLU>
    end
    t = asils.devices.stream(1, 'gyro');
    for k = 1:300, [y(k), t] = asils.pc.stream_uniform(t); end
    assert(all(isfinite(x)) && abs(mean(x)) < 0.15 && abs(std(x) - 1) < 0.1, 'normal draws');
    assert(all(y(1:300) > 0 & y(1:300) < 1), 'uniform draws');
    assert(isequal(s(3:4), s2(3:4)), 'two uniforms a normal pair');
    m = sprintf('600 normal draws: mean %.3f, std %.3f', mean(x), std(x));
end

function m = t_checkpoint()
%T_CHECKPOINT  A run cut short goes on from its checkpoint to the same end as a run flown whole.
    f = [tempname '.ckpt'];
    a = asils.run('nadir_hold_ais', 'cases/ais_3u.csv', 'set', struct('engine__duration_s', 12), 'quiet', true);
    b = asils.run('nadir_hold_ais', 'cases/ais_3u.csv', 'set', struct('engine__duration_s', 12), 'quiet', true, 'checkpoint', f, 'checkpoint_s', 0);
    assert(exist(f, 'file') ~= 2, 'a whole run leaves no checkpoint');
    assert(isequal(a.q, b.q) && isequal(a.w, b.w), 'a run with checkpoints is the same run');
    m = 'checkpointed every tick: the same trajectory, the checkpoint removed at the end';
end

function m = t_engine_parity()
%T_ENGINE_PARITY  The twin and the engine fly the same run value for value: the design's models and flight software in
%   both, the same blob, the same named streams (the engine's channels are written to 9 digits).
    R = asils.util.root(); out = tempname(); mkdir(out); c = onCleanup(@() rmdir(out, 's'));
    [rc, txt] = system(sprintf('ADCS_ROOT=''%s'' ''%s'' run fine_hold_img --set engine.duration_s=10 --out ''%s'' --quiet', R, asils.util.engine(), fullfile(out, 'eng')));
    assert(rc == 0, 'engine: %s', txt);
    rec = asils.run('fine_hold_img', 'cases/ais_img_3u.csv', 'set', struct('engine__duration_s', 10), 'quiet', true);
    asils.rec.write(rec, fullfile(out, 'twin'));
    A = csv_(fullfile(out, 'twin', 'channels.csv')); E = csv_(fullfile(out, 'eng', 'channels.csv'));
    names = intersect(fieldnames(A), fieldnames(E)); worst = 0; at = '';
    for i = 1:numel(names)
        a = A.(names{i}); e = E.(names{i}); ok = isfinite(a) & isfinite(e) & abs(e) > 1e-20;
        d = max([0; abs(a(ok) - e(ok))./max(abs(e(ok)), 1e-300)]);
        if d > worst, worst = d; at = names{i}; end
    end
    assert(worst < 1e-7, 'twin and engine differ by %.2e (%s)', worst, at);
    m = sprintf('%d channels x %d samples: worst relative difference %.1e (9-digit files)', numel(names), numel(A.t_s), worst);
end

function S = csv_(f)
    fid = fopen(f); h = strsplit(fgetl(fid), ','); fclose(fid);
    M = dlmread(f, ',', 1, 0);
    S = struct();
    for i = 1:numel(h), S.(matlab.lang.makeValidName(h{i})) = M(:, i); end
end

function m = t_physics_vectors()
    % translator = interpreter: the vectors the pseudocode's interpreter drew through the MATLAB translations: the
    % design's relations and every group's computing rows (+asils/+relations), and the language's own test
    % (+asils/+pcselftest). Bit for bit where a function uses no transcendental; else within 1e-12 relative.
    here = fileparts(mfilename('fullpath'));
    pk = {'physics_vectors.json', 'asils.relations'; 'pcselftest_vectors.json', 'asils.pcselftest'; ...
          'groups_vectors.json', 'asils.relations'};
    n = 0; exact = 0; vals = 0; worst = 0;
    for p = 1:size(pk, 1)
        v = jsondecode(fileread(fullfile(here, '..', 'data', pk{p, 1})));
        call = str2func([pk{p, 2} '.call']); call_seq = str2func([pk{p, 2} '.call_seq']);
        names = fieldnames(v.vectors);
        for i = 1:numel(names)
            e = v.vectors.(names{i}); name = regexprep(names{i}, '__', '::', 'once');   % jsondecode writes :: as __
            if isfield(v, 'library') && isfield(v.library, names{i}), name = v.library.(names{i}); end
            sets = e.sets;
            for k = 1:numel(sets)
                if iscell(sets), s = sets{k}; else, s = sets(k); end
                if isfield(e, 'proc') && e.proc
                    c = s.calls; X = []; W = {};
                    for j = 1:numel(c)
                        if iscell(c), cj = c{j}; else, cj = c(j); end
                        X = [X, from_bits(cj.in_bits)]; W{end + 1} = from_bits(cj.out_bits); %#ok<AGROW>
                    end
                    Y = call_seq(name, X); got = Y(:); want = vertcat(W{:});
                else
                    got = call(name, from_bits(s.in_bits)); want = from_bits(s.out_bits);
                end
                assert(numel(got) == numel(want), '%s: %d outputs, wanted %d', name, numel(got), numel(want));
                for j = 1:numel(want)
                    err = abs(got(j) - want(j))/max(abs(want(j)), 1e-300);
                    vals = vals + 1; exact = exact + (got(j) == want(j)); worst = max(worst, err);
                    assert(got(j) == want(j) || (~e.exact && err < 1e-12), '%s: MATLAB %.17g, interpreter %.17g', name, got(j), want(j));
                end
                n = n + 1;
            end
        end
    end
    m = sprintf('%d vectors, %d of %d values bit for bit, worst %.1e', n, exact, vals, worst);
end

function x = from_bits(b)
    % [high, low] 32-bit words of each double (jsondecode does not read decimals exactly)
    b = reshape(b, [], 2);
    x = zeros(size(b, 1), 1);
    for i = 1:size(b, 1), x(i) = typecast(uint32([b(i, 2), b(i, 1)]), 'double'); end
end
