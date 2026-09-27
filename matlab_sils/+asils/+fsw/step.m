function [F, out] = step(F, z, t, P, D) %#ok<INUSD>
%ASILS.FSW.STEP  One tick of the reference flight software (the loop's FSW node).
%
%   z  sensor outputs this tick: .w gyro, .B magnetometer, .clean (coils were
%      off when B was read), .sun_ok/.sun, .st_ok/.q_st, .gps_ok/.r_gps/.v_gps,
%      .h rotor momenta (tachometers / flow sensors), .delta gimbal angles
%   out  .m_body coil dipole, .cmd_r rotor hdot, .cmd_g gimbal rates, .duty thruster couples
%
%   Order inside a tick (Standard Code sim.run tick order, condensed):
%     1 onboard orbit (GNSS fix or two-body propagation)
%     2 attitude determination (MEKF predict; ST / Sun / field updates)
%     3 mode manager (transitions)
%     4 guidance -> control -> allocation (coils, momentum devices, thrusters)
%
%   Modes: detumble | nadir_mtq | nadir_fine | target_fine | slew_fine
    G = F.P; dt = P.sim.dt; dev = P.dev;
    jd = F.jd0 + t/86400;

    %% 1 onboard orbit ----------------------------------------------------
    if z.gps_ok
        F.r = z.r_gps; F.v = z.v_gps; F.t_fix = t;
    elseif ~isempty(F.r)
        a = -P.mu*F.r/norm(F.r)^3;
        F.r = F.r + F.v*dt + 0.5*a*dt^2; F.v = F.v + a*dt;
    end

    %% 2 attitude determination -------------------------------------------
    s_ref = asils.fsw.sun_model(jd);
    phase = mod(t + 1e-9, G.mtq_period);
    first = phase < dt - 1e-9;                          % start of an MTQ cycle (coils off)
    if first && ~isempty(F.r) && (t - F.t_Bref) >= 0.999
        if isempty(F.gh), F.gh = asils.env.igrf_gh(asils.util.decyear(jd), P.igrf); end
        C = asils.fsw.gmst_rot(jd);
        F.Bref = asils.env.field(C*F.r, C, F.gh, G.igrf_nmax); F.t_Bref = t;
    end
    usesAD = ~strcmp(F.mode, 'detumble');
    if usesAD
        if ~F.ad_ok
            if dev.st.fitted && z.st_ok
                h = find(z.st_valid, 1);
                F.K = asils.fsw.mekf_init(latency_(z.q_st(:,h), z.w, dev.st.latency), 1e-3, 2e-4, dev.gyro.arw, dev.gyro.rrw);
                F.ad_ok = true; F.t_st = t;
            elseif z.sun_ok && z.clean && ~isempty(F.Bref)
                q0 = asils.fsw.triad(z.sun, z.B, s_ref, F.Bref);
                F.K = asils.fsw.mekf_init(q0, 0.05, 2e-4, dev.gyro.arw, dev.gyro.rrw);
                F.ad_ok = true;
            end
        else
            F.K = asils.fsw.mekf_predict(F.K, z.w, dt);
            if dev.st.fitted && z.st_ok
                for h = find(z.st_valid)          % fuse every valid head
                    F.K = asils.fsw.mekf_quat(F.K, latency_(z.q_st(:,h), z.w - F.K.b, dev.st.latency), ...
                        dev.st.noise_cross*G.mekf.meas_scale, dev.st.noise_roll*G.mekf.meas_scale, dev.st.boresight(:,h));
                end
                F.t_st = t;
            elseif dev.st.fitted && t - F.t_st < G.st_coast_s
                % short star-tracker outage: coast on the gyro (coarse Sun /
                % field updates would pull a fine estimate off by a degree)
            else
                if z.sun_ok && first
                    F.K = asils.fsw.mekf_vector(F.K, z.sun, s_ref, G.mekf.sig_sun);
                end
                if z.clean && first && ~isempty(F.Bref)
                    F.K = asils.fsw.mekf_vector(F.K, z.B, F.Bref, G.mekf.sig_mag);
                end
            end
        end
    end
    if G.truth_knowledge && isfield(z, 'q_true')          % debug/trade switch only
        if isempty(F.K), F.K = asils.fsw.mekf_init(z.q_true, 1e-3, 1e-4, 1e-5, 1e-7); end
        F.K.q = z.q_true; F.K.b = zeros(3,1); F.ad_ok = true; z.w = z.w_true;
    end
    if F.ad_ok, w_raw = z.w - F.K.b; else, w_raw = z.w; end
    % first-order low-pass on the rate the controllers use (the D-term would
    % otherwise turn gyro angular-random-walk noise into white torque)
    a = dt/(G.rate_lpf_s + dt);
    if isempty(F.w_est) || G.rate_lpf_s <= 0, F.w_est = w_raw; else, F.w_est = F.w_est + a*(w_raw - F.w_est); end

    %% 3 mode manager -------------------------------------------------------
    if strcmp(F.mode, 'detumble') && ~isempty(G.auto_next)
        if norm(z.w) < G.detumble_exit, F.hold = F.hold + dt; else, F.hold = 0; end
        if F.hold >= G.detumble_hold_s
            F = enter_(F, G.auto_next, t);
        end
    end

    %% 4 guidance / control / commands ------------------------------------------
    m_body = F.m_hold; nr = F.M.nr; ng = F.M.ng;
    cmd_r = zeros(nr,1); cmd_g = zeros(ng,1); duty = zeros(1, F.nc);
    switch F.mode
        case 'detumble'
            % coil-off window: average the clean samples, then command for the
            % rest of the 1 s cycle (Standard Code ctrl.bdotScheduler duty).
            if phase < G.mtq_meas + dt/2
                m_body = zeros(3,1);
                if first, F.bsum = zeros(3,1); F.bn = 0; end
                F.bsum = F.bsum + z.B/norm(z.B); F.bn = F.bn + 1;
                if abs(phase - G.mtq_meas) < dt/2
                    b = F.bsum/F.bn; b = b/norm(b);
                    law = G.bdot_law; if strcmp(law, 'gyro') && ~dev.gyro.fitted, law = 'mag'; end
                    switch law
                        case 'gyro'      % db/dt = -w x b (Standard Code ctrl.bdot rate-feedback branch)
                            bd = -asils.util.cross3(z.w, b);
                            m_body = asils.fsw.bdot(b - bd, b, 1, norm(z.B), G.bdot_k, dev.mtq.m_max);
                        case 'mag'       % difference of consecutive coil-off windows
                            if ~isempty(F.b1), m_body = asils.fsw.bdot(F.b1, b, G.mtq_period, norm(z.B), G.bdot_k, dev.mtq.m_max); end
                        case 'bangbang'  % Standard Code ctrl.bdot TC == 0: full dipole against d(b)/dt
                            if ~isempty(F.b1)
                                bd = (b - F.b1)/G.mtq_period;
                                m_body = -dev.mtq.m_max*sign(bd).*(abs(bd) > 1e-4);
                            end
                    end
                    if any(m_body)
                        m_body = m_body - G.m_res_est;
                        m_body = m_body*min(1, dev.mtq.m_max/max(abs(m_body)));
                    end
                    F.b1 = b;
                end
            end
            if nr > 0                    % keep rotors at their bias, gimbals still
                cmd_r = -0.2*(z.h - F.h_t_rot);
                cmd_r(F.M.gi(:) > 0 & strcmp(dev.mex.kind(:), 'cmg')) = 0;
            end
        case 'nadir_mtq'
            if first
                m_body = zeros(3,1);
            elseif abs(phase - G.mtq_meas) < dt/2 && F.ad_ok && ~isempty(F.r)
                [F.q_ref, F.w_ref] = asils.fsw.guidance('nadir', F.r, F.v, t, F.gd);
                F.tau_req = asils.fsw.mtq_pd(F.K.q, F.w_est, F.q_ref, F.w_ref, G.mtq);
                m_body = asils.fsw.torque2dipole(F.tau_req, z.B, dev.mtq.m_max) - G.m_res_est;
                m_body = m_body*min(1, dev.mtq.m_max/max(abs(m_body)));   % residual-dipole compensation
            elseif phase < G.mtq_meas
                m_body = zeros(3,1);
            end
        case {'nadir_fine', 'target_fine', 'slew_fine'}
            kind = strrep(F.mode, '_fine', '');
            A = asils.plant.axes(F.M, z.delta);
            Hdev = A*z.h;
            % ---- FDIR: a fixed rotor that does not follow its command is isolated
            if nr > 0 && ~isempty(F.h_prev)
                % compare with what the device CAN do: the command clipped to its
                % torque limit, and never while it sits at its momentum limit
                meas = (z.h - F.h_prev)/dt;
                tmax = dev.mex.torque_max(:);
                expect = max(-0.8*tmax, min(0.8*tmax, F.cmd_r_prev));
                bad = abs(meas - expect) > 0.5*tmax & F.M.gi(:) == 0 & abs(expect) > 0.2*tmax ...
                      & abs(z.h) < 0.9*dev.mex.h_max(:);
                F.fd_count = (F.fd_count + dt).*bad;
                newly = F.fd_count > G.fdir_s & ~F.rot_failed(:);
                if any(newly)
                    F.rot_failed(newly) = true;
                    F.log(end+1).t = t; F.log(end).mode = sprintf('FDIR: rotor %d isolated', find(newly, 1));
                end
            end
            % ---- control law at the control rate
            if F.ad_ok && ~isempty(F.r) && t - F.last_ctrl >= G.rw.dt - 1e-9
                [qr, wr, wdr] = asils.fsw.guidance(kind, F.r, F.v, t, F.gd);
                F.q_ref = qr; F.w_ref = wr;
                [F.tau_req, F.I_q] = asils.fsw.control_law(F.K.q, F.w_est, F.q_ref, F.w_ref, F.I_q, G.rw.dt, G.rw, P.sc.I, Hdev, wdr);
                F.last_ctrl = t;
            end
            % ---- momentum management: coils (default) or thrusters
            dH = Hdev - F.H_t;
            if F.has_rcs_dump
                if norm(dH) > G.rcs.dump_hi, F.rcs_dumping = true; elseif norm(dH) < G.rcs.dump_lo, F.rcs_dumping = false; end
            end
            if first
                m_body = zeros(3,1);
            elseif abs(phase - G.mtq_meas) < dt/2
                m = zeros(3,1);
                if ~F.has_rcs_dump, m = asils.fsw.dump(z.h, A, F.H_t, z.B, G.dump_k, dev.mtq.m_max); end
                if F.idmas && F.ad_ok       % IDMAS split: coils take the torque across the field
                    m = m + asils.fsw.torque2dipole(F.tau_req, z.B, dev.mtq.m_max);
                end
                if F.ad_ok && any(F.rot_failed)   % a lost wheel axis is flown with the coils
                    fixed = find(F.M.gi(:) == 0 & ~F.rot_failed(:));
                    if isempty(fixed), un = F.tau_req;
                    else, un = F.tau_req - A(:, fixed)*(pinv(A(:, fixed))*F.tau_req); end
                    m = m + asils.fsw.torque2dipole(un, z.B, dev.mtq.m_max);
                end
                m_body = m - G.m_res_est;
                m_body = m_body*min(1, dev.mtq.m_max/max(abs(m_body)));
                F.B_dump = z.B;
            elseif phase < G.mtq_meas
                m_body = zeros(3,1);
            end
            % ---- thrusters: slew assist beyond the rotors' authority, and dumping
            tau_rcs = zeros(3,1);
            if F.nc > 0 && F.ad_ok
                req = zeros(3,1);
                if G.rcs.assist
                    % torque beyond the momentum devices' authority ...
                    ex = abs(F.tau_req) - G.rcs.assist_frac*F.cap;
                    req = sign(F.tau_req).*max(0, ex);
                    % ... and every axis whose devices are near momentum saturation
                    % while the request would push them further (rotors absorb -tau)
                    full = abs(Hdev) > G.rcs.assist_frac*F.hcap & sign(-F.tau_req) == sign(Hdev);
                    req(full) = F.tau_req(full);
                end
                if F.rcs_dumping, req = req - G.rcs.dump_k*dH; end
                [duty, tau_rcs] = asils.fsw.rcs_duty(req, dev.rcs, dt);
            end
            % ---- the momentum devices deliver the rest (feedforward of the known coil and thruster torque)
            if any(m_body), tau_coil = asils.util.cross3(m_body + G.m_res_est, F.B_dump); else, tau_coil = zeros(3,1); end
            if F.ad_ok && ~isempty(F.r) && nr > 0
                [cmd_r, cmd_g] = asils.fsw.allocate(F.tau_req - tau_coil - tau_rcs, z, F, P);
            end
            F.h_prev = z.h; F.cmd_r_prev = cmd_r;
    end
    F.m_hold = m_body;
    out.m_body = m_body; out.cmd_r = cmd_r; out.cmd_g = cmd_g; out.duty = duty;
end

function F = enter_(F, mode, t)
    F.mode = mode; F.t_mode = t; F.hold = 0; F.I_q = zeros(3,1);
    F.log(end+1).t = t; F.log(end).mode = mode;
end

function q = latency_(q_st, w, lat)
%LATENCY_  Bring a star-tracker attitude from t - latency to t with the gyro
%   (Standard Code sdp.starTrackerLatency, constant-rate form).
    q = asils.quat.norm(asils.quat.mult(q_st, asils.quat.fromrotvec(w*lat)));
end
