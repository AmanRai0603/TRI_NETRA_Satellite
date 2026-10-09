function rec = run(scenarioId, caseFile, varargin)
%ASILS.RUN  One closed-loop SILS run: case + scenario + product -> recording (the twin's runner, the MATLAB twin of the
%   engine's adcs-sim run.rs).
%
%   rec = asils.run('nadir_hold_ais', 'cases/ais_3u.csv')
%   rec = asils.run(..., 'seed', 3, 'set', struct('engine__f107', 180), 'quiet', true)
%
%   THE LOOP (one tick = the scenario's control step):
%     orbit      truth r, v from the IN-LOOP Precision Orbit Propagator (asils.orbit)
%     env        the field (IGRF at the POP position), Sun, Moon, shadow, density, SRP pressure, atmosphere-relative
%                velocity, refreshed at the design's step (env_refresh_step)
%     sensors    each unit sampled and written onto the bus as its part's bytes (asils.devices.sense)
%     fsw        the flight software on the bus, behind the HAL (asils.hal.exchange: asils.fsw.step)
%     actuators  the commands decoded from the bus and applied (asils.hal.commands, asils.devices.actuate)
%     torques    gravity gradient, aero, radiation, residual dipole (asils.env.torques), the coils' m x B
%     plant      rigid body + rotors + flexible mode, RK4 (asils.plant.step)
%     record     every record step
%   Every model is the design's, generated into +asils/+models (the plant, the environment, the units, the emulators'
%   scaling, the start) and +asils/+alg (the flight software, tools/flight_build.py); the flight software boots from the
%   blob the engine builds for the same run (asils.config). Its random draws are the language's streams (SplitMix64 over
%   a counter, asils.devices.stream), named as the engine names them: the same seed gives the same draws in both. What is
%   written here is the runner: the order of a tick, the recording, the pacing and the HAL's plumbing.
%   Owner: Agastya. Copyright (c) 2026 Agastya. All rights reserved.
    o = struct('seed', 1, 'quiet', false);
    for k = 1:2:numel(varargin), o.(varargin{k}) = varargin{k+1}; end
    P = asils.config(scenarioId, caseFile, o);
    tic_all = tic;
    if ~o.quiet
        fprintf('[asils] %s on %s (%s), seed %d, %.0f s\n', P.id, P.case.id, P.dev.id, P.seed, P.sim.duration_s);
    end
    dev = P.dev; p = P.fsw;
    % the engine has the fast analytic orbit too (engine.orbit = fast); the twin flies the POP
    assert(strcmp(P.orbit.model, 'pop'), 'asils:run:orbit', 'the twin flies the precision orbit (POP); engine.orbit = %s is the engine''s', P.orbit.model);

    %% the truth orbit and environment, the units, the plant, the flight software --------------------------
    O = asils.orbit.init(P);
    gh = asils.models.frames.igrf_gh(asils.models.caltime.decimal_year(P.jd0));
    facets = asils.models.facets.facets_box(P.sc.box_m, P.sc.cm_offset_m, P.sc.sigma_n, P.sc.sigma_t, P.sc.vb_ratio, P.sc.refl, P.sc.spec_frac);
    U = asils.devices.init(dev, P.seed);
    nr = dev.mex.rec.n; ng = dev.mex.rec.ng;
    % plant: the TRUE (dispersed) rotor axes; the software holds the nominal ones
    a0 = zeros(8, 3); a0(1:nr, :) = U.mex.a0(1:nr, :);
    g0 = zeros(4, 3); g0(1:ng, :) = dev.mex.rec.g(1:ng, :);
    geo = asils.models.rotors.rotor_geometry(a0, g0, dev.mex.rec.gi, nr, ng);
    B = asils.plant.body(P.sc.I, geo, P.sc.flex);
    H = asils.hal.open(P);                         % the FSW boundary: SILS / loopback / OILS / HILS
    bus = asils.hal.bus();
    F = asils.fsw.init(P.blob, 0);
    faults = P.faults; fdone = false(1, numel(faults));

    [r, v, O] = asils.orbit.state(O, 0);
    x = initial_state_(P, p, r, v, nr);

    %% recording buffers -------------------------------------------------------------------------------
    dt = P.sim.dt; N = round(P.sim.duration_s/dt);
    every = max(1, round(P.sim.record_dt/dt)); off = floor(every/2); M = floor(N/every) + 2;
    rt = struct('env', max(1, round(P.env.dt_s/dt)), 'st', 1, 'gps', max(1, round(1/dt)), 'es', 1);
    if dev.st.fitted, rt.st = max(1, round(1/(dev.st.rate_hz*dt))); end
    if dev.es.fitted, rt.es = max(1, round(1/(dev.es.rate_hz*dt))); end
    if dev.gps.fitted && dev.gps.latency/dt + 2 > 256
        error('asils:run:refused', 'GNSS latency %g s is %.0f control steps; the twin keeps %d steps of history', dev.gps.latency, dev.gps.latency/dt, 254);
    end
    tmax = dev.mex.rec.torque_max; gmax = p.gim_rate_max;
    Z = @(n) nan(n, M);
    R_ = struct('t', Z(1), 'q', Z(4), 'w', Z(3), 'q_est', Z(4), 'q_ref', Z(4), 'w_ref', Z(3), ...
        'w_meas', Z(3), 'h_w', Z(max(nr,1)), 'cmd_r', Z(max(nr,1)), 'hdot', Z(max(nr,1)), 'delta', Z(max(ng,1)), 'm', Z(3), 'tau_dist', Z(12), 'tau_mtq', Z(3), ...
        'tau_rw', Z(3), 'tau_rcs', Z(3), 'tau_req', Z(3), 'mode', Z(1), 'P_mtq', Z(1), 'P_rw', Z(1), 'P_rcs', Z(1), ...
        'prop_kg', Z(1), 'r', Z(3), 'v', Z(3), 'rho', Z(1), 'nu', Z(1), 'B', Z(3), 'B_meas', Z(3), ...
        'sun_ok', Z(1), 'st_ok', Z(1), 'ad_ok', Z(1), 'sun_eci', Z(3), 'sun_body', Z(3), 'n_failed', Z(1));
    MT = asils.fsw.modes(); modes = MT.state;
    a = struct('m_b', zeros(3, 1), 'm_coil', zeros(3, 1), 'hdot', zeros(8, 1), 'gdot', zeros(4, 1), 'tau_rcs', zeros(3, 1), ...
               'p_mtq', 0, 'p_mex', 0, 'p_rcs', 0);
    prop = 0; eacc = [0 0 0];
    b_eci = zeros(3, 1); sun_rel = [1; 0; 0]; moon_rel = [0; 1; 0]; nu = 1; v_rel = zeros(3, 1); rho = 0; psrp = 0;
    log = struct('t', {}, 'mode', {}); last_mode = modes{p.start_mode + 1};
    j = 0; tprint = 0;
    % a long run may be cut (a machine restarted): with o.checkpoint (a file) the loop's state is kept there every
    % o.checkpoint_s of wall time, and a run that finds it goes on from it (the same run: every state is in it)
    ckpt = asils.util.getf(o, 'checkpoint', ''); ckpt_every = asils.util.getf(o, 'checkpoint_s', 600);
    k0 = 0; wall0 = 0; tcp = tic;
    if ~isempty(ckpt) && exist(ckpt, 'file') == 2
        C_ = load(ckpt);
        k0 = C_.k + 1; wall0 = C_.wall;
        O = C_.O; U = C_.U; F = C_.F; bus = C_.bus; x = C_.x; R_ = C_.R_; a = C_.a; prop = C_.prop; eacc = C_.eacc;
        b_eci = C_.b_eci; sun_rel = C_.sun_rel; moon_rel = C_.moon_rel; nu = C_.nu; v_rel = C_.v_rel; rho = C_.rho; psrp = C_.psrp;
        log = C_.log; last_mode = C_.last_mode; j = C_.j; fdone = C_.fdone;
        clear C_
        if ~o.quiet, fprintf('  resumed from %s at t = %.1f s\n', ckpt, k0*dt); end
    end
    for k = k0:N
        t = k*dt;
        [r, v, O] = asils.orbit.state(O, t);
        if mod(k, rt.env) == 0
            E = asils.orbit.env(O, t, P.jd0, r, v, gh, P.env.igrf_nmax);
            b_eci = E.b_eci; sun_rel = E.sun_rel; moon_rel = E.moon_rel; nu = E.nu; v_rel = E.v_rel; rho = E.rho; psrp = E.p_srp;
            O.W.sc.R_bi = asils.la.dcm(x.q).';               % attitude -> POP (box-wing / panel models read it)
        end
        if ~isempty(faults), [U, fdone, log] = asils.faults.apply(faults, t, U, fdone, log); end
        rb = asils.la.dcm(x.q);
        % what the sensors see of the sky: sens_sky_view's sky_view
        sky = struct();
        [sky.b_b, sky.sb, sky.mb, sky.nb, sky.earth_ang] = asils.models.skyview.sky_view(rb, b_eci, sun_rel, moon_rel, r);

        %% sensors -> bytes; the flight software; bytes -> commands
        [U, bus, z] = asils.devices.sense(U, dev, bus, x, sky, a.m_coil, nu, t, k, rt, r, v, P.jd0, dt, H.scale);
        [F, bus, H] = asils.hal.exchange(H, F, bus);
        cmd = asils.hal.commands(bus, p.m_max, tmax, gmax, dt, H.scale, nr, ng, p.nc);
        bus.can_tx = zeros(0, 10); bus.can_rx = zeros(0, 10);
        mode = F.mode; ad_ok = F.ad_ok;
        last_mode = modes{mode + 1};
        if isempty(log) || ~strcmp(log(end).mode, last_mode), log(end+1) = struct('t', t, 'mode', last_mode); end %#ok<AGROW>

        %% actuators, torques, the plant
        [U, a, prop] = asils.devices.actuate(U, dev, cmd, x, a, prop, dt);
        parts = asils.env.torques(x.q, r, v_rel, b_eci, sun_rel, nu, psrp, rho, P.sc.I, facets, P.sc.m_res, P.mu, P.env.on);
        tau_d = (parts(:, 1) + parts(:, 2)) + (parts(:, 3) + parts(:, 4));
        tau_mtq = asils.models.coiltorque.coil_torque(a.m_b, sky.b_b);
        eacc = [eacc(1) + a.p_mtq, eacc(2) + a.p_mex, eacc(3) + a.p_rcs];
        if mod(k, every) == off || k == 0
            j = j + 1;
            if k > 0, pavg = eacc/every; else, pavg = [a.p_mtq a.p_mex a.p_rcs]; end
            eacc = [0 0 0];
            R_.t(j) = t; R_.q(:,j) = x.q; R_.w(:,j) = x.w; R_.w_meas(:,j) = z.w_meas;
            if ad_ok, R_.q_est(:,j) = F.K.q; end
            R_.q_ref(:,j) = F.q_ref; R_.w_ref(:,j) = F.w_ref; R_.tau_req(:,j) = F.tau_req;
            if nr > 0
                ax = asils.models.rotors.rotor_axes(geo, x.d);
                tr = zeros(3, 1);
                for i = 1:nr, for c = 1:3, tr(c) = tr(c) - ax(i, c)*a.hdot(i); end, end
                R_.h_w(:,j) = x.h(1:nr); R_.tau_rw(:,j) = tr;
                R_.cmd_r(:,j) = cmd.cmd_r(1:nr); R_.hdot(:,j) = a.hdot(1:nr);
                R_.n_failed(j) = sum(bitget(F.faults, 1:16));
            end
            if ng > 0, R_.delta(:,j) = x.d(1:ng); end
            R_.tau_rcs(:,j) = a.tau_rcs; R_.prop_kg(j) = prop;
            R_.m(:,j) = a.m_b; R_.tau_dist(:,j) = parts(:); R_.tau_mtq(:,j) = tau_mtq;
            R_.mode(j) = mode + 1; R_.P_mtq(j) = pavg(1); R_.P_rw(j) = pavg(2); R_.P_rcs(j) = pavg(3);
            R_.r(:,j) = r; R_.v(:,j) = v; R_.rho(j) = rho; R_.nu(j) = nu; R_.B(:,j) = sky.b_b; R_.B_meas(:,j) = z.b_meas;
            R_.sun_ok(j) = z.sun_ok; R_.st_ok(j) = z.st_ok; R_.ad_ok(j) = ad_ok;
            R_.sun_eci(:,j) = unit_(sun_rel); R_.sun_body(:,j) = sky.sb;
        end
        if k == N, break, end
        x = asils.plant.step(x, dt, B, tau_d + tau_mtq + a.tau_rcs, a.hdot, a.gdot);
        H = asils.hal.pace(H, t);                  % real-time pacing for OILS / HILS
        if ~isempty(ckpt) && toc(tcp) > ckpt_every
            wall = wall0 + toc(tic_all); %#ok<NASGU>
            save('-binary', [ckpt '.tmp'], 'k', 'wall', 'O', 'U', 'F', 'bus', 'x', 'R_', 'a', 'prop', 'eacc', 'b_eci', 'sun_rel', ...
                 'moon_rel', 'nu', 'v_rel', 'rho', 'psrp', 'log', 'last_mode', 'j', 'fdone');
            movefile([ckpt '.tmp'], ckpt, 'f');
            tcp = tic;
        end

        if ~o.quiet && toc(tic_all) - tprint > 30
            tprint = toc(tic_all);
            fprintf('  t = %7.0f / %.0f s  mode %-13s |w| %.3f deg/s  (%.0f s wall)\n', t, P.sim.duration_s, last_mode, ...
                    sqrt(x.w(1)^2 + x.w(2)^2 + x.w(3)^2)*180/pi, tprint);
        end
    end
    asils.hal.close(H);
    if ~isempty(ckpt) && exist(ckpt, 'file') == 2, delete(ckpt); end      % the run is whole: nothing to go on from
    fn = fieldnames(R_);
    for i = 1:numel(fn), R_.(fn{i}) = R_.(fn{i})(:, 1:j); end
    rec = R_;
    rec.P = P; rec.modes = modes; rec.mission_modes = MT.mission; rec.mode_log = log;
    rec.orbit = struct('raan_rad', O.raan_rad, 'inc_rad', O.inc_rad, 'a_m', O.a_m);
    rec.wall_s = wall0 + toc(tic_all);
    rec = asils.metrics.derive(rec);
    rec.metrics = asils.metrics.evaluate(rec);
    if ~o.quiet
        fprintf('[asils] done in %.0f s wall\n', rec.wall_s);
        asils.metrics.print(rec);
    end
end

function x = initial_state_(P, p, r, v, nr)
% The initial attitude and rate the scenario asks for, and the rotor momenta: dyn_initial_state's initial_state
% (asils.models.initstate), its draws from the run's initial stream. Read here: the scenario's kinds and numbers (its
% degrees, its defaults: dyn's stated values) and the flight software's guidance references, nadir's and the start mode's.
    ini = asils.util.getf(P.scenario, 'initial', struct());
    att = asils.util.getf(ini, 'attitude', struct()); rate = asils.util.getf(ini, 'rate', struct());
    gd = struct('q_off', p.gd_q_off, 'roll_deg', p.gd_roll_deg, 't0', p.gd_t0, 't_slew', p.gd_T, 'axis', p.gd_axis, ...
                'q_inertial', p.gd_q_inertial, 'sun_axis', p.sun_axis, 'roll_axis', p.roll_axis, ...
                'sun_eci', asils.alg.frames.sun_model(P.jd0), 'flip', false);
    if p.gd_yaw_flip ~= 0
        gd.flip = asils.alg.guidance.yaw_flip(r, v, gd.q_off, gd.sun_axis, gd.roll_axis, gd.sun_eci, gd.flip, p.gd_flip_hyst);
    end
    gfn = @(kind) asils.alg.guidance.guidance(kind, r, v, 0.0, gd.q_off, gd.roll_deg, gd.t0, gd.t_slew, gd.axis, gd.q_inertial, ...
                                              gd.sun_axis, gd.roll_axis, gd.sun_eci, gd.flip);
    q_nad = gfn(0);
    has_g = P.gd_kind0 >= 0; q_g = [0; 0; 0; 1]; w_g = zeros(3, 1);
    if has_g, [q_g, w_g] = gfn(P.gd_kind0); end
    d = P.start;
    switch asils.util.getf(att, 'kind', '')        % initstate's AttStart: nadir, random, error_from_target, error_from_guidance
        case 'nadir', ak = 0; case 'random', ak = 1; case 'error_from_target', ak = 2; case 'error_from_guidance', ak = 3;
        otherwise, ak = d.att_kind;
    end
    axis_ = asils.util.getf(att, 'axis_body', d.axis); axis_ = axis_(:);
    angle = asils.util.getf(att, 'angle_deg', d.angle_deg)*(pi/180);        % Rust's to_radians: x (pi/180)
    switch asils.util.getf(rate, 'kind', '')       % initstate's RateStart: body, random_direction, lvlh, guidance
        case 'random_direction', rk = 1; case 'lvlh', rk = 2; case 'guidance', rk = 3; case 'body', rk = 0;
        otherwise, rk = d.rate_kind;
    end
    mag = asils.util.getf(rate, 'magnitude_deg_s', d.magnitude_deg_s);
    if ischar(mag), mag = P.case.v.(strrep(strrep(mag, 'case:', ''), '.', '_')); end
    mag = mag*(pi/180);
    extra = asils.util.getf(rate, 'extra_deg_s', d.extra_deg_s)*(pi/180);
    value = asils.util.getf(rate, 'value_deg_s', d.rate_deg_s); value = value(:)*(pi/180.0);
    h0 = asils.util.getf(ini, 'wheel_momentum_Nms', NaN);
    s = asils.devices.stream(P.seed, 'initial');
    [q, w, h] = asils.models.initstate.initial_state(ak, q_nad, has_g, q_g, w_g, axis_, angle, rk, r, v, value, mag, extra, nr, h0, P.h_t_rot, s);
    x = asils.models.rigidbody.PlantState_zero();
    x.q = q; x.w = w; x.h = h;
end

function u = unit_(a)
    n = sqrt(a(1)*a(1) + a(2)*a(2) + a(3)*a(3));
    if n < 1e-300, u = zeros(3, 1); else, s = 1.0/n; u = [a(1)*s; a(2)*s; a(3)*s]; end
end
