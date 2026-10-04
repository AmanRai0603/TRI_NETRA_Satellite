function F = mode_manager(F, z, t, dt, G, dev)
%ASILS.FSW.MODE_MANAGER  The mode manager (fsw/pseudocode/08_mode_manager.md): commanded
%   changes, the detumble exit, the spin guards, the gyro-propagated Sun vector of the
%   acquisition laws, and the Sun acquisition's end. Group gdn (design/groups.toml); C:
%   adcs_modes.c; Rust: modes.rs. The controller states it moves between: asils.fsw.modes.
%   Owner: Agastya. Copyright (c) 2026 Agastya. All rights reserved.
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
    F.mode = mode; F.t_mode = t; F.hold = 0; F.ho = false; F.ho_t = 0; F.I_q = zeros(3,1);
    if any(strcmp(mode, {'detumble', 'detumble_rcs', 'spinup', 'sun_spin'})), F.ad_ok = false; end   % re-initialise after
    F.log(end+1).t = t; F.log(end).mode = mode;
end
