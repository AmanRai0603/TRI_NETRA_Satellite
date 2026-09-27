function [duty, tau] = rcs_duty(req, r, T)
%ASILS.FSW.RCS_DUTY  Requested body torque -> valve on-time of each thruster couple
%   (one +/- couple per body axis) and the torque the FSW then expects.
%   The FSW quantises its own valve commands exactly as the valves will
%   (minimum impulse bit, valve resolution): a request below the MIB is NOT
%   fired and NOT fed forward to the momentum devices.
    Tc = r.tau_couple; duty = zeros(1, size(Tc, 2)); tau = zeros(3, 1);
    for ax = 1:3
        u = req(ax);
        if u == 0, continue, end
        if u > 0, k = 2*ax - 1; else, k = 2*ax; end
        on = min(1, abs(u)/abs(Tc(ax, k)))*T;
        if on < r.mib_s, continue, end
        on = round(on/r.valve_res_s)*r.valve_res_s;
        duty(k) = on/T;
        tau = tau + Tc(:, k)*duty(k);
    end
end
