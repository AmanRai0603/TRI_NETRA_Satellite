function ok = run_all_tests()
%RUN_ALL_TESTS  Unit and integration tests of the TRI-NETRA ADCS SILS.
%   >> startup_asils; addpath tests; run_all_tests
%   Each test prints PASS/FAIL with the number it checked. Each runs from its own seed
%   (rng(1000 + its position)), so a random test draws the same numbers every time.
%   Copyright (c) 2026 Agastya. All rights reserved.
    T = {@t_quat, @t_kinematics, @t_sso, @t_case, @t_igrf, @t_shadow, ...
         @t_torques, @t_plant_conservation, @t_cmg_plant, @t_mekf, @t_quest, @t_lqr, ...
         @t_fmr_spin_down, @t_cmg_steering, @t_hal_loopback, @t_select, @t_gen_bdot, @t_sun_spin_law, @t_sun_guidance, @t_sun_model, @t_st_chain, @t_sun_chain, @t_es_chain, ...
         @t_sizing, @t_modes_table, @t_orbit_vs_pop, @t_short_runs, @t_campaign_draw, @t_metrics_evaluate, @t_metrics_ecss, @t_flex_plant, @t_solution_scenario, ...
         @t_coil_lag, @t_wheel_motor, @t_wheel_stiction, @t_st_moon_blind, @t_st_rate_noise, @t_gps_latency, @t_earth_radiation, @t_fidelity_refused, @t_chain_parity, @t_chains_from_part, @t_physics_vectors};
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
    m = sprintf('composition %.1e, round trip %.1e', e, e2);
end

function m = t_kinematics()
    % inertially fixed vector seen from a body spinning at w: v_B' = -w x v_B
    q = asils.quat.norm(randn(4,1)); w = [0.01; -0.02; 0.03]; dt = 1e-3; v = [1; 2; 3];
    x = [q; w]; I = diag([1 2 3]);
    x1 = asils.plant.step(x, dt, eye(3), eye(3), asils.plant.geometry(zeros(3,0)), zeros(3,1), zeros(0,1));
    d_num = (asils.quat.dcm(x1(1:4))*v - asils.quat.dcm(q)*v)/dt;
    d_ana = -cross(w, asils.quat.dcm(q)*v);
    assert(norm(d_num - d_ana) < 1e-4, 'mismatch %.2e', norm(d_num - d_ana));
    m = sprintf('dv/dt error %.1e', norm(d_num - d_ana));
end

function m = t_sso()
    i = asils.orbit.sso_inclination(550);
    assert(abs(i - 97.593) < 0.01, 'got %.4f', i);
    m = sprintf('i(550 km) = %.3f deg', i);
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
    gh = asils.env.igrf_gh(2025.0);
    B0 = norm(asils.env.igrf_ned(gh, 0, 0, 0));
    Bp = norm(asils.env.igrf_ned(gh, 80*pi/180, 0, 0));
    B5 = norm(asils.env.igrf_ned(gh, 0, 0, 550));
    assert(B0 > 25000 && B0 < 40000 && Bp > 50000 && Bp < 62000 && B5 < B0, ...
        '|B| equator %.0f, 80N %.0f, 550 km %.0f nT', B0, Bp, B5);
    m = sprintf('|B| equator %.0f nT, 80N %.0f nT, 550 km %.0f nT', B0, Bp, B5);
end

function m = t_shadow()
    s = [1.496e11; 0; 0];
    a = asils.env.shadow([7e6; 0; 0], s); b = asils.env.shadow([-7e6; 0; 0], s); c = asils.env.shadow([0; 7e6; 0], s);
    assert(a == 1 && b == 0 && c == 1, 'nu = %g %g %g', a, b, c);
    m = 'sunlit 1, umbra 0, terminator side 1';
end

function m = t_torques()
    sc = struct('box_m', [0.34 0.1 0.1], 'cm_offset_m', [0;0;0], 'sigma_n', 0.8, 'sigma_t', 0.8, ...
                'vb_ratio', 0.05, 'refl', 0.6, 'spec_frac', 0.5);
    G = asils.env.geometry(sc); I = diag([0.0067 0.042 0.042]); q = [0;0;0;1];
    [~, p] = asils.env.torques(q, [7e6;0;0], [0;7500;0], [2e-5;0;0], [1.5e11;0;0], 1, 4.5e-6, 1e-12, I, G, [0;0;0], 3.986e14, [1 1 1 1]);
    assert(norm(p(:,1)) < 1e-15, 'GG on a principal axis %.2e', norm(p(:,1)));
    assert(norm(p(:,2)) < 1e-15 && norm(p(:,3)) < 1e-15, 'symmetric box about its CM must see no aero/SRP torque');
    sc.cm_offset_m = [0; 0; 0.02]; G = asils.env.geometry(sc);
    [~, p] = asils.env.torques(q, [7e6;0;0], [0;7500;0], [2e-5;0;0], [1.5e11;0;0], 1, 4.5e-6, 1e-12, I, G, [0;0;0], 3.986e14, [1 1 1 1]);
    assert(norm(p(:,2)) > 0, 'offset CM must give aero torque');
    m = sprintf('GG/aero/SRP symmetry held; aero with 2 cm offset %.2e N m', norm(p(:,2)));
end

function m = t_plant_conservation()
    I = diag([0.0067 0.042 0.043]); Aw = eye(3);
    x = [asils.quat.norm(randn(4,1)); 0.1*randn(3,1); 1e-3*randn(3,1)];
    H0 = asils.quat.dcm(x(1:4))'*(I*x(5:7) + Aw*x(8:10));
    M = asils.plant.geometry(Aw);
    for k = 1:1000, x = asils.plant.step(x, 0.1, I, inv(I), M, zeros(3,1), zeros(3,1)); end
    H1 = asils.quat.dcm(x(1:4))'*(I*x(5:7) + Aw*x(8:10));
    e = norm(H1 - H0)/norm(H0);
    assert(e < 1e-6, 'inertial momentum drift %.2e', e);
    m = sprintf('100 s torque-free: inertial |dH|/|H| = %.1e', e);
end

function m = t_cmg_plant()
    % gimballing a CMG pyramid exchanges momentum with the body: the INERTIAL
    % total momentum stays constant while the gimbals move
    pr = asils.product.load('TRN-P-3U-CMG'); X = pr.mex;
    M = asils.plant.geometry(X.A0, X.G, X.gi); I = diag([0.0067 0.042 0.043]);
    x = [asils.quat.norm(randn(4,1)); zeros(3,1); X.h0(:); zeros(4,1)];
    Hi = @(x) asils.quat.dcm(x(1:4))'*(I*x(5:7) + asils.plant.axes(M, x(12:15))*x(8:11));
    H0 = Hi(x);
    for k = 1:200, x = asils.plant.step(x, 0.05, I, inv(I), M, zeros(3,1), zeros(4,1), [0.3; -0.2; 0.1; 0.4]); end
    e = norm(Hi(x) - H0); w = norm(x(5:7));
    assert(e < 1e-9 && w > 1e-3, 'dH %.2e, body rate %.2e', e, w);
    m = sprintf('10 s of gimballing: |dH_inertial| = %.1e, body rate %.3f deg/s', e, w*180/pi);
end

function m = t_quest()
    rng(5); q = asils.quat.norm(randn(4,1)); r = randn(3, 8); r = r./sqrt(sum(r.^2, 1));
    b = asils.quat.dcm(q)*r;
    qe = asils.fsw.quest(b, r); e = asils.quat.angle(q, qe);
    assert(e < 1e-9, 'QUEST error %.2e rad', e);
    m = sprintf('q-method on 8 exact stars: %.1e rad', e);
end

function m = t_lqr()
    K = asils.fsw.lqr_gain([0 1; 0 0], [0; 1], eye(2), 1);       % double integrator: K = [1, sqrt(3)]
    assert(norm(K - [1 sqrt(3)]) < 1e-9, 'K = %s', mat2str(K));
    m = sprintf('double-integrator LQR gain [%.4f %.4f]', K);
end

function m = t_fmr_spin_down()
    pr = asils.product.load('TRN-P-3U-FMR'); X = pr.mex;
    assert(abs(X.T_sd(1) - 0.755) < 0.01, 'T_sd %.3f s', X.T_sd(1));
    assert(abs(X.h_max(1) - 1.0e-3) < 5e-5, 'h_max %.2e', X.h_max(1));
    m = sprintf('ring spin-down %.3f s (IDMAS 0.75 s), h_max %.2e N m s', X.T_sd(1), X.h_max(1));
end

function m = t_cmg_steering()
    pr = asils.product.load('TRN-P-3U-CMG'); X = pr.mex;
    M = asils.plant.geometry(X.A0, X.G, X.gi); d = [0.1; -0.2; 0.3; 0.05];
    A = asils.plant.axes(M, d); tau = [1e-4; -2e-4; 5e-5];
    [gd, ~] = asils.fsw.steer_sr(tau, A, X.h0(:), M, 1.5, false, 1e-12, 10);
    got = zeros(3,1);
    for i = 1:4, a = A(:,i); g = M.G(:,i); got = got - X.h0(i)*gd(i)*cross(g, a); end
    e = norm(got - tau)/norm(tau);
    assert(e < 1e-3, 'steering torque error %.2e', e);
    m = sprintf('SR steering delivers the torque to %.1e relative', e);
end

function m = t_hal_loopback()
    z = struct('w', [0.01; -0.02; 0.03], 'B', [2e-5; -1e-5; 3e-5], 'sun', [0.6; 0; 0.8], 'sun_ok', true, ...
        'st_ok', true, 'gps_ok', true, 'clean', true, 'q_st', asils.quat.norm([0.1; 0.2; 0.3; 0.9]), ...
        'r_gps', [6.9e6; 1.2e5; -3e4], 'v_gps', [10; 7600; 5], 'h', [1e-3; -2e-3], 'delta', zeros(0,1));
    dev = asils.product.load('TRN-P-3U-IMG'); L = asils.hal.lsb(dev);
    b = asils.hal.pack_sensors(z, L); z2 = asils.hal.unpack_sensors(b, z, L);
    e = [norm(z2.w - z.w)/L.gyro, norm(z2.B - z.B)/L.mag, norm(z2.r_gps - z.r_gps)/L.pos];
    assert(all(e < 2), 'loopback error in LSB %s', mat2str(e, 3));
    m = sprintf('%d-byte sensor frame round-trips within one LSB', numel(b));
end

function m = t_mekf()
    rng(3); q = asils.quat.norm(randn(4,1)); K = asils.fsw.mekf_init(asils.quat.norm(q + [0.05;0;0;0]), 0.1, 1e-3, 1e-5, 1e-8);
    r1 = [1;0;0]; r2 = [0;0.6;0.8];
    for k = 1:200
        K = asils.fsw.mekf_predict(K, [0;0;0], 0.1);
        K = asils.fsw.mekf_vector(K, asils.quat.dcm(q)*r1 + 1e-3*randn(3,1), r1, 1e-3);
        K = asils.fsw.mekf_vector(K, asils.quat.dcm(q)*r2 + 1e-3*randn(3,1), r2, 1e-3);
    end
    e = asils.quat.angle(K.q, q)*180/pi;
    assert(e < 0.05, 'MEKF error %.3f deg', e);
    m = sprintf('two-vector MEKF converged to %.4f deg', e);
end

function m = t_orbit_vs_pop()
    % The in-loop RK4 + Hermite must reproduce the POP's own batch propagator.
    P = asils.config('nadir_hold_ais', fullfile(asils.util.root(), 'cases', 'ais_3u.csv'));
    P.orbit.atmos = 'exponential'; P.sim.duration_s = 600;
    O = asils.orbit.init(P);
    cfg = O.cfg; cfg.tspan = 600; cfg.output = struct('times', [0; 300; 555; 600]);
    cfg.integrator = struct('method', 'rk78', 'rtol', 1e-12, 'atol', 1e-12);
    evalc('sol = op.propagate(cfg);');
    e = 0;
    for tt = [300 555 600]
        [r, ~, O] = asils.orbit.state(O, tt);
        rp = sol.stateAt(tt); e = max(e, norm(r - rp(1:3)));
    end
    assert(e < 1.0, 'in-loop vs POP rk78 differ by %.3f m', e);
    m = sprintf('in-loop POP vs op.propagate rk78: max %.3f m over 600 s', e);
end

function m = t_short_runs()
    r1 = asils.run('nadir_hold_ais', 'cases/ais_3u.csv', 'set', struct('sim__duration_s', 30), 'quiet', true);
    r2 = asils.run('fine_hold_img', 'cases/ais_img_3u.csv', 'set', struct('sim__duration_s', 30), 'quiet', true);
    assert(all(isfinite(r1.q(:))) && all(isfinite(r2.q(:))), 'non-finite state');
    m = sprintf('30 s runs: AIS %.1f s wall, IMG %.1f s wall', r1.wall_s, r2.wall_s);
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
%T_GEN_BDOT  L1 with w_d = 0 is plain B-dot; with w_d it drives the rate to w_d
%   (Standard Code theory eq 4.1-4.3: the torque m x B opposes w - w_d across B).
    B = [2e-5; -1e-5; 3e-5]; w = [0.05; -0.02; 0.1];
    m0 = asils.fsw.gen_bdot(B, -asils.util.cross3(w, B), zeros(3,1), 1e6);
    assert(dot(asils.util.cross3(m0, B), w) < 0, 'plain B-dot does not remove energy');
    wd = [0; 0; 0.1];
    m1 = asils.fsw.gen_bdot(B, -asils.util.cross3(w, B), wd, 1e6);
    tq = asils.util.cross3(m1, B);
    assert(dot(tq, w - wd) < 0, 'L1 does not drive toward w_d');
    m = sprintf('dV/dt < 0 for w_d = 0 and w_d = %.2f rad/s e_z', wd(3));
end

function m = t_sun_spin_law()
%T_SUN_SPIN_LAW  L2 (He et al.): the commanded torque never raises the Lyapunov
%   value (A . m0 <= 0), and E1 turns the coils off in eclipse.
    J = diag([0.0067 0.042 0.042]); g = struct('spin_dps', 6, 'k1', 0.01, 'k2', 0.05);
    worst = -Inf;
    for k = 1:200
        B = 3e-5*randn(3,1); w = 0.1*randn(3,1); s = randn(3,1); s = s/norm(s);
        m0 = asils.fsw.sun_spin(B, w, s, false, J, g);
        ws = -g.spin_dps*pi/180; sg = sign(w(3));
        A = asils.util.cross3(B, g.k1*(J*w - sg*J(3,3)*ws*s) + g.k2*diag([J(3,3)-J(1,1), J(3,3)-J(2,2), 0])*w);
        worst = max(worst, dot(A, m0));
    end
    assert(worst <= 1e-20, 'A.m0 = %.2e > 0', worst);
    assert(~any(asils.fsw.sun_spin([1e-5;0;0], [0;0;0.1], [0;0;-1], true, J, g)), 'E1 eclipse');
    m = sprintf('max A.m0 = %.1e over 200 draws; E1 coils off', worst);
end

function m = t_sun_guidance()
%T_SUN_GUIDANCE  Sun referencing puts the power face on the Sun and the roll
%   axis on the orbit normal projected across the Sun line.
    gd = struct('sun_axis', [0;0;-1], 'roll_axis', [0;1;0], 'sun_eci', [0.3;0.9;0.2]);
    r = [7e6;0;0]; v = [0;7.5e3;0];
    R = asils.quat.dcm(asils.fsw.guidance('sun', r, v, 0, gd));
    s = gd.sun_eci/norm(gd.sun_eci); n = cross(r, v); n = n/norm(n); e = n - (n'*s)*s; e = e/norm(e);
    e1 = acosd(min(1, [0 0 -1]*R*s)); e2 = acosd(min(1, [0 1 0]*R*e));
    assert(e1 < 1e-6 && e2 < 1e-6, 'Sun %.2e deg, roll %.2e deg', e1, e2);
    m = sprintf('power face on the Sun %.0e deg, roll axis %.0e deg', e1, e2);
end

function m = t_sun_model()
%T_SUN_MODEL  The onboard Sun (precessed to J2000) agrees with DE440 to 0.01 deg.
    P = asils.config('fine_hold_img', 'cases/ais_img_3u.csv'); O = asils.orbit.init(P);
    X = asils.orbit.context(O, 0); [r, ~] = asils.orbit.state(O, 0);
    s = X.sun_eci - r; e = acosd(asils.fsw.sun_model(asils.util.jd(P.epoch_utc))'*s/norm(s));
    assert(e < 0.01, 'onboard Sun off by %.4f deg', e);
    m = sprintf('onboard Sun vs DE440 %.4f deg', e);
end

function m = t_st_chain()
%T_ST_CHAIN  Star-tracker component chain on rendered frames: every frame
%   solved, the attitude within 60 arcsec (3-axis, roll included).
    rand('seed', 3); randn('seed', 3);
    cat = asils.devices.star_catalogue(4000); cam = asils.comp.star_tracker.camera(0.17);
    K = asils.comp.star_tracker.pairs(cat, cam.fov); K.R_head_nominal = eye(3);
    E = zeros(1, 8);
    for k = 1:8
        q = asils.quat.norm(randn(4,1));
        [qm, ok] = asils.comp.star_tracker.chain(q, eye(3), cat, K, cam);
        assert(ok, 'frame %d not solved', k);
        E(k) = asils.quat.angle(q, qm)*180/pi*3600;
    end
    assert(max(E) < 60, 'worst frame %.1f arcsec', max(E));
    m = sprintf('8/8 frames solved, median %.1f arcsec, worst %.1f arcsec', median(E), max(E));
end

function m = t_sun_chain()
%T_SUN_CHAIN  Quadrant Sun sensor: currents -> angles within 0.5 deg over +/-30 deg.
    p = asils.comp.sun_sensor.head(); p.noise = 0; E = [];
    for a = -30:10:30
        for b = -30:15:30
            s = [tand(a); tand(b); 1]; s = s/norm(s);
            [sh, ok] = asils.comp.sun_sensor.angles(asils.comp.sun_sensor.currents(s, p), p);
            assert(ok, 'no Sun at %g, %g', a, b); E(end+1) = acosd(min(1, s'*sh)); %#ok<AGROW>
        end
    end
    assert(max(E) < 0.5, 'worst %.3f deg', max(E));
    m = sprintf('noise-free worst %.2e deg over +/-30 deg', max(E));
end

function m = t_es_chain()
%T_ES_CHAIN  Earth sensor: limb points -> horizon fit recovers nadir within the
%   unit's 0.25 deg (SYN-ES-1) with 0.1 deg noise per limb crossing.
    p = asils.comp.earth_sensor.head(); randn('seed', 5);
    rho = asin(6378.137/6928.137); E = zeros(1, 20);
    for k = 1:20
        n = [0.3*randn; 0.3*randn; 1]; n = n/norm(n);
        [nm, ok] = asils.comp.earth_sensor.horizon(asils.comp.earth_sensor.limb(n, rho, p), rho);
        assert(ok, 'no horizon'); E(k) = acosd(min(1, n'*nm));
    end
    assert(max(E) < 0.25, 'worst %.3f deg', max(E));
    m = sprintf('20 attitudes, worst nadir error %.3f deg', max(E));
end

function m = t_sizing()
%T_SIZING  The sizing laws reproduce their anchor parts at the anchor demand.
    Dm = struct('h_req', 0.01, 'tau_req', 0.001);
    w = asils.sizing.rw(Dm, 'test');
    assert(abs(w.nominal.mass_kg - 0.15)/0.15 < 0.05 && abs(w.nominal.rotor_inertia_kgm2 - 1.6e-5)/1.6e-5 < 0.05, 'RW anchor');
    Dm.h_req = 0.008; c = asils.sizing.cmg(Dm, 'test', false);
    assert(abs(c.nominal.mass_kg - 0.12)/0.12 < 0.05, 'CMG anchor %.3f', c.nominal.mass_kg);
    Dm.h_req = 1e-3; f = asils.sizing.fmr(Dm, 'test');
    assert(f{2}.nominal.h_max_Nms >= 1e-3 && f{2}.nominal.field_power_W > 1, 'FMR');
    m = sprintf('RW %.3f kg, CMG %.3f kg, FMR-Y %.1f W with field', w.nominal.mass_kg, c.nominal.mass_kg, f{2}.nominal.power_steady_W);
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

function m = t_campaign_draw()
    % a Monte Carlo run is the same run wherever it is drawn, inside its bounds; an edge run puts
    % one dispersion at a bound and leaves the rest nominal
    C = asils.util.readjson(fullfile(asils.util.root(), 'data', 'campaigns', 'mc_nadir_ais.json'));
    f = intersect({'case', 'xCase', 'x_case', 'case_'}, fieldnames(C));   % 'case' is a keyword: the reader renames it
    P0 = asils.config(C.scenario, fullfile('cases', [C.(f{1}) '.csv']), struct('seed', 1));
    [a, da] = asils.campaign.draw(C, P0, 3); [b, db] = asils.campaign.draw(C, P0, 3); [~, dc] = asils.campaign.draw(C, P0, 4);
    assert(isequal(a, b) && isequal(da, db), 'the same run draws the same values');
    assert(~isequal(da, dc), 'another run draws other values');
    for k = 1:40
        [~, d] = asils.campaign.draw(C, P0, k);
        if isfield(d, 'F107'), assert(d.F107 >= 70 && d.F107 <= 220, 'solar flux inside its bounds'); end
    end
    E = asils.util.readjson(fullfile(asils.util.root(), 'data', 'campaigns', 'edge_nadir_ais.json'));
    ds = E.dispersions; if ~iscell(ds), ds = num2cell(ds); end
    j = find(cellfun(@(s) strcmp(s.kind, 'solar_flux'), ds), 1);
    [~, lo] = asils.campaign.draw(E, P0, 2*j - 1); [~, hi] = asils.campaign.draw(E, P0, 2*j);
    assert(lo.F107 == ds{j}.lo && hi.F107 == ds{j}.hi, 'edge runs %d and %d put the solar flux at its bounds', 2*j - 1, 2*j);
    m = sprintf('mc draws reproducible, in bounds over 40 runs; edge runs %d/%d at the flux bounds', 2*j - 1, 2*j);
end

function m = t_metrics_evaluate()
    % the statistics and the verdict on a known channel: APE rising 0..10 deg over the window
    rec = asils.run('nadir_hold_ais', 'cases/ais_3u.csv', 'set', struct('sim__duration_s', 20), 'quiet', true);
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


function m = t_metrics_ecss()
% the ECSS indices on signals with known answers (= engine/crates/adcs-sim/tests/ecss.rs)
    t = (0:999)*0.1; idx = 1:1000;
    rec = struct('t', t, 'P', struct('dev', struct('boresight', [0; 0; 1])));
    b = 1e-3; rec.e_vec = repmat([b; 0; 0], 1, 1000); rec.e_ake = rec.e_vec;
    assert(max(asils.metrics.ecss('rpe', rec, idx, 10, NaN)) < 1e-12, 'a bias has no relative error');
    v = asils.metrics.ecss('mpe', rec, idx, 10, NaN);
    assert(numel(v) == 10 && all(abs(v - b*180/pi) < 1e-12), 'a bias is all mean error');
    assert(max(asils.metrics.ecss('pde', rec, idx, 10, 30)) < 1e-12, 'a bias has no drift');
    a = 2e-3; rec.e_vec = [zeros(1, 1000); a*sin(2*pi*2.5*t); zeros(1, 1000)];
    assert(abs(max(asils.metrics.ecss('rpe', rec, idx, 10, NaN)) - a*180/pi) < 1e-9, 'a fast sine is all relative');
    assert(max(asils.metrics.ecss('mpe', rec, idx, 10, NaN)) < 1e-12, 'whole periods average to zero');
    r = 1e-5; rec.e_vec = [r*t; zeros(1, 1000); r*t];
    v = asils.metrics.ecss('pde', rec, idx, 10, 30);
    assert(numel(v) == 7 && all(abs(v - 30*r*sqrt(2)*180/pi) < 1e-9), 'a ramp drifts');
    v = asils.metrics.ecss('pde_los', rec, idx, 10, 30);
    assert(all(abs(v - 30*r*180/pi) < 1e-9), 'about the boresight is not a line-of-sight error');
    assert(abs(max(asils.metrics.ecss('rpe_los', rec, idx, 10, NaN)) - 4.95*r*180/pi) < 1e-9, 'half a block of ramp');
    rec.e_ake = [zeros(1, 1000); 1e-3*ones(1, 1000); zeros(1, 1000)]; rec.e_ake(:, t < 20) = NaN;
    assert(numel(asils.metrics.ecss('mke', rec, idx, 10, NaN)) == 8, 'blocks with no estimate are skipped');
    m = 'bias, sine, ramp and gaps give the engine''s answers';
end

% ---------------- device fidelity (B3.5; = engine/crates/adcs-sim-core/tests/core.rs) ----------------

function m = t_coil_lag()
%T_COIL_LAG  A coil's dipole lags its command by tau (63.2 % at one tau, 1/e of
%   it on average over that step), saturates at m_max, and an open coil stops at once.
    t = struct('m_max', 0.4, 'p_max', 0.3, 'tau', 0.01);
    D = struct('A', eye(3), 'scale', [1 1 1], 'dead', false(1, 3), 'm', zeros(1, 3), 'mc', zeros(1, 3));
    [avg, P, D, e] = asils.devices.mtq([0.2; 0; 0], D, t, t.tau); %#ok<ASGLU>
    assert(abs(e(1) - 0.2*(1 - exp(-1))) < 1e-15 && abs(e(1)/0.2 - 0.632) < 1e-3, 'end %.6f', e(1));
    assert(abs(avg(1) - 0.2*exp(-1)) < 1e-15, 'mean %.6f', avg(1));
    [~, ~, D, e] = asils.devices.mtq([0.2; 0; 0], D, t, t.tau);
    assert(abs(e(1) - 0.2*(1 - exp(-2))) < 1e-15, 'second step %.6f', e(1));
    for k = 1:50, [~, ~, D] = asils.devices.mtq([0; 0; 5], D, t, t.tau); end
    [avg, P, D, e] = asils.devices.mtq([0; 0; 5], D, t, t.tau);
    assert(abs(e(3) - 0.4) < 1e-12 && abs(avg(3) - 0.4) < 1e-12 && abs(P - 0.3) < 1e-9, 'saturation %.4f %.4f', e(3), P);
    D.dead(3) = true;
    [avg, ~, ~, e] = asils.devices.mtq([0; 0; 5], D, t, t.tau);
    assert(avg(3) == 0 && e(3) == 0, 'open coil');
    m = sprintf('%.1f %% at one tau, mean 1/e, saturates at 0.4 A m^2, open coil 0', 100*(1 - exp(-1)));
end

function [M, D] = one_wheel_()
    M = struct('kind', {{'rw'}}, 'h_max', 0.01, 'torque_max', 1e-3, 'J', 1.6e-5, 'coulomb', 1e-5, 'viscous', 1e-8, ...
        'p_steady', 0, 'friction_comp', 0.95, 'torque_noise', 0, 'eta', 0.8, ...
        't_stall', 4e-3*5/12, 'w_nl', 5/4e-3, 'speed_max', 600, 'f_static', 1.5e-5, 'w_stribeck', 1.0, ...
        'gimbal_rate_max', 1, 'gimbal_power', 0);
    D = struct('failed', false, 'gfailed', false(1, 0), 'tscale', 1, 'fscale', 1);
end

function m = t_wheel_motor()
%T_WHEEL_MOTOR  The wheel torque follows its back-EMF line T_s (1 - |w|/w_nl)
%   above the knee, the current limit below it and when braking, and stops at the speed limit.
    [M, D] = one_wheel_(); M.f_static = 0; M.coulomb = 0; M.viscous = 0;
    ts = 4e-3*5/12;
    hd = @(cmd, w) asils.devices.mex(cmd, zeros(0,1), w*1.6e-5, zeros(0,1), D, M, 0.1);
    assert(abs(hd(1e-3, 300) - 1e-3) < 1e-15, 'below the knee');
    for w = [520 580 -550]
        want = ts*(1 - abs(w)/1250)*sign(w);
        assert(abs(hd(1e-3*sign(w), w) - want) < 1e-15, 'w %g: %.6e vs %.6e', w, hd(1e-3*sign(w), w), want);
    end
    assert(abs(hd(-1e-3, 580) + 1e-3) < 1e-15, 'braking');
    assert(hd(1e-3, 600) == 0 && abs(hd(-1e-3, 600) + 1e-3) < 1e-15, 'speed limit');
    m = sprintf('on the line at 580 rad/s: %.3f mN m (stall %.3f, no-load 1250 rad/s)', 1e3*hd(1e-3, 580), 1e3*ts);
end

function m = t_wheel_stiction()
%T_WHEEL_STICTION  At rest a wheel breaks away only above its static friction; a
%   creeping one stops within the step; the Stribeck excess at w_s is (Fs - Fc)/e.
    [M, D] = one_wheel_();
    hd = @(cmd, h) asils.devices.mex(cmd, zeros(0,1), h, zeros(0,1), D, M, 0.1);
    assert(hd(1.4e-5, 0) == 0 && hd(-1.4e-5, 0) == 0, 'held at rest');
    assert(abs(hd(1.6e-5, 0) - 1.6e-5) < 1e-18, 'breaks away');
    assert(abs(hd(0, 1e-7)*0.1 + 1e-7) < 1e-20, 'creep stopped');
    want = 1e-4 - 0.05*(1e-5 + 1e-8) - (1.5e-5 - 1e-5)*exp(-1);
    assert(abs(hd(1e-4, 1.6e-5) - want) < 1e-15, 'Stribeck %.6e vs %.6e', hd(1e-4, 1.6e-5), want);
    m = 'held under 15 uN m, breaks away above; Stribeck (Fs - Fc)/e at 1 rad/s';
end

function [s, D] = tracker_(blind)
    s = struct('boresight', [0; 0; 1], 'noise_cross', 1e-4, 'noise_roll', 1e-3, 'latency', 0, 'max_rate', 1, ...
        'sun_excl', 0.5, 'earth_excl', 0.3, 'fov', 0.17, 'model', 'noise', 'moon_excl', 0.26, 'blind_s', blind, 'noise_rate_ref', 0.01);
    D = struct('hist_t', [], 'hist_q', [], 'dead', false, 'blind_until', -Inf, 'q_bias', [0;0;0;1], 'q_mis', [0;0;0;1]);
end

function m = t_st_moon_blind()
%T_ST_MOON_BLIND  No attitude inside the Moon's exclusion cone; blind for blind_s
%   after the Moon (or the Sun) leaves it; back at once with no blind time.
    [s, D] = tracker_(5); q = [0;0;0;1]; sun = [1;0;0]; nad = [0;0;-1];
    mo = @(deg) [sind(deg); 0; cosd(deg)];
    [~, v, D] = asils.devices.star_tracker(q, 0, zeros(3,1), sun, mo(40), nad, 1, D, s); assert(v, 'clear sky');
    [~, v, D] = asils.devices.star_tracker(q, 1, zeros(3,1), sun, mo(14), nad, 1, D, s); assert(~v, 'Moon in the cone');
    [~, v, D] = asils.devices.star_tracker(q, 1.2, zeros(3,1), sun, mo(16), nad, 1, D, s); assert(~v, 'still blind');
    [~, v, D] = asils.devices.star_tracker(q, 5.9, zeros(3,1), sun, mo(20), nad, 1, D, s); assert(~v, 'blind until 6 s');
    [~, v] = asils.devices.star_tracker(q, 6.0, zeros(3,1), sun, mo(20), nad, 1, D, s); assert(v, 'recovered at 6 s');
    [s, D] = tracker_(0);
    [~, v, D] = asils.devices.star_tracker(q, 0, zeros(3,1), mo(25), mo(90), nad, 1, D, s); assert(~v, 'Sun in the cone');
    [~, v] = asils.devices.star_tracker(q, 0.2, zeros(3,1), mo(35), mo(90), nad, 1, D, s); assert(v, 'no blind time');
    m = 'Moon cone 15 deg refused; recovered 5 s after; Sun likewise';
end

function m = t_st_rate_noise()
%T_ST_RATE_NOISE  Cross-boresight noise rms = noise_cross (1 + |w|/w_ref).
    q = [0;0;0;1]; r = [];
    for c = [0 1; 0.02 3]'
        [s, D] = tracker_(0); n = 4000; ss = 0;
        for i = 1:n
            [z, v, D] = asils.devices.star_tracker(q, i*0.2, [0;0;c(1)], [1;0;0], [-1;0;0], [0;0;-1], 1, D, s); assert(v);
            ss = ss + (2*z(1))^2 + (2*z(2))^2;
        end
        rms = sqrt(ss/(2*n)); r(end+1) = rms; %#ok<AGROW>
        assert(abs(rms/(1e-4*c(2)) - 1) < 0.05, 'w %g: rms %.3e vs %.3e', c(1), rms, 1e-4*c(2));
    end
    m = sprintf('rms %.2e at rest, %.2e at 2 w_ref (x%.2f)', r(1), r(2), r(2)/r(1));
end

function m = t_gps_latency()
%T_GPS_LATENCY  A fix holds the truth of latency_s ago (linear between ticks).
    G = struct('t', zeros(1,0), 'r', zeros(3,0), 'v', zeros(3,0)); L = 0.25;
    rf = @(t) [7e6 + 100*t; -3*t; 2*t]; vf = @(t) [100; -3; 2 + t];
    for k = 0:30, t = k*0.1; G = asils.devices.gps_history(G, t, rf(t), vf(t), L); end
    [te, r, v] = asils.devices.gps_delayed(G, 3.0, L);
    assert(abs(te - 2.75) < 1e-12 && norm(r - rf(2.75)) < 1e-6 && norm(v - vf(2.75)) < 1e-12, 'delayed state');
    r3 = rf(3.0); lagm = r3(1) - r(1);
    assert(abs(lagm - 25) < 1e-6, 'along-track lag %.3f m', lagm);
    G2 = struct('t', zeros(1,0), 'r', zeros(3,0), 'v', zeros(3,0));
    G2 = asils.devices.gps_history(G2, 0, rf(0), vf(0), L); G2 = asils.devices.gps_history(G2, 0.1, rf(0.1), vf(0.1), L);
    [te, r] = asils.devices.gps_delayed(G2, 0.1, L);
    assert(te == 0 && isequal(r, rf(0)), 'before the history reaches back');
    m = sprintf('the fix at 3 s is the state of 2.75 s: %.1f m behind at 100 m/s', lagm);
end

function m = t_earth_radiation()
%T_EARTH_RADIATION  Albedo and Earth infrared pressures in closed form; in
%   eclipse only the infrared torques a box whose CM is off its centre.
    rn = 6378137 + 500e3; vf = (6378137/rn)^2;
    [pa, pi_] = asils.env.earth_pressure([rn;0;0], [1.5e11;0;0], 4.56e-6);
    assert(abs(pa - 0.30*4.56e-6*vf) < 1e-20 && abs(pi_ - 237/299792458*vf) < 1e-20, 'pressures');
    pa0 = asils.env.earth_pressure([rn;0;0], [-1.5e11;0;0], 4.56e-6); assert(pa0 == 0, 'night side');
    sc = struct('box_m', [0.1 0.1 0.34], 'cm_offset_m', [0; 0.01; 0], 'sigma_n', 0.8, 'sigma_t', 0.8, 'vb_ratio', 0.05, 'refl', 0.6, 'spec_frac', 0.5);
    G = asils.env.geometry(sc);
    [~, p] = asils.env.torques([0;0;0;1], [rn;0;0], [1;0;0], [0;0;0], [-1.5e11;0;0], 0, 4.56e-6, 0, eye(3), G, [0;0;0], 3.986e14, [0 0 1 0]);
    F = pi_*0.1*0.34*(1 + 0.3 + 2*0.3/3);
    assert(abs(p(3,3) - 0.01*F) < 1e-12*0.01*F && abs(p(1,3)) < 1e-25 && abs(p(2,3)) < 1e-25, 'IR torque %.4e vs %.4e', p(3,3), 0.01*F);
    m = sprintf('albedo %.2e Pa, IR %.2e Pa at 500 km; IR torque %.2e N m', pa, pi_, p(3,3));
end

function m = t_fidelity_refused()
%T_FIDELITY_REFUSED  A part that leaves out a B3.5 value its device reads is refused by name.
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

function m = t_flex_plant()
% one flexible mode (= engine/crates/adcs-sim-core/tests/core.rs): momentum and energy kept,
% the free-free frequency Omega/sqrt(1 - p)
    p = 0.3; f = 0.5; I = diag([0.02 0.03 0.05]);
    dl = [0; 0; sqrt(p*0.05)];
    M = asils.plant.geometry(zeros(3,0));
    M.flex = struct('on', true, 'delta', dl, 'omega', 2*pi*f, 'zeta', 0, 'Minv', inv(I - dl*dl'));
    x = [0; 0; 0; 1; 0.01; -0.02; 0; 0.01; 0];
    Hi = @(x) asils.quat.dcm(x(1:4))'*(I*x(5:7) + dl*x(9));
    E = @(x) 0.5*x(5:7)'*I*x(5:7) + x(5:7)'*dl*x(9) + 0.5*x(9)^2 + 0.5*M.flex.omega^2*x(8)^2;
    H0 = Hi(x); E0 = E(x); dt = 0.01; t = []; last = x(8);
    for k = 1:20000
        x = asils.plant.step(x, dt, I, inv(I), M, zeros(3,1), zeros(0,1), zeros(0,1));
        if last > 0 && x(8) <= 0, t(end+1) = k*dt; end %#ok<AGROW>
        last = x(8);
    end
    assert(norm(Hi(x) - H0) < 1e-8*(1 + norm(H0)), 'inertial momentum %g -> %g', norm(H0), norm(Hi(x)));
    assert(abs(E(x) - E0) < 1e-8*(1 + abs(E0)), 'energy %g -> %g', E0, E(x));
    T = (t(end) - t(1))/(numel(t) - 1); want = sqrt(1 - p)/f;
    assert(abs(T - want) < 0.01*want, 'period %.4f s vs %.4f s', T, want);
    m = sprintf('momentum and energy kept; period %.3f s (free-free %.3f s)', T, want);
end

function m = t_chain_parity()
%T_CHAIN_PARITY  The twin's chains on the same noise-free inputs as the engine's
%   (engine/crates/adcs-sim-core/tests/chains.rs, the_star_tracker_chain_is_the_twins and
%   quadrant_currents_invert_at_known_angles): the same pair table, spots and attitude.
    cat = asils.devices.star_catalogue(4000); cam = asils.comp.star_tracker.camera(0.17); cam.noise = false;
    K = asils.comp.star_tracker.pairs(cat, cam.fov); K.R_head_nominal = eye(3);
    assert(numel(K.ang) == 452404 && K.i(1) == 3973 && K.j(1) == 3986, 'pair table');
    q = asils.quat.norm([0.1; -0.2; 0.3; 0.9]);
    S = asils.comp.star_tracker.centroid(asils.comp.star_tracker.render(asils.quat.dcm(q), cat, cam), cam);
    eng = [402.79524679120351, 342.01110964608188, 275610.86868389987; 706.03213561477514, 423.98417144594674, 42335.81788271133; ...
           773.00827469410081, 916.8667833984988, 14011.925350572936]';
    assert(size(S, 2) == 20, '%d spots', size(S, 2));
    d = S(:, [1 14 20]) - eng;
    assert(max(max(abs(d(1:2, :)))) < 1e-9 && max(abs(d(3, :)./eng(3, :))) < 1e-9, 'spots off the engine by %.3g px', max(max(abs(d(1:2, :)))));
    [qb, ok, info] = asils.comp.star_tracker.chain(q, eye(3), cat, K, cam);
    qe = [0.10259925829569796; -0.20519218143710233; 0.30779129307304492; 0.92338187159088136];
    e = asils.quat.angle(qb, qe)*180/pi*3600;
    assert(ok && info.used == 20 && e < 1e-6, 'attitude off the engine by %.3g arcsec', e);
    p = asils.comp.sun_sensor.head(); p.noise = 0;
    s = [tand(12); tand(-25); 1]; s = s/norm(s);
    I = asils.comp.sun_sensor.currents(s, p);
    Ie = [0.12298383871970874; 0.072995743368318142; 0.25847853897287454; 0.43548680351329888];
    assert(max(abs(I - Ie)) < 1e-15, 'currents off the engine by %.3g', max(abs(I - Ie)));
    m = sprintf('20 spots within 1e-9 px, attitude within %.1e arcsec, currents within 1e-15', e);
end

function m = t_chains_from_part()
%T_CHAINS_FROM_PART  model = 'image' and level = 'chain' read every value from the part:
%   one left out is refused by name, as is a model or level there is none of; the image
%   model answers through the device (render -> centroid -> identify -> QUEST), and the
%   Sun-sensor chain through its head.
    R = asils.util.root(); dd = fullfile(R, 'store', 'sized', sprintf('zz_test_%06d', randi(1e6)));
    mkdir(fullfile(dd, 'parts')); mkdir(fullfile(dd, 'products'));
    c = onCleanup(@() rmdir(dd, 's'));
    base = asils.util.readjson(fullfile(R, 'data', 'products', 'TRN-P-3U-IMG.json'));
    fills = base.fill; if ~iscell(fills), fills = num2cell(fills); end
    ist = find(cellfun(@(f) strcmp(f.slot, 'star_tracker'), fills)); isun = find(cellfun(@(f) strcmp(f.slot, 'sun_sensors'), fills));
    cam = struct('detector_px', 1024, 'psf_sigma_px', 1.2, 'flux_mag6_e', 3000, 'background_e', 50, 'read_noise_e', 8, ...
        'centroid_k_sigma', 5, 'max_spots', 20, 'id_tol_rad', 2e-4, 'id_mag_tol', 0.25, 'fit_tol_rad', 1e-4);
    hd = struct('aperture_side_m', 1e-3, 'aperture_height_m', 0.6e-3, 'current_noise_frac', 0.005, 'current_min_frac', 0.05);
    chain_parts_(R, dd, cam, hd, ''); chain_product_(dd, base, fills, ist, isun, 'image', 'chain');
    dv = asils.product.load('T-CHAINS');
    assert(strcmp(dv.st.model, 'image') && dv.st.camera.detector_px == 1024 && dv.sun.head.h == 0.6e-3, 'values from the part');
    keys = [fieldnames(cam); fieldnames(hd)];
    for k = 1:numel(keys)
        chain_parts_(R, dd, cam, hd, keys{k});
        try, asils.product.load('T-CHAINS'); error('not refused'); catch e
            assert(~isempty(strfind(e.message, keys{k})), '%s: %s', keys{k}, e.message);
        end
    end
    chain_parts_(R, dd, cam, hd, '');
    for bad = {{'pinhole', 'chain', 'the models are noise, quest and image'}, {'image', 'currents', 'the levels are model and chain'}}
        chain_product_(dd, base, fills, ist, isun, bad{1}{1}, bad{1}{2});
        try, asils.product.load('T-CHAINS'); error('not refused'); catch e
            assert(~isempty(strfind(e.message, bad{1}{3})), '%s', e.message);
        end
    end
    % the image model through the device, the head on body +y
    s = struct('boresight', [0; 1; 0], 'noise_cross', 1e-4, 'noise_roll', 1e-3, 'latency', 0, 'max_rate', 1, ...
        'sun_excl', 0.5, 'earth_excl', 0.3, 'fov', 0.17, 'model', 'image', 'moon_excl', 0.26, 'blind_s', 0, 'noise_rate_ref', 0.01);
    D = struct('hist_t', [], 'hist_q', [], 'dead', false, 'blind_until', -Inf, 'q_bias', [0;0;0;1], 'q_mis', [0;0;0;1]);
    D.cat = asils.devices.star_catalogue(4000); D.cam = asils.comp.star_tracker.camera(0.17, dv.st.camera);
    D.K = asils.comp.star_tracker.pairs(D.cat, 0.17);
    q = asils.quat.norm([0.2; 0.1; -0.3; 0.9]);
    [z, v] = asils.devices.star_tracker(q, 0, zeros(3,1), [0;-1;0], [0;-1;0], [0;0;1], 1, D, s);
    e = asils.quat.angle(q, z)*180/pi*3600;
    assert(v && e < 60, 'image model: valid %d, %.1f arcsec', v, e);
    % the Sun-sensor chain through its head
    ss = dv.sun; Ds = struct('n', ss.normals, 'bias', zeros(3, size(ss.normals, 2)));
    sb = [0.1; -0.2; 1]; sb = sb/norm(sb); w = 0;
    for k = 1:100
        [zs, ok] = asils.devices.sun_sensor(sb, 1, Ds, ss); assert(ok, 'Sun chain');
        w = max(w, acosd(min(1, zs'*sb)));
    end
    assert(w < 2, 'Sun chain worst %.2f deg', w);
    m = sprintf('%d values refused by name; image model %.1f arcsec; Sun chain worst %.2f deg', numel(keys), e, w);
end

function chain_parts_(R, dd, cam, hd, drop)
%CHAIN_PARTS_  T-ST and T-SUN: the synthetic parts with the chains' values, all but `drop`.
    p = asils.util.readjson(fullfile(R, 'data', 'parts', 'SYN-ST-1.json'));
    for k = fieldnames(cam)', if ~strcmp(k{1}, drop), p.nominal.(k{1}) = cam.(k{1}); end, end
    p.part_number = 'T-ST'; put_(fullfile(dd, 'parts', 'T-ST.json'), p);
    p = asils.util.readjson(fullfile(R, 'data', 'parts', 'SYN-SUN-1.json'));
    for k = fieldnames(hd)', if ~strcmp(k{1}, drop), p.nominal.(k{1}) = hd.(k{1}); end, end
    p.part_number = 'T-SUN'; put_(fullfile(dd, 'parts', 'T-SUN.json'), p);
end

function chain_product_(dd, base, fills, ist, isun, model, level)
%CHAIN_PRODUCT_  T-CHAINS: the imaging product on T-ST (model) and T-SUN (level).
    pr = base; f2 = fills; f2{ist}.part = 'T-ST'; f2{ist}.model = model; f2{isun}.part = 'T-SUN'; f2{isun}.level = level;
    pr.fill = f2; pr.id = 'T-CHAINS'; put_(fullfile(dd, 'products', 'T-CHAINS.json'), pr);
end

function m = t_physics_vectors()
    % translator = interpreter: the vectors the pseudocode's interpreter drew through the MATLAB
    % translations: the design's relations and every group's computing rows, one package
    % (+asils/+relations, tools/engine_build.py: each relation once, a group's copy of the library's
    % called where the package holds it, the file's `library`), and the language's own test
    % (+asils/+pcselftest, tools/pcode.py: every feature: records, procs with state, settling loops, tables).
    % Bit for bit where a function uses no transcendental; else within 1e-12 relative.
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

