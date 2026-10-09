function F = init(blob, start_ns)
%ASILS.FSW.INIT  Boot the twin's flight software from an adcs-fswcfg/1 blob: the runtime of
%   fsw-rs/src/fsw.rs (Fsw::init) and fsw/src/adcs_fsw.c (adcs_fsw_init), in MATLAB.
%
%   F = asils.fsw.init(blob)            the blob's bytes (uint8), as the engine builds it
%   F = asils.fsw.init(blob, start_ns)  the bus time [ns] of tick 0
%
%   The parameters are the blob's (asils.fsw.params_decode, generated from fsw/params/params.toml):
%   the same values the engine's flight software boots with (`adcs params`), never recomputed here.
%   The algorithms are the design's, generated into +asils/+alg by tools/flight_build.py; this file
%   is the runtime around them (the state, its start, the targets the laws act about). A state the
%   fitted hardware cannot fly is refused, as adcs_fsw_init refuses it.
%   Owner: Agastya. Copyright (c) 2026 Agastya. All rights reserved.
    if nargin < 2, start_ns = 0; end
    if isstruct(blob), p = blob; else, p = asils.fsw.params_decode(blob); end
    NR = 8; NG = 4; NC = 6;
    ms = p.start_mode;
    if p.auto_next ~= 255, ms = [ms, p.auto_next]; end
    ms = [ms, reshape(p.sched_mode(1:p.n_sched), 1, [])];
    for m = ms
        if ~asils.alg.modes.modes_feasible(p.nr, p.nc, m)
            error('asils:fsw:infeasible', 'the flight software refuses controller state %d: the fitted hardware cannot fly it', m);
        end
    end
    F = struct();
    F.p = p; F.ready = true; F.start_ns = start_ns;
    F.t = 0; F.jd = p.jd0; F.mode = p.start_mode; F.t_mode = 0; F.hold = 0;
    F.z = struct('mag_ok', false, 'b', zeros(3, 1), 'gyro_ok', false, 'w', zeros(3, 1), 'sun_ok', false, 'sun', zeros(3, 1), ...
                 'es_ok', false, 'nadir', zeros(3, 1), 'st_ok', false, 'st_valid', zeros(2, 1), 'q_st', zeros(2, 4), ...
                 'gps_ok', false, 'r', zeros(3, 1), 'v', zeros(3, 1), 'h', zeros(NR, 1), 'delta', zeros(NG, 1));
    F.rx = {zeros(0, 1), zeros(0, 1), zeros(0, 1)};       % the UART frame assembly buffers, ports 0 to 2
    F.clean = true;
    F.mag_age = 0; F.gyro_age = 0; F.mag_seen = false; F.b_good = zeros(3, 1);
    F.have_r = false; F.r = zeros(3, 1); F.v = zeros(3, 1);
    F.K = struct('q', zeros(4, 1), 'b', zeros(3, 1), 'P', zeros(6, 6), 'arw', 0, 'rrw', 0);
    F.ad_ok = false; F.t_st = -1e9; F.mag_done = false; F.n_rej = 0; F.gh_jd = 0;
    F.es_n = zeros(3, 1); F.es_t = -1e9; F.w_est = zeros(3, 1);
    F.gh = []; F.bref = []; F.t_bref = -1e9;
    F.bsum = zeros(3, 1); F.bsum_raw = zeros(3, 1); F.bn = 0; F.b1 = []; F.b1raw = [];
    F.m_hold = zeros(3, 1); F.b_dump = zeros(3, 1);
    F.sigma = p.ss_sigma0; F.sz_sum = 0; F.sz_n = 0; F.sz_t0 = 0; F.s_prop = []; F.acq_hold = 0;
    F.ho = false; F.ho_t = 0;
    F.gd = struct('q_off', p.gd_q_off, 'roll_deg', p.gd_roll_deg, 't0', p.gd_t0, 't_slew', p.gd_T, 'axis', p.gd_axis, ...
                  'q_inertial', p.gd_q_inertial, 'sun_axis', p.sun_axis, 'roll_axis', p.roll_axis, ...
                  'sun_eci', asils.alg.frames.sun_model(p.jd0), 'flip', false);
    F.g_rw = gains_(p.rw_law, p.rw_Kp, p.rw_Kd, p.rw_Ki, p.rw_Klqr, p.rw_lambda, p.rw_phi, p.rw_Gs, p.rw_err_max, p.rw_int_max);
    ml = 0; if p.mtq_law == 1, ml = 1; elseif p.mtq_law == 2, ml = 2; end
    F.g_mtq = gains_(ml, p.mtq_Kp, p.mtq_Kd, p.mtq_Ki, p.mtq_Klqr, p.mtq_lambda, p.mtq_phi, p.mtq_Gs, p.mtq_err_max, p.mtq_int_max);
    F.q_ref = zeros(4, 1); F.w_ref = zeros(3, 1); F.tau_req = zeros(3, 1); F.i_q = zeros(3, 1); F.last_ctrl = -1e9; F.capturing = false;
    F.h_prev = []; F.cmd_r_prev = zeros(NR, 1); F.rot_failed = false(NR, 1); F.fd_count = zeros(NR, 1);
    F.fw_e = zeros(NR, 1); F.fw_h0 = zeros(NR, 1); F.fw_t0 = 0; F.fw_last = 0; F.fw_on = false; F.fw_bad = zeros(NR, 1);
    % momentum targets and per-axis authority (fsw.rs Fsw::init)
    F.h_t_rot = zeros(NR, 1); F.h_t = zeros(3, 1); F.cap = zeros(3, 1); F.hcap = zeros(3, 1);
    for i = 1:p.nr
        if p.rot_kind(i) == 0
            if p.h_bias < 0.25*p.rot_hmax(i), F.h_t_rot(i) = p.h_bias; else, F.h_t_rot(i) = 0.25*p.rot_hmax(i); end
        elseif p.rot_kind(i) >= 2
            F.h_t_rot(i) = p.rot_h0(i);
        end
        if p.rot_gi(i) == 0
            for k = 1:3
                F.cap(k) = F.cap(k) + fabs_(p.rot_a0(i, k))*p.rot_tmax(i);
                F.hcap(k) = F.hcap(k) + fabs_(p.rot_a0(i, k))*p.rot_hmax(i);
                F.h_t(k) = F.h_t(k) + p.rot_a0(i, k)*F.h_t_rot(i);
            end
        end
    end
    if p.ng > 0
        h0 = 0;
        for i = 1:p.nr, if p.rot_h0(i) > h0, h0 = p.rot_h0(i); end, end
        for k = 1:3, F.cap(k) = F.cap(k) + 2*h0*p.gim_rate_max; F.hcap(k) = F.hcap(k) + 2.5*h0; end
    end
    F.dump_hi = p.rcs_dump_hi; F.dump_lo = p.rcs_dump_lo;
    if p.nr > 0
        hmin = 1e300;
        for k = 1:3, if F.hcap(k) < hmin, hmin = F.hcap(k); end, end
        if 0.5*hmin < F.dump_hi, F.dump_hi = 0.5*hmin; end
        if 0.15*hmin < F.dump_lo, F.dump_lo = 0.15*hmin; end
    end
    F.has_rcs_dump = p.nc > 0 && p.rcs_dump ~= 0; F.rcs_dumping = false; F.rcs_left = [];
    F.sched_i = 0;
    F.out_m = zeros(3, 1); F.out_r = zeros(NR, 1); F.out_g = zeros(NG, 1); F.out_duty = zeros(NC, 1);
    F.faults = 0;
    % the parameters the generated records take, made once (they are the blob's)
    F.mp = mode_params_(p);
    F.rp = struct('nr', p.nr, 'rot_tmax', p.rot_tmax, 'rot_hmax', p.rot_hmax, 'rot_gi', p.rot_gi, 'rot_kind', p.rot_kind, ...
                  'fdir_s', p.fdir_s, 'fdir_win_s', p.fdir_win_s, 'fdir_h_frac', p.fdir_h_frac);
    F.cp = ctl_params_(p, F.g_mtq);
end

function g = gains_(law, kp, kd, ki, klqr, lambda, phi, gs, err_max, int_max)
    g = struct('law', law, 'kp', kp, 'kd', kd, 'ki', ki, 'klqr', klqr, 'lambda', lambda, 'phi', phi, 'gs', gs, ...
               'err_max', err_max, 'int_max', int_max);
end

function y = fabs_(x)
    if x < 0, y = -x; elseif x == 0, y = 0; else, y = x; end
end

function mp = mode_params_(p)
    mp = struct('auto_next', p.auto_next, 'n_sched', p.n_sched, 'sched_t', p.sched_t, 'sched_mode', p.sched_mode, ...
        'detumble_exit', p.detumble_exit, 'detumble_hold_s', p.detumble_hold_s, 'ss_law', p.ss_law, ...
        'ss_omega_max_dps', p.ss_omega_max_dps, 'ss_spin_dps', p.ss_spin_dps, 'ss_z_in_dps', p.ss_z_in_dps, ...
        'ss_perp_in_dps', p.ss_perp_in_dps, 'ss_t_check_s', p.ss_t_check_s, 'ss_sun_min', p.ss_sun_min, ...
        'ss_dwell_in_s', p.ss_dwell_in_s, 'ss_omega_exit_dps', p.ss_omega_exit_dps, 'ss_perp_out_dps', p.ss_perp_out_dps, ...
        'ss_dwell_out_s', p.ss_dwell_out_s, 'sun_axis', p.sun_axis, 'sa_done_deg', p.sa_done_deg, 'sa_done_hold_s', p.sa_done_hold_s);
end

function c = ctl_params_(p, g)
    c = struct('mtq_law', p.mtq_law, 'mtq_eps', p.mtq_eps, 'mtq_k1', p.mtq_k1, 'mtq_k2', p.mtq_k2, 'mtq_k16', p.mtq_k16, ...
        'mtq_lam16', p.mtq_lam16, 'sb_kp', p.sb_kp, 'sb_kd', p.sb_kd, 'sb_kroll', p.sb_kroll, 'sb_kdroll', p.sb_kdroll, ...
        'sb_roll_gate', p.sb_roll_gate, 'sun_axis', p.sun_axis, 'roll_axis', p.roll_axis, 'mtq_pth', p.mtq_Pth, 'mtq_pw', p.mtq_Pw, ...
        'mtq_period', p.mtq_period, 'j', p.J, 'capture_deg', p.capture_deg, 'capture_rate_deg_s', p.capture_rate_deg_s, ...
        'sa_w_max_deg_s', p.sa_w_max_deg_s, 'sa_kd', p.sa_kd, 'g_law', g.law, 'g_kp', g.kp, 'g_kd', g.kd, 'g_ki', g.ki, ...
        'g_klqr', g.klqr, 'g_lambda', g.lambda, 'g_phi', g.phi, 'g_gs', g.gs, 'g_err_max', g.err_max, 'g_int_max', g.int_max);
end
