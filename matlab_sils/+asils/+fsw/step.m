function [F, bus] = step(F, bus)
%ASILS.FSW.STEP  One tick of the twin's flight software at bus.now_ns: the runtime of fsw-rs/src/fsw.rs
%   (Fsw::step) and fsw/src/adcs_fsw.c (adcs_fsw_step), in MATLAB.
%
%   [F, bus] = asils.fsw.step(F, bus)
%
%   The sensors are read from the bus as their bytes (asils.fsw.drv_read: registers, UART frames, CAN
%   telemetry), the commands written back as bytes (asils.fsw.drv_write: PWM words, CAN frames). The tick's
%   order, its state and the branch of each controller state are the runtime's (code, as in C and Rust);
%   every law, estimator, guidance, mode rule, driver decoding and command word is the design's, generated
%   into +asils/+alg by tools/flight_build.py (fsw/pseudocode 01-09) and called here, never written here.
%   Owner: Agastya. Copyright (c) 2026 Agastya. All rights reserved.
    p = F.p;
    dt = p.dt; nr = p.nr;
    D2R = pi/180;
    F.t = (bus.now_ns - F.start_ns)*1e-9;
    F.jd = p.jd0 + F.t/86400.0;
    [z, F, bus] = asils.fsw.drv_read(F, bus);
    % sensor health (modes::fdir_sensors)
    [z.mag_ok, z.b, z.w, F.b_good, F.mag_age, F.mag_seen, F.gyro_age] = asils.alg.modes.fdir_sensors(z.mag_ok, z.b, ...
        ~isempty(F.bref), bref_(F), F.b_good, F.mag_age, F.mag_seen, p.has_gyro ~= 0, z.gyro_ok, z.w, F.gyro_age, dt);
    F.z = z;

    % 1 onboard orbit (02): the GNSS fix, else two-body + J2 by velocity Verlet
    % a fix inside the Earth is not a fix: dropped, the orbit propagated as without one
    if z.gps_ok && norm3_(z.r) > 0.9*6378137.0
        l = p.gps_latency;          % the fix is the state l seconds ago
        if p.gnss_ecef ~= 0
            % receiver fix in ECEF: r = C' r_e, v = C' (v_e + w_E x r_e), C at the fix's epoch
            c = asils.alg.frames.eci2ecef(F.jd - l/86400.0);
            ve = z.v + cross3_([0; 0; 7.2921158553e-5], z.r);
            F.r = asils.alg.math.mat3t_vec(c, z.r); F.v = asils.alg.math.mat3t_vec(c, ve);
        else
            F.r = z.r; F.v = z.v;
        end
        if l > 0
            % carried forward to now: one Verlet step of l
            a0 = asils.alg.steplaws.orbit_acc(F.r, p.mu);
            for i = 1:3, F.r(i) = F.r(i) + F.v(i)*l + 0.5*a0(i)*l*l; end
            a1 = asils.alg.steplaws.orbit_acc(F.r, p.mu);
            for i = 1:3, F.v(i) = F.v(i) + 0.5*(a0(i) + a1(i))*l; end
        end
        F.have_r = true;
    elseif F.have_r
        a0 = asils.alg.steplaws.orbit_acc(F.r, p.mu);
        for i = 1:3, F.r(i) = F.r(i) + F.v(i)*dt + 0.5*a0(i)*dt*dt; end
        a1 = asils.alg.steplaws.orbit_acc(F.r, p.mu);
        for i = 1:3, F.v(i) = F.v(i) + 0.5*(a0(i) + a1(i))*dt; end
    end

    % 2 estimation
    F.gd.sun_eci = asils.alg.frames.sun_model(F.jd);
    if p.gd_yaw_flip ~= 0 && F.have_r
        g = F.gd;
        F.gd.flip = asils.alg.guidance.yaw_flip(F.r, F.v, g.q_off, g.sun_axis, g.roll_axis, g.sun_eci, g.flip, p.gd_flip_hyst);
    end
    phase = fmod_(F.t + 1e-9, p.mtq_period);
    first = phase < dt - 1e-9;
    if first, F.mag_done = false; end
    if z.es_ok, F.es_n = z.nadir; F.es_t = F.t; end
    if first && F.have_r && (F.t - F.t_bref) >= 0.999
        if isempty(F.gh) || fabs_(F.jd - F.gh_jd) > 1.0
            F.gh = asils.alg.frames.igrf_gh(asils.alg.frames.decyear(F.jd)); F.gh_jd = F.jd;
        end
        F.bref = asils.alg.frames.field_eci(F.r, F.jd, F.gh, p.igrf_nmax);
        F.t_bref = F.t;
    end
    m = F.mode;
    if ~(m == 0 || m == 7 || m == 5 || m == 6)            % not DETUMBLE, DETUMBLE_RCS, SPINUP, SUN_SPIN
        if ~F.ad_ok
            if p.has_st ~= 0 && z.st_ok
                h = 0;
                while h < p.n_heads && ~z.st_valid(h + 1), h = h + 1; end
                q0 = asils.alg.estimation.latency(z.q_st(h + 1, :).', z.w, p.st_latency);
                F.K = mekf_new_(q0, 1e-3, 2e-4, p.gyro_arw, p.gyro_rrw);
                F.ad_ok = true;
                F.t_st = F.t;
            elseif z.sun_ok && z.mag_ok && F.clean && ~isempty(F.bref)
                % Sun and field not parallel
                [q0, ok] = asils.alg.estimation.triad(z.sun, z.b, F.gd.sun_eci, F.bref);
                if ok
                    F.K = mekf_new_(q0, 0.05, 2e-4, p.gyro_arw, p.gyro_rrw);
                    F.ad_ok = true;
                end
            end
        else
            [F.K.q, F.K.P] = asils.alg.estimation.mekf_predict(F.K.q, F.K.b, F.K.P, F.K.arw, F.K.rrw, z.w, dt);
            if p.has_st ~= 0 && z.st_ok
                for h = 1:p.n_heads
                    qs = z.q_st(h, :);
                    qn = sqrt(qs(1)*qs(1) + qs(2)*qs(2) + qs(3)*qs(3) + qs(4)*qs(4));
                    if z.st_valid(h) && fabs_(qn - 1.0) < 1e-3          % a unit quaternion, or no reading
                        wb = z.w - F.K.b;
                        ql = asils.alg.estimation.latency(qs.', wb, p.st_latency);
                        [F.K.q, F.K.b, F.K.P] = asils.alg.estimation.mekf_quat(F.K.q, F.K.b, F.K.P, ql, ...
                            p.st_noise_cross*p.mekf_meas_scale, p.st_noise_roll*p.mekf_meas_scale, p.st_bs(h, :).', 0.0);
                    end
                end
                F.t_st = F.t;
            elseif p.has_st ~= 0 && F.t - F.t_st < p.st_coast_s
                % short star-tracker outage: coast on the gyro
            else
                % vector updates once per coil cycle; the field on the first clean tick of the cycle
                tried = 0; took = 0;
                if z.sun_ok && first
                    tried = tried + 1;
                    [F.K, ok] = mekf_vector_(F.K, z.sun, F.gd.sun_eci, p.mekf_sig_sun, p.mekf_gate);
                    if ok, took = took + 1; end
                end
                if F.clean && ~F.mag_done && z.mag_ok && ~isempty(F.bref)
                    bn = norm3_(z.b);
                    if bn > 1e-9, e = p.mekf_mag_err_T/bn; else, e = p.mekf_mag_err_T/1e-9; end
                    F.mag_done = true; tried = tried + 1;
                    [F.K, ok] = mekf_vector_(F.K, z.b, F.bref, sqrt(p.mekf_sig_mag*p.mekf_sig_mag + e*e), p.mekf_gate);
                    if ok, took = took + 1; end
                end
                if first && F.have_r && F.t - F.es_t < p.mtq_period
                    % latest Earth-sensor sample of the cycle
                    if p.es_noise > 1e-3, sg = p.es_noise; else, sg = 1e-3; end
                    tried = tried + 1;
                    [F.K, ok] = mekf_vector_(F.K, F.es_n, F.r*(-1.0), 2.0*sg, p.mekf_gate);
                    if ok, took = took + 1; end
                    F.es_t = -1e9;
                end
                % every update gated for mekf_rej_max in a row: the estimate has diverged, re-initialise
                if tried > 0, if took > 0, F.n_rej = 0; else, F.n_rej = F.n_rej + tried; end, end
                if p.mekf_rej_max > 0 && F.n_rej >= p.mekf_rej_max, F.ad_ok = false; F.n_rej = 0; end
            end
        end
    end
    a = dt/(p.rate_lpf_s + dt);
    if F.ad_ok, wr = z.w - F.K.b; else, wr = z.w; end
    if p.rate_lpf_s <= 0
        F.w_est = wr;
    else
        for i = 1:3, F.w_est(i) = F.w_est(i) + a*(wr(i) - F.w_est(i)); end
    end

    % 3 mode manager: commanded changes, safe mode, the transitions
    F = set_modes_(F, asils.alg.modes.modes_schedule(modes_state_(F), F.mp));
    F = set_modes_(F, asils.alg.modes.fdir_safe(modes_state_(F)));
    F = set_modes_(F, asils.alg.modes.modes_step(modes_state_(F), F.mp, z.w, z.sun_ok, z.sun, dt));

    % 4 guidance / control / commands
    m_body = F.m_hold;
    cmd_r = zeros(8, 1); cmd_g = zeros(4, 1); duty = zeros(6, 1);
    switch F.mode
        case 0                                         % DETUMBLE
            if phase < p.mtq_meas + dt/2.0
                m_body = zeros(3, 1);
                if first, F.bsum = zeros(3, 1); F.bsum_raw = zeros(3, 1); F.bn = 0; end
                if z.mag_ok
                    F.bsum = F.bsum + unit_(z.b);
                    F.bsum_raw = F.bsum_raw + z.b;
                    F.bn = F.bn + 1;
                end
                % no field this cycle: no dipole, no rate across the gap
                if fabs_(phase - p.mtq_meas) < dt/2.0 && F.bn == 0
                    F.b1 = []; F.b1raw = [];
                elseif fabs_(phase - p.mtq_meas) < dt/2.0
                    b = unit_(F.bsum*(1.0/F.bn));
                    law = p.bdot_law;
                    if law == 0 && (p.has_gyro == 0 || F.gyro_age > 0.0), law = 1; end
                    if law == 0
                        bd = cross3_(z.w, b)*(-1.0);
                        m_body = asils.alg.control.bdot(b - bd, b, 1.0, norm3_(z.b), p.bdot_k, p.m_max);
                    elseif law == 1
                        if ~isempty(F.b1), m_body = asils.alg.control.bdot(F.b1, b, p.mtq_period, norm3_(z.b), p.bdot_k, p.m_max); end
                    elseif law == 2
                        % bang-bang with a boundary layer: full dipole outside it, 4 x the B-dot gain inside it
                        bl = p.m_max*norm3_(z.b)/(4.0*p.bdot_k);
                        if ~isempty(F.b1)
                            for i = 1:3
                                r = ((b(i) - F.b1(i))/p.mtq_period)/bl;
                                if r > 1.0, r = 1.0; elseif r < -1.0, r = -1.0; end
                                m_body(i) = -p.m_max*r;
                            end
                        end
                    else
                        bav = F.bsum_raw*(1.0/F.bn);
                        if ~isempty(F.b1raw)
                            bd = zeros(3, 1);
                            for i = 1:3, bd(i) = (bav(i) - F.b1raw(i))/p.mtq_period; end
                            m_body = asils.alg.control.gen_bdot(bav, bd, zeros(3, 1), p.ss_k_l1);
                            mx = asils.alg.math.maxabs3(m_body);
                            if mx < 1e-30, mx = 1e-30; end
                            if p.m_max/mx < 1.0, m_body = m_body*(p.m_max/mx); else, m_body = m_body*1.0; end
                        end
                        F.b1raw = bav;
                    end
                    if ~is_zero3_(m_body), m_body = asils.alg.control.sat_dipole(m_body - p.m_res_est, p.m_max); end
                    F.b1 = b;
                end
            end
            if nr > 0, cmd_r = idle_rotors_(F, cmd_r, true); end
        case 7                                         % DETUMBLE_RCS
            tc = p.rcsd_period_s;
            if p.nc > 0 && (isempty(F.rcs_left) || fmod_(F.t + 1e-9, tc) < dt - 1e-9)
                left = zeros(6, 1);
                if norm3_(F.w_est) > p.rcsd_deadband_deg_s*D2R
                    F.tau_req = mat3_vec_(p.J, F.w_est*(-1.0/p.rcsd_T_damp_s));
                    [dc, ~] = asils.alg.allocation.rcs_duty(F.tau_req, p.nc, p.rcs_tau, p.rcs_mib, p.rcs_res, tc);
                    for i = 1:6, left(i) = dc(i)*tc; end
                end
                F.rcs_left = left;
            end
            if p.nc > 0
                for i = 1:6
                    duty(i) = F.rcs_left(i)/dt;
                    if duty(i) > 1.0, duty(i) = 1.0; end
                    F.rcs_left(i) = F.rcs_left(i) - dt;
                    if F.rcs_left(i) < 0.0, F.rcs_left(i) = 0.0; end
                end
            end
            m_body = zeros(3, 1);
            if nr > 0, cmd_r = idle_rotors_(F, cmd_r, true); end
        case {1, 9}                                    % NADIR_MTQ, SUN_MTQ
            if first
                m_body = zeros(3, 1);
            elseif fabs_(phase - p.mtq_meas) < dt/2.0 && F.ad_ok && F.have_r
                g = F.gd;
                [F.q_ref, F.w_ref] = asils.alg.guidance.guidance(asils.alg.guidance.guid_kind(F.mode), F.r, F.v, F.t, g.q_off, ...
                    g.roll_deg, g.t0, g.t_slew, g.axis, g.q_inertial, g.sun_axis, g.roll_axis, g.sun_eci, g.flip);
                % hand-over (05_control.md): the rate error is damped first with the detumble gain (Avanzini & Giulietti 2012)
                qe = asils.alg.math.qmult(asils.alg.math.qconj(F.q_ref), F.K.q);
                wr = mat3_vec_(asils.alg.math.dcm(qe), F.w_ref);
                we = F.w_est - wr;
                wen = norm3_(we);
                if ~F.ho && wen > p.ho_in_dps*D2R, F.ho = true; F.ho_t = 0.0; end
                if F.ho
                    if wen < p.ho_out_dps*D2R, F.ho_t = F.ho_t + p.mtq_period; else, F.ho_t = 0.0; end
                    if F.ho_t >= p.ho_hold_s, F.ho = false; end
                end
                if F.ho
                    F.tau_req = zeros(3, 1);
                    bn = norm3_(z.b);
                    if bn > 1e-9, k = p.bdot_k/bn; else, k = 0.0; end
                    md = cross3_(we, unit_(z.b))*k;
                else
                    F = set_ctl_(F, asils.alg.steplaws.ctl_mtq(ctl_state_(F), F.cp));
                    if bitand(p.mtq_gg_ff, 1 + (F.mode ~= 9)) ~= 0
                        % the gravity-gradient feed-forward: 3 mu/|r|^5 (r_b x J r_b) cancelled
                        rb = mat3_vec_(asils.alg.math.dcm(F.K.q), F.r);
                        rn = norm3_(rb);
                        f = 3.0*p.mu/(rn*rn*rn*rn*rn);
                        c = cross3_(rb, mat3_vec_(p.J, rb));
                        for i = 1:3, F.tau_req(i) = F.tau_req(i) - f*c(i); end
                    end
                    md = asils.alg.control.torque2dipole(F.tau_req, z.b, p.m_max);
                end
                m_body = asils.alg.control.sat_dipole(md - p.m_res_est, p.m_max);
            elseif phase < p.mtq_meas
                m_body = zeros(3, 1);
            end
        case {2, 3, 4, 10, 8}                          % NADIR_FINE, TARGET_FINE, SLEW_FINE, SUN_FINE, SUN_ACQ_ROTOR
            acq = F.mode == 8;
            ctl_ok = F.ad_ok || acq;
            A = asils.alg.allocation.rotor_axes(p.nr, p.rot_a0, p.rot_gi, p.gim_axis, z.delta);
            hdev = zeros(3, 1);
            for i = 1:nr
                hdev(1) = hdev(1) + A(1, i)*z.h(i); hdev(2) = hdev(2) + A(2, i)*z.h(i); hdev(3) = hdev(3) + A(3, i)*z.h(i);
            end
            F = fdir_rotors_(F, z, dt);
            % control law at the control rate
            if acq && F.t - F.last_ctrl >= p.rw_dt - 1e-9
                F = set_ctl_(F, asils.alg.steplaws.ctl_sun_acq(ctl_state_(F), F.cp, hdev));
                F.last_ctrl = F.t;
            elseif ~acq && F.ad_ok && F.have_r && F.t - F.last_ctrl >= p.rw_dt - 1e-9
                g = F.gd;
                [F.q_ref, F.w_ref, wd] = asils.alg.guidance.guidance(asils.alg.guidance.guid_kind(F.mode), F.r, F.v, F.t, g.q_off, ...
                    g.roll_deg, g.t0, g.t_slew, g.axis, g.q_inertial, g.sun_axis, g.roll_axis, g.sun_eci, g.flip);
                [s, on] = asils.alg.steplaws.ctl_capture(ctl_state_(F), F.cp, hdev);
                F = set_ctl_(F, s);
                F.capturing = on;
                if F.capturing
                    F.i_q = zeros(3, 1);
                else
                    gr = F.g_rw;
                    [F.tau_req, F.i_q] = asils.alg.control.control_law(F.K.q, F.w_est, F.q_ref, F.w_ref, F.i_q, p.rw_dt, gr.law, gr.kp, ...
                        gr.kd, gr.ki, gr.klqr, gr.lambda, gr.phi, gr.gs, gr.err_max, gr.int_max, p.J, hdev, wd);
                end
                F.last_ctrl = F.t;
            end
            % momentum management
            dh = hdev - F.h_t;
            if F.has_rcs_dump
                if norm3_(dh) > F.dump_hi, F.rcs_dumping = true;
                elseif norm3_(dh) < F.dump_lo, F.rcs_dumping = false; end
            end
            if first
                m_body = zeros(3, 1);
            elseif fabs_(phase - p.mtq_meas) < dt/2.0
                mm = zeros(3, 1);
                if ~F.has_rcs_dump, mm = asils.alg.allocation.dump(hdev, F.h_t, z.b, p.dump_k, p.m_max); end
                if p.alloc == 1 && ctl_ok, mm = mm + asils.alg.control.torque2dipole(F.tau_req, z.b, p.m_max); end
                if F.ad_ok
                    anyf = false; fx = zeros(8, 1); nf = 0;
                    for i = 1:nr
                        if F.rot_failed(i), anyf = true; elseif p.rot_gi(i) == 0, nf = nf + 1; fx(nf) = i; end
                    end
                    if anyf
                        un = F.tau_req;
                        if nf > 0
                            af = zeros(3, 8);
                            for i = 1:nf, for k = 1:3, af(k, i) = A(k, fx(i)); end, end
                            pin = asils.alg.math.pinv_rows(af, nf);
                            y = zeros(8, 1);
                            for i = 1:nf, y(i) = pin(i, 1)*F.tau_req(1) + pin(i, 2)*F.tau_req(2) + pin(i, 3)*F.tau_req(3); end
                            for k = 1:3, for i = 1:nf, un(k) = un(k) - af(k, i)*y(i); end, end
                        end
                        mm = mm + asils.alg.control.torque2dipole(un, z.b, p.m_max);
                    end
                end
                m_body = asils.alg.control.sat_dipole(mm - p.m_res_est, p.m_max);
                F.b_dump = z.b;
            elseif phase < p.mtq_meas
                m_body = zeros(3, 1);
            end
            % thrusters: assist and dumping
            tau_rcs = zeros(3, 1);
            if p.nc > 0 && ctl_ok
                req = zeros(3, 1);
                if p.rcs_assist ~= 0
                    for i = 1:3
                        ex = fabs_(F.tau_req(i)) - p.rcs_assist_frac*F.cap(i);
                        if ex > 0.0, req(i) = sign_(F.tau_req(i))*ex; else, req(i) = sign_(F.tau_req(i))*0.0; end
                        if fabs_(hdev(i)) > p.rcs_assist_frac*F.hcap(i) && sign_(-F.tau_req(i)) == sign_(hdev(i)), req(i) = F.tau_req(i); end
                    end
                end
                if F.rcs_dumping, for i = 1:3, req(i) = req(i) - p.rcs_dump_k*dh(i); end, end
                [duty, tau_rcs] = asils.alg.allocation.rcs_duty(req, p.nc, p.rcs_tau, p.rcs_mib, p.rcs_res, dt);
            end
            % momentum devices deliver the rest (coil and thruster torques fed forward)
            tau_coil = zeros(3, 1);
            if ~is_zero3_(m_body), tau_coil = cross3_(m_body + p.m_res_est, F.b_dump); end
            if ctl_ok && (F.have_r || acq) && nr > 0
                tr = zeros(3, 1);
                for i = 1:3, tr(i) = F.tau_req(i) - tau_coil(i) - tau_rcs(i); end
                [cmd_r, cmd_g] = asils.alg.steplaws.alloc_rotors(tr, A, cmd_r, cmd_g, p.nr, p.ng, p.rot_gi, p.rot_kind, ...
                    F.rot_failed, F.z.h, p.gim_axis, p.gim_rate_max, p.cmg_lam0, p.cmg_mu, p.cmg_k_null, p.rot_h0);
            end
            hp = zeros(8, 1);
            hp(1:nr) = z.h(1:nr);
            F.cmd_r_prev(1:nr) = cmd_r(1:nr);
            if ~isempty(F.h_prev), hp(nr+1:end) = F.h_prev(nr+1:end); end
            F.h_prev = hp;
        case {5, 6}                                    % SPINUP, SUN_SPIN
            if phase < p.mtq_meas + dt/2.0
                m_body = zeros(3, 1);
                if first, F.bsum_raw = zeros(3, 1); F.bn = 0; end
                if z.mag_ok, F.bsum_raw = F.bsum_raw + z.b; F.bn = F.bn + 1; end
                if fabs_(phase - p.mtq_meas) < dt/2.0 && F.bn == 0
                    F.b1raw = [];
                elseif fabs_(phase - p.mtq_meas) < dt/2.0
                    bav = F.bsum_raw*(1.0/F.bn);
                    if F.mode == 5
                        bd = zeros(3, 1);
                        if ~isempty(F.b1raw), for i = 1:3, bd(i) = (bav(i) - F.b1raw(i))/p.mtq_period; end, end
                        m0 = asils.alg.control.gen_bdot(bav, bd, [0.0; 0.0; F.sigma*p.ss_spin_dps*D2R], p.ss_k_l1);
                    else
                        ecl = (~z.sun_ok && p.ss_eclipse == 1) || isempty(F.s_prop);
                        if isempty(F.s_prop), s = zeros(3, 1); else, s = F.s_prop; end
                        if p.ss_law == 1
                            m0 = asils.alg.control.sun_spin_deruiter(bav, F.w_est, s, ecl, p.J, p.ss_spin_dps, p.ss_dr_k, p.ss_dr_k1, p.ss_dr_k2);
                        else
                            m0 = asils.alg.control.sun_spin(bav, F.w_est, s, ecl, p.J, p.ss_spin_dps, p.ss_k1, p.ss_k2, p.ss_rz_floor);
                        end
                    end
                    % P8 Celani 2026: power face onto the Sun, no spin
                    if p.ss_law == 2
                        bs = bav(1)*bav(1) + bav(2)*bav(2) + bav(3)*bav(3);
                        if ~isempty(F.s_prop) && bs >= 1e-18
                            tau = asils.alg.control.mtq_boresight(p.sun_axis, unit_(F.s_prop), F.w_est, p.sb_kp, p.sb_kd);
                            c = cross3_(bav, tau);
                            m0 = [c(1)/bs; c(2)/bs; c(3)/bs];
                        else
                            m0 = zeros(3, 1);
                        end
                    end
                    F.b1raw = bav;
                    if ~is_zero3_(m0), m_body = asils.alg.control.sat_dipole(m0 - p.m_res_est, p.m_max); end
                end
            end
            if nr > 0, cmd_r = idle_rotors_(F, cmd_r, false); end
        otherwise
            m_body = zeros(3, 1);                      % no such state (refused at init and by command): coils off
    end
    if F.mag_seen && F.mag_age > 2.0*p.mtq_period, m_body = zeros(3, 1); end     % no field to act on
    F.m_hold = m_body;
    F.clean = is_zero3_(m_body);
    bus = asils.fsw.drv_write(bus, p, m_body, cmd_r, cmd_g, duty);
    F.out_m = m_body; F.out_r = cmd_r; F.out_g = cmd_g; F.out_duty = duty;
end

% ---------------- the runtime's helpers (fsw-rs math.rs, the toolbox's small kernel) ----------------
function y = fabs_(x)
    if x < 0, y = -x; elseif x == 0, y = 0; else, y = x; end
end
function y = sign_(x)
    if x > 0, y = 1; elseif x < 0, y = -1; else, y = 0; end
end
function r = fmod_(x, y)
    r = rem(x, y);
end
function n = norm3_(a)
    n = sqrt(a(1)*a(1) + a(2)*a(2) + a(3)*a(3));
end
function u = unit_(a)
    n = norm3_(a); if n < 1e-30, n = 1e-30; end
    u = [a(1)/n; a(2)/n; a(3)/n];
end
function c = cross3_(a, b)
    c = [a(2)*b(3) - a(3)*b(2); a(3)*b(1) - a(1)*b(3); a(1)*b(2) - a(2)*b(1)];
end
function r = mat3_vec_(m, v)
    r = [m(1,1)*v(1) + m(1,2)*v(2) + m(1,3)*v(3); m(2,1)*v(1) + m(2,2)*v(2) + m(2,3)*v(3); m(3,1)*v(1) + m(3,2)*v(2) + m(3,3)*v(3)];
end
function z = is_zero3_(a)
    z = a(1) == 0 && a(2) == 0 && a(3) == 0;
end
function b = bref_(F)
    if isempty(F.bref), b = zeros(3, 1); else, b = F.bref; end
end

% ---------------- the MEKF's state (fsw-rs est.rs) ----------------
function K = mekf_new_(q0, sa, sb, arw, rrw)
    [q, b, P] = asils.alg.estimation.mekf_init(q0, sa, sb);
    K = struct('q', q, 'b', b, 'P', P, 'arw', arw, 'rrw', rrw);
end
function [K, took] = mekf_vector_(K, bm, rr, sigma, gate)
    [K.q, K.b, K.P, took] = asils.alg.estimation.mekf_vector(K.q, K.b, K.P, bm, rr, sigma, gate);
end

% ---------------- the records of the generated mode manager and step laws ----------------
function s = modes_state_(F)
    if isempty(F.s_prop), sp = zeros(3, 1); spo = false; else, sp = F.s_prop; spo = true; end
    s = struct('mode', F.mode, 't', F.t, 't_mode', F.t_mode, 'hold', F.hold, 'ho', F.ho, 'ho_t', F.ho_t, 'ad_ok', F.ad_ok, ...
        'n_rej', F.n_rej, 'i_q', F.i_q, 'sz_sum', F.sz_sum, 'sz_n', F.sz_n, 'sz_t0', F.sz_t0, 'sigma', F.sigma, ...
        's_prop', sp, 's_prop_ok', spo, 'acq_hold', F.acq_hold, 'sched_i', F.sched_i, 'faults', F.faults, ...
        'w_est', F.w_est, 'mag_seen', F.mag_seen, 'mag_age', F.mag_age, 'gyro_age', F.gyro_age);
end
function F = set_modes_(F, s)
    F.mode = s.mode; F.t = s.t; F.t_mode = s.t_mode; F.hold = s.hold; F.ho = s.ho; F.ho_t = s.ho_t;
    F.ad_ok = s.ad_ok; F.n_rej = s.n_rej; F.i_q = s.i_q; F.sz_sum = s.sz_sum; F.sz_n = s.sz_n;
    F.sz_t0 = s.sz_t0; F.sigma = s.sigma; if s.s_prop_ok, F.s_prop = s.s_prop; else, F.s_prop = []; end
    F.acq_hold = s.acq_hold; F.sched_i = s.sched_i; F.faults = s.faults; F.w_est = s.w_est; F.mag_seen = s.mag_seen;
    F.mag_age = s.mag_age; F.gyro_age = s.gyro_age;
end
function s = ctl_state_(F)
    if isempty(F.s_prop), sp = zeros(3, 1); spo = false; else, sp = F.s_prop; spo = true; end
    s = struct('q', F.K.q, 'w_est', F.w_est, 'q_ref', F.q_ref, 'w_ref', F.w_ref, 'i_q', F.i_q, 'tau_req', F.tau_req, ...
        'mode', F.mode, 's_prop', sp, 's_prop_ok', spo, 'cap', F.cap, 'hcap', F.hcap);
end
function F = set_ctl_(F, s)
    F.K.q = s.q; F.w_est = s.w_est; F.q_ref = s.q_ref; F.w_ref = s.w_ref; F.i_q = s.i_q; F.tau_req = s.tau_req;
    F.mode = s.mode; if s.s_prop_ok, F.s_prop = s.s_prop; else, F.s_prop = []; end
    F.cap = s.cap; F.hcap = s.hcap;
end
function cmd_r = idle_rotors_(F, cmd_r, zero_cmg)
    p = F.p;
    cmd_r = asils.alg.steplaws.alloc_idle(cmd_r, p.nr, F.z.h, F.h_t_rot, p.rot_gi, p.rot_kind, zero_cmg);
end
function F = fdir_rotors_(F, z, dt)
    if isempty(F.h_prev), hp = zeros(8, 1); hpo = false; else, hp = F.h_prev; hpo = true; end
    st = struct('h_prev', hp, 'h_prev_ok', hpo, 'cmd_r_prev', F.cmd_r_prev, 'fd_count', F.fd_count, 'rot_failed', F.rot_failed, ...
        'faults', F.faults, 'fw_e', F.fw_e, 'fw_h0', F.fw_h0, 'fw_t0', F.fw_t0, 'fw_last', F.fw_last, 'fw_on', F.fw_on, 'fw_bad', F.fw_bad);
    s = asils.alg.modes.fdir_rotors(st, F.rp, F.t, dt, z.h);
    if s.h_prev_ok, F.h_prev = s.h_prev; else, F.h_prev = []; end
    F.cmd_r_prev = s.cmd_r_prev; F.fd_count = s.fd_count; F.rot_failed = s.rot_failed; F.faults = s.faults;
    F.fw_e = s.fw_e; F.fw_h0 = s.fw_h0; F.fw_t0 = s.fw_t0; F.fw_last = s.fw_last; F.fw_on = s.fw_on; F.fw_bad = s.fw_bad;
end
