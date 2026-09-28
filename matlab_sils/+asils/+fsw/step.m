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
%   Controller states and the mission mode each serves: asils.fsw.modes
%     detumble          detumble (coils) | detumble_rcs (thrusters)
%     sun_acquisition   spinup + sun_spin (coils, Standard Code L1/L2) |
%                       sun_acq_rotor (momentum devices, Sun vector)
%     sun_referencing   sun_mtq (coils) | sun_fine (momentum devices)
%     nadir_pointing    nadir_mtq (coils) | nadir_fine (momentum devices)
%     (imaging)         target_fine, slew_fine
%
%   Which law does each job comes from the algorithm registry, resolved once
%   at configuration (asils.fsw.select -> P.fsw.alg, laws mapped in fsw.init):
%   G.bdot_law, G.mtq.law, G.rw.law. The mode code below only dispatches.
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
    F.gd.sun_eci = s_ref;                                % Sun-referencing guidance uses the onboard model
    phase = mod(t + 1e-9, G.mtq_period);
    first = phase < dt - 1e-9;                          % start of an MTQ cycle (coils off)
    if first && ~isempty(F.r) && (t - F.t_Bref) >= 0.999
        if isempty(F.gh), F.gh = asils.env.igrf_gh(asils.util.decyear(jd), P.igrf); end
        C = asils.fsw.gmst_rot(jd);
        F.Bref = asils.env.field(C*F.r, C, F.gh, G.igrf_nmax); F.t_Bref = t;
    end
    usesAD = ~any(strcmp(F.mode, {'detumble', 'detumble_rcs', 'spinup', 'sun_spin'}));   % rate-only / spin modes need no attitude
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
                if z.es_ok && first && ~isempty(F.r)             % Earth sensor: nadir vector
                    F.K = asils.fsw.mekf_vector(F.K, z.nadir, -F.r/norm(F.r), max(dev.es.noise, 1e-3)*2);
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
    while F.sched_i <= numel(F.sched) && t >= F.sched{F.sched_i}.t_s   % commanded changes (ground / schedule)
        F = enter_(F, F.sched{F.sched_i}.mode, t); F.sched_i = F.sched_i + 1;
    end
    if any(strcmp(F.mode, {'detumble', 'detumble_rcs'})) && ~isempty(G.auto_next)
        if norm(z.w) < G.detumble_exit, F.hold = F.hold + dt; else, F.hold = 0; end
        if F.hold >= G.detumble_hold_s
            F = enter_(F, G.auto_next, t);
        end
    end
    if any(strcmp(F.mode, {'spinup', 'sun_spin'}))
        [F, nxt] = spin_guards_(F, z, t, dt, G.ss);
        if ~isempty(nxt), F = enter_(F, nxt, t); end
    end
    if any(strcmp(F.mode, {'spinup', 'sun_spin', 'sun_acq_rotor'}))
        % Sun vector for the acquisition laws: measured when valid, else (E2)
        % propagated on the gyro from the last sunlit sample (ctrl.propagateSun)
        if z.sun_ok, F.s_prop = z.sun;
        elseif ~isempty(F.s_prop)
            F.s_prop = asils.quat.dcm(asils.quat.fromrotvec(F.w_est*dt))*F.s_prop;
            F.s_prop = F.s_prop/norm(F.s_prop);
        end
    end
    if strcmp(F.mode, 'sun_acq_rotor') && ~isempty(G.auto_next)     % acquired -> next mode
        ok = z.sun_ok && acosd(max(-1, min(1, z.sun'*dev.sun_axis))) < G.sa.done_deg && F.ad_ok;
        if ok, F.acq_hold = F.acq_hold + dt; else, F.acq_hold = 0; end
        if F.acq_hold >= G.sa.done_hold_s, F = enter_(F, G.auto_next, t); end
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
                if first, F.bsum = zeros(3,1); F.bsum_raw = zeros(3,1); F.bn = 0; end
                F.bsum = F.bsum + z.B/norm(z.B); F.bsum_raw = F.bsum_raw + z.B; F.bn = F.bn + 1;
                if abs(phase - G.mtq_meas) < dt/2
                    b = F.bsum/F.bn; b = b/norm(b);
                    law = G.bdot_law; if strcmp(law, 'gyro') && ~dev.gyro.fitted, law = 'mag'; end
                    switch law
                        case 'gyro'      % db/dt = -w x b (Standard Code ctrl.bdot rate-feedback branch)
                            bd = -asils.util.cross3(z.w, b);
                            m_body = asils.fsw.bdot(b - bd, b, 1, norm(z.B), G.bdot_k, dev.mtq.m_max);
                        case 'mag'       % difference of consecutive coil-off windows
                            if ~isempty(F.b1), m_body = asils.fsw.bdot(F.b1, b, G.mtq_period, norm(z.B), G.bdot_k, dev.mtq.m_max); end
                        case 'bangbang'  % Standard Code ctrl.bdot TC == 0: full dipole against d(b)/dt,
                                         % with a boundary layer (4 x the B-dot gain inside it): pure sign
                                         % switching limit-cycles around the detumble exit rate
                            if ~isempty(F.b1)
                                bd = (b - F.b1)/G.mtq_period;
                                bl = dev.mtq.m_max*norm(z.B)/(4*G.bdot_k);
                                m_body = -dev.mtq.m_max*min(1, max(-1, bd/bl));
                            end
                        case 'l1'        % Standard Code ctrl.genBdot with omega_d = 0, on the raw field [T]
                            Bav = F.bsum_raw/F.bn;
                            if ~isempty(F.B1raw)
                                m_body = asils.fsw.gen_bdot(Bav, (Bav - F.B1raw)/G.mtq_period, zeros(3,1), G.ss.k_l1);
                                m_body = m_body*min(1, dev.mtq.m_max/max(max(abs(m_body)), 1e-30));
                            end
                            F.B1raw = Bav;
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
        case 'detumble_rcs'
            % thruster rate damping: tau = -I w / T_damp (couples, MIB-aware),
            % off inside a small deadband; coils off, rotors held at their bias
            % thruster PWM on a 1 s cycle (a 0.1 s tick would leave every small
            % request below the minimum impulse bit): the on-times are set at
            % the start of the cycle and played out over its ticks
            Tc = G.rcsd.period_s;
            if F.nc > 0 && (isempty(F.rcs_left) || mod(t + 1e-9, Tc) < dt - 1e-9)
                F.rcs_left = zeros(1, F.nc);
                if norm(F.w_est) > G.rcsd.deadband_deg_s*pi/180
                    F.tau_req = -P.sc.I*F.w_est/G.rcsd.T_damp_s;
                    [dc, ~] = asils.fsw.rcs_duty(F.tau_req, dev.rcs, Tc);
                    F.rcs_left = dc*Tc;
                end
            end
            if F.nc > 0
                duty = min(1, F.rcs_left/dt);
                F.rcs_left = max(0, F.rcs_left - dt);
            end
            m_body = zeros(3,1);
            if nr > 0
                cmd_r = -0.2*(z.h - F.h_t_rot);
                cmd_r(F.M.gi(:) > 0 & strcmp(dev.mex.kind(:), 'cmg')) = 0;
            end
        case {'nadir_mtq', 'sun_mtq'}
            if first
                m_body = zeros(3,1);
            elseif abs(phase - G.mtq_meas) < dt/2 && F.ad_ok && ~isempty(F.r)
                [F.q_ref, F.w_ref] = asils.fsw.guidance(strrep(F.mode, '_mtq', ''), F.r, F.v, t, F.gd);
                F = mtq_law_(F, G, P);
                m_body = asils.fsw.torque2dipole(F.tau_req, z.B, dev.mtq.m_max) - G.m_res_est;
                m_body = m_body*min(1, dev.mtq.m_max/max(abs(m_body)));   % residual-dipole compensation
            elseif phase < G.mtq_meas
                m_body = zeros(3,1);
            end
        case {'nadir_fine', 'target_fine', 'slew_fine', 'sun_fine', 'sun_acq_rotor'}
            kind = strrep(F.mode, '_fine', '');
            acq = strcmp(F.mode, 'sun_acq_rotor');      % Sun-vector law, no attitude solution needed
            ctl_ok = F.ad_ok || acq;
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
            if acq && t - F.last_ctrl >= G.rw.dt - 1e-9
                F.tau_req = sun_acq_law_(F, z, dev.sun_axis, G.sa, P.sc.I, Hdev);
                F.last_ctrl = t;
            elseif F.ad_ok && ~isempty(F.r) && t - F.last_ctrl >= G.rw.dt - 1e-9
                [qr, wr, wdr] = asils.fsw.guidance(kind, F.r, F.v, t, F.gd);
                F.q_ref = qr; F.w_ref = wr;
                [F.tau_req, F.capturing] = capture_law_(F, G, P.sc.I, Hdev);
                if F.capturing
                    F.I_q = zeros(3,1);            % no integral windup during the manoeuvre
                else
                    [F.tau_req, F.I_q] = asils.fsw.control_law(F.K.q, F.w_est, F.q_ref, F.w_ref, F.I_q, G.rw.dt, G.rw, P.sc.I, Hdev, wdr);
                end
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
                if F.idmas && ctl_ok        % IDMAS split: coils take the torque across the field
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
            if F.nc > 0 && ctl_ok
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
            if ctl_ok && (~isempty(F.r) || acq) && nr > 0
                [cmd_r, cmd_g] = asils.fsw.allocate(F.tau_req - tau_coil - tau_rcs, z, F, P);
            end
            F.h_prev = z.h; F.cmd_r_prev = cmd_r;
        case {'spinup', 'sun_spin'}
            % Standard Code duty: coils off and the field averaged over the
            % measure window, the law run once per cycle, the dipole held.
            if phase < G.mtq_meas + dt/2
                m_body = zeros(3,1);
                if first, F.bsum_raw = zeros(3,1); F.bn = 0; end
                F.bsum_raw = F.bsum_raw + z.B; F.bn = F.bn + 1;
                if abs(phase - G.mtq_meas) < dt/2
                    Bav = F.bsum_raw/F.bn;
                    if strcmp(F.mode, 'spinup')       % L1: ctrl.spinupTick
                        if isempty(F.B1raw), bd = zeros(3,1); else, bd = (Bav - F.B1raw)/G.mtq_period; end
                        wd = F.sigma*G.ss.spin_dps*pi/180*[0;0;1];
                        m0 = asils.fsw.gen_bdot(Bav, bd, wd, G.ss.k_l1);
                    else                              % L2: ctrl.sunSpin (He et al. 2023)
                        ecl = ~z.sun_ok && strcmp(G.ss.eclipse, 'E1');
                        [m0, F.V_ss] = asils.fsw.sun_spin(Bav, F.w_est, F.s_prop, ecl || isempty(F.s_prop), P.sc.I, G.ss);
                    end
                    F.B1raw = Bav;
                    if any(m0)
                        m_body = m0 - G.m_res_est;
                        m_body = m_body*min(1, dev.mtq.m_max/max(abs(m_body)));   % act.saturateDipole
                    end
                end
            end
            if nr > 0, cmd_r = -0.2*(z.h - F.h_t_rot); end
    end
    F.m_hold = m_body;
    out.m_body = m_body; out.cmd_r = cmd_r; out.cmd_g = cmd_g; out.duty = duty;
end

function [tau, on] = capture_law_(F, G, I, Hdev)
%CAPTURE_LAW_  Large-error capture for the fine modes: beyond G.capture_deg
%   the linear law would ask for far more torque than small momentum devices
%   have, saturate them and wind its integrator up. Instead: an eigenaxis rate
%   command sized to THIS product's authority -- the rate that stops within
%   the remaining angle at half the torque capacity, never above half the
%   momentum capacity or G.capture_rate -- tracked by a rate loop. Slews
%   (guidance 'slew') keep their own profile.
    tau = zeros(3,1); on = false;
    if strcmp(F.mode, 'slew_fine') || G.capture_deg <= 0, return, end
    qe = asils.quat.mult(asils.quat.conj(F.q_ref), F.K.q); if qe(4) < 0, qe = -qe; end
    th = 2*acos(min(1, qe(4)));
    if th < G.capture_deg*pi/180, return, end
    on = true;
    e = qe(1:3)/max(norm(qe(1:3)), 1e-12);
    Jm = max(diag(I));
    alpha = 0.5*min(F.cap)/Jm;
    wmax = min(G.capture_rate_deg_s*pi/180, 0.5*min(F.hcap)/Jm);
    wref = asils.quat.dcm(qe)*F.w_ref;
    wc = wref - e*min(wmax, sqrt(2*alpha*th));
    w = F.w_est; H = I*w + Hdev;
    kr = min(0.5, 4*alpha/max(wmax, 1e-6));
    tau = I*(kr*(wc - w)) + [w(2)*H(3)-w(3)*H(2); w(3)*H(1)-w(1)*H(3); w(1)*H(2)-w(2)*H(1)];
end

function tau = sun_acq_law_(F, z, a, g, I, Hdev)
%SUN_ACQ_LAW_  Sun-vector acquisition with momentum devices (no attitude
%   solution): rate command w_c = k (a x s) turns the power face a onto the
%   measured (or gyro-propagated) Sun s -- in body axes ds/dt = -w x s, so
%   d(s.a)/dt = k (1 - (s.a)^2) >= 0 -- saturated at g.w_max; a rate loop with gyroscopic
%   compensation gives the torque. Antiparallel start: turn about any axis
%   normal to a. No Sun ever seen: hold the rate at zero.
    wmax = g.w_max_deg_s*pi/180; wc = zeros(3,1);
    if ~isempty(F.s_prop)
        s = F.s_prop/norm(F.s_prop);
        c = asils.util.cross3(a, s);
        if s'*a < -0.95
            c = asils.util.cross3(a, [1;0;0]); if norm(c) < 0.1, c = asils.util.cross3(a, [0;1;0]); end
            c = c/norm(c);
        end
        wc = (wmax/0.5)*c;
        if norm(wc) > wmax, wc = wc*wmax/norm(wc); end
    end
    w = F.w_est; H = I*w + Hdev;
    tau = I*(g.kd*(wc - w)) + [w(2)*H(3)-w(3)*H(2); w(3)*H(1)-w(1)*H(3); w(1)*H(2)-w(2)*H(1)];
end

function F = mtq_law_(F, G, P)
%MTQ_LAW_  The magnetic pointing law the registry selected (slot mtq_pointing).
%   Every law returns a body torque request; torque2dipole keeps the part
%   across the field. Same bandwidth for all, so a trade compares laws.
    g = G.mtq;
    switch g.law
        case 'mtq_pd'            % quaternion PD (Lovera & Astolfi 2004)
            F.tau_req = asils.fsw.mtq_pd(F.K.q, F.w_est, F.q_ref, F.w_ref, g);
        case {'mtq_lqr', 'mtq_smc'}
            g.law = strrep(g.law, 'mtq_', '');
            [F.tau_req, F.I_q] = asils.fsw.control_law(F.K.q, F.w_est, F.q_ref, F.w_ref, F.I_q, G.mtq_period, g, P.sc.I, zeros(3,1));
        case 'mtq_rate_damp'     % damp the rate relative to LVLH only; gravity gradient holds pitch/roll
            qe = asils.quat.mult(asils.quat.conj(F.q_ref), F.K.q);
            F.tau_req = -g.Kd.*(F.w_est - asils.quat.dcm(qe)*F.w_ref);
        otherwise
            error('asils:fsw:law', 'unknown magnetic pointing law %s', g.law);
    end
end

function [F, nxt] = spin_guards_(F, z, t, dt, s)
%SPIN_GUARDS_  Standard Code modes.transitions rows for SpinUp / SunSpin and the
%   spin-sign flip G_sigma of ctrl.spinupTick (theory doc sec. 4.4, 7.2).
    nxt = ''; d = pi/180; w = F.w_est;
    wz = w(3); wp = norm(w(1:2));
    if norm(w) > s.omega_max_dps*d, nxt = 'detumble'; return, end     % G_fault, dwell 0
    if strcmp(F.mode, 'spinup')
        conv = abs(wz - F.sigma*s.spin_dps*d) < s.z_in_dps*d && wp < s.perp_in_dps*d;
        % G_sigma: -Z_B must end up on the Sun; a converged spin that keeps the
        % Sun on +Z flips the target spin sign
        if conv && z.sun_ok, F.sz_sum = F.sz_sum + z.sun(3); F.sz_n = F.sz_n + 1; end
        if t - F.sz_t0 >= s.t_check_s && F.sz_n > 0 && F.sz_sum/F.sz_n > s.sun_min
            F.sigma = -F.sigma; F.sz_sum = 0; F.sz_n = 0; F.sz_t0 = t;
            F.log(end+1).t = t; F.log(end).mode = sprintf('spin sign -> %+d', F.sigma);
        end
        ok = conv && z.sun_ok && z.sun(3) < 0;                           % G_S->SS
        if ok, F.hold = F.hold + dt; else, F.hold = 0; end
        if F.hold >= s.dwell_in_s, nxt = 'sun_spin'; end
    else
        bad = abs(wz) < s.omega_exit_dps*d || wp > s.perp_out_dps*d;    % G_SS->S
        if bad, F.hold = F.hold + dt; else, F.hold = 0; end
        if F.hold >= s.dwell_out_s, nxt = 'spinup'; end
    end
end

function F = enter_(F, mode, t)
    if strcmp(mode, 'spinup'), F.sz_sum = 0; F.sz_n = 0; F.sz_t0 = t; end
    F.mode = mode; F.t_mode = t; F.hold = 0; F.I_q = zeros(3,1);
    F.log(end+1).t = t; F.log(end).mode = mode;
end

function q = latency_(q_st, w, lat)
%LATENCY_  Bring a star-tracker attitude from t - latency to t with the gyro
%   (Standard Code sdp.starTrackerLatency, constant-rate form).
    q = asils.quat.norm(asils.quat.mult(q_st, asils.quat.fromrotvec(w*lat)));
end
