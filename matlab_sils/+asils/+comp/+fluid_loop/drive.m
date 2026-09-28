function [pump, D] = drive(i, cmd_r, h, D, m, dt)
%ASILS.COMP.FLUID_LOOP.DRIVE  Fluid-loop driver (in-house): momentum-rate
%   command -> pump drive for ring i. Integrates the commanded momentum into a
%   target, filters the flow sensor, feeds forward the MODELLED laminar loss and
%   servoes the measured flow to the target (gain k_flow), limited by the pump.
%   A loss model 20 % wrong then leaves no torque error proportional to the
%   stored momentum. (Moved out of the device model: this is the unit's
%   firmware, asils.devices.mex is the physics.)
    Tsd = m.T_sd(i);
    D.htgt(i) = max(-m.h_max(i), min(m.h_max(i), D.htgt(i) + cmd_r*dt));
    D.hf(i) = D.hf(i) + dt/(m.flow_tau + dt)*(h + m.flow_noise_h(i)*randn - D.hf(i));
    pump = cmd_r + D.hf(i)/Tsd + m.k_flow*(D.htgt(i) - D.hf(i));
    pump = max(-m.torque_max(i), min(m.torque_max(i), pump));
end
