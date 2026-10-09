function [U, done, log] = apply(faults, t, U, done, log)
%ASILS.FAULTS.APPLY  Inject every fault whose time has come (once each), clear the ones that end, and log both: the
%   engine's run.rs Units::faults, in MATLAB (the runner's fault injection, core).
%   faults: the scenario's list (kind, t_s, index from 1, value, end_s); done: which are in; log: the mode log
%   (struct array t, mode). The flight software is NOT told: it must detect what it can.
%   Owner: Agastya. Copyright (c) 2026 Agastya. All rights reserved.
    for i = 1:numel(faults)
        if iscell(faults), f = faults{i}; else, f = faults(i); end
        idx = asils.util.getf(f, 'index', 0);
        if done(i)
            % a device back from silence (end_s): it answers again from then on
            e = asils.util.getf(f, 'end_s', []);
            if ~isempty(e) && t >= e
                if strcmp(f.kind, 'gps_outage')
                    if U.gps.dead, U.gps.dead = false; log(end+1) = struct('t', t, 'mode', sprintf('FAULT cleared: %s %d', f.kind, idx)); end %#ok<AGROW>
                elseif U.mag.dead
                    U.mag.dead = false; log(end+1) = struct('t', t, 'mode', sprintf('FAULT cleared: %s %d', f.kind, idx)); %#ok<AGROW>
                end
            end
            continue
        end
        if t < f.t_s, continue, end
        done(i) = true;
        ix = max(idx, 1);
        switch f.kind
            case 'rotor_fail',     U.mex.failed(ix) = true;
            case 'gimbal_stuck',   U.mex.gfailed(ix) = true;
            case 'st_head_fail',   if ~isempty(U.st), U.st.dead(ix) = true; end
            case 'coil_fail',      U.mtq.dead(ix) = true;
            case 'gyro_bias_step', if ~isempty(U.gyro), U.gyro.b = U.gyro.b + f.value(:); end
            case 'gps_outage',     U.gps.dead = true;
            case 'rcs_valve_fail', if ~isempty(U.rcs), U.rcs.failed(ix) = true; end
            case 'mag_fail',       U.mag.dead = true;
            otherwise, error('asils:faults:kind', 'unknown fault %s', f.kind);
        end
        log(end+1) = struct('t', t, 'mode', sprintf('FAULT injected: %s %d', f.kind, idx)); %#ok<AGROW>
    end
end
