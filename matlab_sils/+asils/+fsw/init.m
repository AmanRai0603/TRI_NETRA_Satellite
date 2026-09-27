function F = init(P, jd0)
%ASILS.FSW.INIT  Flight-software state at power-on (threaded through every tick,
%   no persistent/global -- the Standard Code rule).
    F.P = P.fsw; F.jd0 = jd0;
    F.mode = F.P.start_mode; F.t_mode = 0; F.hold = 0;
    F.K = []; F.ad_ok = false; F.t_st = -1e9;
    F.r = []; F.v = []; F.t_fix = -1;
    F.b1 = []; F.bsum = zeros(3,1); F.bn = 0; F.m_body = zeros(3,1); F.m_hold = zeros(3,1);
    dev = P.dev; X = dev.mex;
    F.M = asils.plant.geometry(X.A0, X.G, X.gi);        % NOMINAL geometry (FSW never sees the true misalignment)
    nr = F.M.nr;
    F.rot_failed = false(nr, 1); F.fd_count = zeros(nr, 1); F.h_prev = []; F.cmd_r_prev = zeros(nr, 1);
    F.idmas = any(strcmp(dev.algorithms, 'idmas_split'));
    F.nc = 0; if dev.rcs.fitted, F.nc = size(dev.rcs.tau_couple, 2); end
    F.has_rcs_dump = F.nc > 0 && F.P.rcs.dump;
    F.rcs_dumping = false;
    % momentum target of the devices and their per-axis torque authority
    F.h_t_rot = zeros(nr, 1); F.H_t = zeros(3, 1); F.cap = zeros(3, 1); F.hcap = zeros(3, 1);
    for i = 1:nr
        switch X.kind{i}
            case 'rw',  F.h_t_rot(i) = F.P.h_bias;           % wheels biased off zero speed
            case {'cmg', 'vscmg'}, F.h_t_rot(i) = X.h0(i);
        end
        if X.gi(i) == 0
            F.cap = F.cap + abs(X.A0(:,i))*X.torque_max(i);
            F.hcap = F.hcap + abs(X.A0(:,i))*X.h_max(i);
        end
    end
    if F.M.ng > 0
        F.cap = F.cap + 2*max(X.h0)*X.gimbal_rate_max*ones(3,1);
        F.hcap = F.hcap + 2.5*max(X.h0)*ones(3,1);        % pyramid envelope ~ 2.5 h0 per axis
    end
    for i = 1:nr, if X.gi(i) == 0, F.H_t = F.H_t + X.A0(:,i)*F.h_t_rot(i); end, end
    % thruster dump thresholds follow the product's momentum capacity
    if nr > 0
        F.P.rcs.dump_hi = min(F.P.rcs.dump_hi, 0.5*min(F.hcap));
        F.P.rcs.dump_lo = min(F.P.rcs.dump_lo, 0.15*min(F.hcap));
    end
    F.I_q = zeros(3,1); F.q_ref = nan(4,1); F.w_ref = nan(3,1);
    F.gd = F.P.guidance; F.gh = []; F.Bref = []; F.t_Bref = -1e9;
    F.last_ctrl = -1e9; F.tau_req = zeros(3,1); F.B_dump = zeros(3,1); F.m_dump = zeros(3,1);
    F.w_est = zeros(3,1);
    F.log = struct('t', {}, 'mode', {});
    F.log(1).t = 0; F.log(1).mode = F.mode;
end
