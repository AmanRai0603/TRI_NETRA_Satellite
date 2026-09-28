function [tau_B, mdot, P_W, on_s] = rcs(duty, D, r, T)
%ASILS.DEVICES.RCS  Cold-gas thruster couples, pulse-width modulated.
%   duty (1 x nc) in [0,1] per couple over the control period T. Each couple
%   is two nozzles firing together (pure torque r.tau_couple(:,k), no net
%   force). A pulse shorter than the minimum impulse bit is not fired;
%   longer pulses are quantised to the valve resolution. Thrust scale and
%   nozzle misalignment are drawn per part (asils.devices.init).
%   Returns the period-average body torque, propellant flow [kg/s], valve power.
    on_s = max(0, min(1, duty(:)'))*T;
    on_s(on_s < r.mib_s) = 0;
    on_s = round(on_s/r.valve_res_s)*r.valve_res_s;
    on_s(D.failed) = 0;
    f = on_s/T;
    tau_B = D.tau_couple*(f(:).*D.tscale(:));
    isp = r.isp_s; if isfield(D, 'isp'), isp = D.isp; end
    mdot = sum(f.*D.tscale)*2*r.thrust_N/(isp*9.80665);
    P_W = r.valve_power_W*sum(on_s > 0);
end
