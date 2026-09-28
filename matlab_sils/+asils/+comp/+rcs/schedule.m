function [on_s, tau] = schedule(req, rcs, Tc)
%ASILS.COMP.RCS.SCHEDULE  N2O thruster valve schedule for one PWM period Tc:
%   torque request -> the couple for each axis sign -> on-time quantised to the
%   minimum impulse bit and valve resolution (a request below the MIB is not
%   fired and not fed forward). Wraps asils.fsw.rcs_duty.
    [duty, tau] = asils.fsw.rcs_duty(req, rcs, Tc);
    on_s = duty*Tc;
end
