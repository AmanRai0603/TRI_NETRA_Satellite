function [F, m_body, tau_w_cmd] = step(F, z, t, P, D)
%ASILS.FSW.STEP  One tick of the reference flight software (the loop's FSW node).
%
%   z  sensor outputs this tick: .w gyro, .B magnetometer, .clean (coils were
%      off when B was read), .sun_ok/.sun, .st_ok/.q_st, .gps_ok/.r_gps/.v_gps
%
%   Order inside a tick (Standard Code sim.run tick order, condensed):
%     1 onboard orbit (GNSS fix or two-body propagation)
%     2 attitude determination (MEKF predict; ST / Sun / field updates)
%     3 mode manager (transitions)
%     4 guidance -> control -> actuator commands (MTQ duty cycle, wheels)
%
%   Modes: detumble | nadir_mtq | nadir_rw | target_rw | slew_rw
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
    m_body = F.m_hold; tau_w_cmd = zeros(size(F.tau_w));
    switch F.mode
        case 'detumble'
            % coil-off window: average the clean samples; the field-direction
            % derivative is taken between CONSECUTIVE cycles (1 s baseline) --
            % the Standard Code averaged 8+8 samples for the same reason: a
            % 0.2 s difference of noisy samples swamps a slow rate.
            if phase < G.mtq_meas + dt/2
                m_body = zeros(3,1);
                if first, F.bsum = zeros(3,1); F.bn = 0; end
                F.bsum = F.bsum + z.B/norm(z.B); F.bn = F.bn + 1;
                if abs(phase - G.mtq_meas) < dt/2
                    b = F.bsum/F.bn; b = b/norm(b);
                    if dev.gyro.fitted
                        % gyro-fed B-dot (Standard Code ctrl.bdot rate-feedback branch):
                        % the body sees the field direction turn at db/dt = -w x b
                        bd = -asils.util.cross3(z.w, b);
                        m_body = asils.fsw.bdot(b - bd, b, 1, norm(z.B), G.bdot_k, dev.mtq.m_max);
                    elseif ~isempty(F.b1)
                        m_body = asils.fsw.bdot(F.b1, b, G.mtq_period, norm(z.B), G.bdot_k, dev.mtq.m_max);
                    end
                    if any(m_body)
                        m_body = m_body - G.m_res_est;
                        m_body = m_body*min(1, dev.mtq.m_max/max(abs(m_body)));
                    end
                    F.b1 = b;
                end
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
        case {'nadir_rw', 'target_rw', 'slew_rw'}
            kind = strrep(F.mode, '_rw', '');
            % momentum dumping on the MTQ duty cycle (field sample with coils off)
            if first
                m_body = zeros(3,1);
            elseif abs(phase - G.mtq_meas) < dt/2
                hb = dev.rw.axes*(G.h_bias*ones(size(z.h_w)));
                F.m_dump = asils.fsw.dump(z.h_w, dev.rw.axes, hb, z.B, G.dump_k, dev.mtq.m_max);
                m_body = F.m_dump - G.m_res_est;                      % + residual-dipole compensation
                m_body = m_body*min(1, dev.mtq.m_max/max(abs(m_body)));
                F.B_dump = z.B;
            elseif phase < G.mtq_meas
                m_body = zeros(3,1);
            end
            % the wheels cancel the dump torque they know is coming (feedforward)
            if any(m_body), tau_ff = asils.util.cross3(m_body + G.m_res_est, F.B_dump); else, tau_ff = zeros(3,1); end
            if F.ad_ok && ~isempty(F.r)
                if t - F.last_ctrl >= G.rw.dt - 1e-9
                    [F.q_ref, F.w_ref] = asils.fsw.guidance(kind, F.r, F.v, t, F.gd);
                    [~, F.tau_req, F.I_q] = asils.fsw.pd_alloc(F.K.q, F.w_est, F.q_ref, F.w_ref, ...
                        z.h_w, F.I_q, G.rw.dt, G.rw, P.sc.I, dev.rw.axes, P.Awp);
                    F.last_ctrl = t;
                end
                F.tau_w = -P.Awp*(F.tau_req - tau_ff);
                tau_w_cmd = F.tau_w;
            end
    end
    F.m_hold = m_body;
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
