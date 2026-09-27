function rec = run(scenarioId, caseFile, varargin)
%ASILS.RUN  One closed-loop SILS run: case + scenario + product -> recording.
%
%   rec = asils.run('nadir_hold_ais', 'cases/ais_3u.csv')
%   rec = asils.run(..., 'seed', 3, 'set', struct('env__F107', 180), 'quiet', true)
%
%   THE LOOP (one tick = P.sim.dt):
%     orbit     truth r,v from the IN-LOOP Precision Orbit Propagator (asils.orbit)
%     env       B (IGRF at the POP position), Sun + shadow (DE440), density (POP
%               DTM2020), atmosphere-relative velocity -- refreshed at P.env.dt_s
%     sensors   gyro, magnetometer, sun sensors, star tracker, GNSS (asils.devices)
%     fsw       onboard orbit, MEKF, mode manager, guidance, control (asils.fsw)
%     actuators magnetorquers, reaction wheels (asils.devices)
%     torques   gravity gradient, aero, SRP, residual dipole (asils.env.torques),
%               all driven by the orbit state -- "the OD information flows"
%     plant     rigid body + wheels, RK4 (asils.plant)
%     record    every P.sim.record_dt
%
%   Copyright (c) 2026 Agastya. All rights reserved.
    o = struct('seed', 1, 'quiet', false);
    for k = 1:2:numel(varargin), o.(varargin{k}) = varargin{k+1}; end
    P = asils.config(scenarioId, caseFile, o);
    rng(P.seed, 'twister');
    tic_all = tic;
    if ~o.quiet
        fprintf('[asils] %s on %s (%s), seed %d, %.0f s\n', P.id, P.case.id, P.dev.id, P.seed, P.sim.duration_s);
    end

    %% initialise nodes ----------------------------------------------------
    O = asils.orbit.init(P);
    D = asils.devices.init(P);
    P.igrf = asils.env.igrf_coefs();
    jd0 = asils.util.jd(P.epoch_utc);
    gh = asils.env.igrf_gh(asils.util.decyear(jd0), P.igrf);
    G = asils.env.geometry(P.sc);
    dev = P.dev;
    % momentum-exchange devices: TRUE geometry (misaligned) for the plant;
    % the FSW holds the nominal one (asils.fsw.init)
    Mt = asils.plant.geometry(zeros(3,0));
    if dev.mex.fitted, Mt = asils.plant.geometry(D.mex.A0, dev.mex.G, dev.mex.gi); end
    nr = Mt.nr; ng = Mt.ng;
    nc = 0; if dev.rcs.fitted, nc = size(dev.rcs.tau_couple, 2); end
    Acoil_p = pinv(dev.mtq.axes);
    I = P.sc.I; Iinv = P.sc.Iinv;
    F = asils.fsw.init(P, jd0);
    H = asils.hal.open(P);                        % the FSW boundary: SILS / loopback / OILS / HILS
    faults = asils.util.getf(P, 'faults', []);

    [r, v] = asils.orbit.state(O, 0);
    [q0, w0] = initial_(P, r, v, F);
    h0 = asils.util.getf(P.scenario.initial, 'wheel_momentum_Nms', NaN);
    if isnan(h0), h0v = F.h_t_rot; else, h0v = h0*ones(nr, 1); end
    x = [q0; w0; h0v; zeros(ng, 1)];

    %% recording buffers ----------------------------------------------------------
    dt = P.sim.dt; N = round(P.sim.duration_s/dt);
    every = max(1, round(P.sim.record_dt/dt)); M = floor(N/every) + 2;
    Z = @(n) nan(n, M);
    R_ = struct('t', Z(1), 'q', Z(4), 'w', Z(3), 'q_est', Z(4), 'q_ref', Z(4), 'w_ref', Z(3), ...
        'w_meas', Z(3), 'h_w', Z(max(nr,1)), 'cmd_r', Z(max(nr,1)), 'hdot', Z(max(nr,1)), 'delta', Z(max(ng,1)), 'm', Z(3), 'tau_dist', Z(12), 'tau_mtq', Z(3), ...
        'tau_rw', Z(3), 'tau_rcs', Z(3), 'tau_req', Z(3), 'mode', Z(1), 'P_mtq', Z(1), 'P_rw', Z(1), 'P_rcs', Z(1), ...
        'prop_kg', Z(1), 'r', Z(3), 'v', Z(3), 'rho', Z(1), 'nu', Z(1), 'B', Z(3), 'B_meas', Z(3), ...
        'sun_ok', Z(1), 'st_ok', Z(1), 'ad_ok', Z(1), 'sun_eci', Z(3), 'sun_body', Z(3), 'n_failed', Z(1));
    modes = {'detumble', 'nadir_mtq', 'nadir_fine', 'target_fine', 'slew_fine'};

    env_every = max(1, round(P.env.dt_s/dt));
    st_every = 1; if dev.st.fitted, st_every = max(1, round(1/(dev.st.rate_hz*dt))); end
    gps_every = max(1, round(1/dt));
    m_B = zeros(3,1); P_mtq = 0; P_mex = 0; P_rcs = 0; hdot = zeros(nr,1); gdot = zeros(ng,1);
    tau_rcs = zeros(3,1); prop = 0;
    z = struct('gps_ok', false, 'st_ok', false, 'st_valid', false, 'sun_ok', false, 'clean', true, ...
               'q_st', [0;0;0;1], 'sun', [1;0;0], 'r_gps', [], 'v_gps', [], 'h', zeros(nr,1), 'delta', zeros(ng,1));
    j = 0; tprint = 0; parts = zeros(3,4); Eacc = [0 0 0]; off = floor(every/2);
    for k = 0:N
        t = k*dt;
        [r, v, O] = asils.orbit.state(O, t);
        if mod(k, env_every) == 0
            X = asils.orbit.context(O, t);
            [B_eci, ~] = asils.env.field(X.C*r, X.C, gh, P.env.igrf_nmax);
            sun_rel = X.sun_eci - r;
            nu = asils.env.shadow(r, X.sun_eci);
            v_rel = v - O.omega_e*[-r(2); r(1); 0];
            rho = X.rho; Psrp = X.P_srp;
            O.W.sc.R_bi = asils.quat.dcm(x(1:4))';     % attitude -> POP (box-wing models read it)
        end
        if ~isempty(faults), [D, F] = asils.faults.apply(faults, t, D, F); end
        q = x(1:4); w = x(5:7); h = x(8:7+nr); d = x(8+nr:7+nr+ng);
        Rb = asils.quat.dcm(q);
        B_B = Rb*B_eci;
        sB = Rb*sun_rel; sB = sB/norm(sB); nB = -Rb*r/norm(r);

        %% sensors (engineering values, then across the HAL as the FSW reads them)
        if dev.gyro.fitted, [z.w, D.gyro] = asils.devices.gyro(w, D.gyro, dev.gyro, dt); else, z.w = w; end
        z.clean = norm(m_B) == 0;
        z.B = asils.devices.magnetometer(B_B, D.mag, dev.mag, m_B);
        if dev.sun.fitted
            [z.sun, z.sun_ok] = asils.devices.sun_sensor(sB, nu, D.sun, dev.sun);
        elseif dev.css.fitted
            [z.sun, z.sun_ok] = asils.devices.css(sB, nu, nB, asin(6378137/norm(r)), D.css, dev.css);
        end
        z.st_ok = false;
        if dev.st.fitted, D.st = asils.devices.st_history(D.st, t, q, 0.5); end
        if dev.st.fitted && mod(k, st_every) == 0
            [z.q_st, z.st_valid, D.st] = asils.devices.star_tracker(q, t, w, sB, nB, asin(6378137/norm(r)), D.st, dev.st);
            z.st_ok = any(z.st_valid);
        end
        z.gps_ok = dev.gps.fitted && mod(k, gps_every) == 0 && ~D.gps_dead;
        if z.gps_ok, [z.r_gps, z.v_gps] = asils.devices.gps(r, v, dev.gps); end
        if nr > 0, z.h = h + 1e-7*randn(nr,1); end
        if ng > 0, z.delta = d + 1e-5*randn(ng,1); end
        z.q_true = q; z.w_true = w;
        z = asils.hal.sensors(H, z, t);

        %% flight software (behind the HAL)
        [F, out] = asils.fsw.step(F, z, t, P, D);
        out = asils.hal.actuators(H, out, t);

        %% actuators
        [m_B, P_mtq] = asils.devices.mtq(Acoil_p*out.m_body, D.mtq, dev.mtq);
        if nr > 0, [hdot, gdot, P_mex, D.mex] = asils.devices.mex(out.cmd_r, out.cmd_g, h, d, D.mex, dev.mex, dt); end
        if nc > 0
            [tau_rcs, mdot, P_rcs] = asils.devices.rcs(out.duty, D.rcs, dev.rcs, dt);
            prop = prop + mdot*dt;
            if prop >= dev.rcs.propellant_kg, D.rcs.failed(:) = true; end     % tank empty
        end

        %% environment torques from the orbit state, then the plant
        [tau_d, parts] = asils.env.torques(q, r, v_rel, B_eci, sun_rel, nu, Psrp, rho, I, G, P.sc.m_res, P.mu, P.env.on);
        tau_mtq = [m_B(2)*B_B(3)-m_B(3)*B_B(2); m_B(3)*B_B(1)-m_B(1)*B_B(3); m_B(1)*B_B(2)-m_B(2)*B_B(1)];

        Eacc = Eacc + [P_mtq P_mex P_rcs];
        if mod(k, every) == off || k == 0
            j = j + 1;
            if k > 0, Pavg = Eacc/every; else, Pavg = [P_mtq P_mex P_rcs]; end
            Eacc = [0 0 0];
            R_.t(j) = t; R_.q(:,j) = q; R_.w(:,j) = w; R_.w_meas(:,j) = z.w;
            if F.ad_ok, R_.q_est(:,j) = F.K.q; end
            R_.q_ref(:,j) = F.q_ref; R_.w_ref(:,j) = F.w_ref; R_.tau_req(:,j) = F.tau_req;
            if nr > 0
                R_.h_w(:,j) = h; R_.tau_rw(:,j) = -asils.plant.axes(Mt, d)*hdot;
                R_.cmd_r(:,j) = out.cmd_r; R_.hdot(:,j) = hdot;
                R_.n_failed(j) = sum(F.rot_failed);
            end
            if ng > 0, R_.delta(:,j) = d; end
            R_.tau_rcs(:,j) = tau_rcs; R_.prop_kg(j) = prop;
            R_.m(:,j) = m_B; R_.tau_dist(:,j) = parts(:); R_.tau_mtq(:,j) = tau_mtq;
            R_.mode(j) = find(strcmp(modes, F.mode)); R_.P_mtq(j) = Pavg(1); R_.P_rw(j) = Pavg(2); R_.P_rcs(j) = Pavg(3);
            R_.r(:,j) = r; R_.v(:,j) = v; R_.rho(j) = rho; R_.nu(j) = nu; R_.B(:,j) = B_B; R_.B_meas(:,j) = z.B;
            R_.sun_ok(j) = z.sun_ok; R_.st_ok(j) = z.st_ok; R_.ad_ok(j) = F.ad_ok;
            R_.sun_eci(:,j) = sun_rel/norm(sun_rel); R_.sun_body(:,j) = sB;
        end
        if k == N, break, end
        x = asils.plant.step(x, dt, I, Iinv, Mt, tau_d + tau_mtq + tau_rcs, hdot, gdot);
        H = asils.hal.pace(H, t);                  % real-time pacing for OILS / HILS

        if ~o.quiet && toc(tic_all) - tprint > 30
            tprint = toc(tic_all);
            fprintf('  t = %7.0f / %.0f s  mode %-11s |w| %.3f deg/s  (%.0f s wall)\n', t, P.sim.duration_s, F.mode, norm(w)*180/pi, tprint);
        end
    end
    asils.hal.close(H);
    fn = fieldnames(R_);
    for i = 1:numel(fn), R_.(fn{i}) = R_.(fn{i})(:, 1:j); end
    rec = R_;
    rec.P = rmfield(P, 'igrf'); rec.modes = modes; rec.mode_log = F.log;
    rec.orbit = struct('raan_rad', O.raan_rad, 'inc_rad', O.inc_rad, 'a_m', O.a_m);
    rec.wall_s = toc(tic_all);
    rec = asils.metrics.derive(rec);
    rec.metrics = asils.metrics.evaluate(rec);
    if ~o.quiet
        fprintf('[asils] done in %.0f s wall\n', rec.wall_s);
        asils.metrics.print(rec);
    end
end

function [q0, w0] = initial_(P, r, v, F)
    S = P.scenario.initial;
    [q_ref, ~] = asils.fsw.guidance('nadir', r, v, 0, F.gd);
    switch S.attitude.kind
        case 'random'
            q0 = asils.quat.norm(randn(4,1));
        case 'error_from_target'
            ax = S.attitude.axis_body(:); ax = ax/norm(ax);
            q0 = asils.quat.norm(asils.quat.mult(q_ref, asils.quat.fromrotvec(S.attitude.angle_deg*pi/180*ax)));
        otherwise
            q0 = q_ref;
    end
    switch S.rate.kind
        case 'random_direction'
            mag = S.rate.magnitude_deg_s;
            if ischar(mag), mag = P.case.v.(strrep(strrep(mag, 'case:', ''), '.', '_')); end
            dd = randn(3,1); w0 = dd/norm(dd)*mag*pi/180;
        case 'lvlh'
            Rq = asils.quat.dcm(q0); w0 = Rq*(asils.util.cross3(r, v)/(r'*r));
        otherwise
            w0 = S.rate.value_deg_s(:)*pi/180;
    end
end
