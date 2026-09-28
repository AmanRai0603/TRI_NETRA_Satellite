function ok = run_all_tests()
%RUN_ALL_TESTS  Unit and integration tests of the TRI-NETRA ADCS SILS.
%   >> startup_asils; addpath tests; run_all_tests
%   Each test prints PASS/FAIL with the number it checked.
%   Copyright (c) 2026 Agastya. All rights reserved.
    T = {@t_quat, @t_kinematics, @t_sso, @t_case, @t_igrf, @t_shadow, ...
         @t_torques, @t_plant_conservation, @t_cmg_plant, @t_mekf, @t_quest, @t_lqr, ...
         @t_fmr_spin_down, @t_cmg_steering, @t_hal_loopback, @t_select, @t_gen_bdot, @t_sun_spin_law, @t_sun_guidance, @t_sun_model, @t_st_chain, @t_sun_chain, @t_es_chain, ...
         @t_sizing, @t_modes_table, @t_orbit_vs_pop, @t_short_runs};
    n = 0;
    for i = 1:numel(T)
        name = func2str(T{i});
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
