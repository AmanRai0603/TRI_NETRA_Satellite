function F = fdir(F, z, t, dt, G, dev)
%ASILS.FSW.FDIR  Rotor FDIR in the momentum-device states (fsw/pseudocode/07): a fixed rotor
%   that does not follow its command is isolated. Instantaneous: the momentum rate against the
%   command clipped to what the device can do; windowed: the momentum change over fdir_win_s
%   against the change commanded, fluid loops only. Group fdir (design/groups.toml); C:
%   adcs_fdir.c; Rust: fdir.rs. (The twin has no sensor-health safe mode: the C and Rust
%   flight software's adcs_fdir_sensors / adcs_fdir_safe.)
%   Owner: Agastya. Copyright (c) 2026 Agastya. All rights reserved.
    nr = F.M.nr;
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
        % windowed: the momentum each fixed rotor was commanded to change over fdir_win_s
        % against the change measured; catches a rotor that does not follow the small
        % commands of fine pointing (fsw/pseudocode/07)
        if ~F.fw_on || t - F.fw_last > 1.5*dt
            F.fw_E = zeros(nr, 1); F.fw_h0 = z.h(:); F.fw_t0 = t; F.fw_on = true;
        else
            F.fw_E = F.fw_E + max(-0.8*tmax, min(0.8*tmax, F.cmd_r_prev(:)))*dt;
            if t - F.fw_t0 >= G.fdir_win_s - 1e-9
                hmax = dev.mex.h_max(:); E = F.fw_E; Mh = z.h(:) - F.fw_h0;
                judged = strcmp(dev.mex.kind(:), 'fmr') & F.M.gi(:) == 0 & ~F.rot_failed(:) & abs(E) > G.fdir_h_frac*hmax ...
                         & abs(z.h(:)) < 0.9*hmax & abs(F.fw_h0) < 0.9*hmax;
                miss = abs(Mh - E) > 0.5*abs(E);
                F.fw_bad(judged) = (F.fw_bad(judged) + 1).*miss(judged);
                newly = judged & F.fw_bad >= 2;            % FDIR_WIN_BAD
                if any(newly)
                    F.rot_failed(newly) = true;
                    F.log(end+1).t = t; F.log(end).mode = sprintf('FDIR: rotor %d isolated (momentum window)', find(newly, 1));
                end
                F.fw_E = zeros(nr, 1); F.fw_h0 = z.h(:); F.fw_t0 = t;
            end
        end
        F.fw_last = t;
    end
end
