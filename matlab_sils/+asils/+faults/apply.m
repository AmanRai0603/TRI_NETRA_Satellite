function [D, F] = apply(faults, t, D, F)
%ASILS.FAULTS.APPLY  Inject scheduled faults into the devices (once, at t_s).
%   faults: struct array (from the scenario) with fields
%     kind   'rotor_fail' | 'gimbal_stuck' | 'st_head_fail' | 'coil_fail' |
%            'gyro_bias_step' | 'gps_outage' | 'rcs_valve_fail' | 'mag_fail'
%     t_s    time of the fault [s]
%     index  which unit (rotor, gimbal, head, coil, couple); ignored otherwise
%     value  size of the fault where it has one (gyro bias step [rad/s], 3x1)
%   The FSW is NOT told: it must detect what it can (asils.fsw.step FDIR).
    if ~isfield(D, 'fault_done'), D.fault_done = false(1, numel(faults)); end
    for i = 1:numel(faults)
        f = faults(i);
        if iscell(faults), f = faults{i}; end
        if D.fault_done(i) || t < f.t_s, continue, end
        D.fault_done(i) = true;
        switch f.kind
            case 'rotor_fail',     D.mex.failed(f.index) = true;
            case 'gimbal_stuck',   D.mex.gfailed(f.index) = true;
            case 'st_head_fail',   D.st.dead(f.index) = true;
            case 'coil_fail',      D.mtq.dead(f.index) = true;
            case 'gyro_bias_step', D.gyro.b = D.gyro.b + f.value(:);
            case 'gps_outage',     D.gps_dead = true;
            case 'rcs_valve_fail', D.rcs.failed(f.index) = true;
            case 'mag_fail',       D.mag.dead = true;
            otherwise, error('asils:faults:kind', 'unknown fault %s', f.kind);
        end
        F.log(end+1).t = t; F.log(end).mode = sprintf('FAULT injected: %s %d', f.kind, asils.util.getf(f, 'index', 0));
    end
end
