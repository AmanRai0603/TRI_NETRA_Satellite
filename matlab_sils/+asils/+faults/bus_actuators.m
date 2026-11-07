function [B, cmd] = bus_actuators(B, t, cmd)
%ASILS.FAULTS.BUS_ACTUATORS  After the step: the commands that reach the actuators. A failed or stuck write leaves the device on its
%   previous command; a device without supply (bus_brownout) gets none. The engine's BusFaults::commands, in MATLAB.
%   Owner: Agastya. Copyright (c) 2026 Agastya. All rights reserved.
    if isempty(B.w), return, end
    if asils.faults.bus_on(B, t, 'coils', {'bus_nack', 'bus_stuck'})
        cmd.m_body = B.prev_m;
    elseif asils.faults.bus_on(B, t, 'coils', {'bus_brownout'})
        cmd.m_body = zeros(3, 1);
    end
    if asils.faults.bus_on(B, t, 'momentum_devices', {'bus_nack', 'bus_stuck', 'bus_can_error'})
        cmd.cmd_r = B.prev_r; cmd.cmd_g = B.prev_g; cmd.duty = B.prev_d;
    elseif asils.faults.bus_on(B, t, 'momentum_devices', {'bus_brownout'})
        cmd.cmd_r = zeros(8, 1); cmd.cmd_g = zeros(4, 1); cmd.duty = zeros(6, 1);
    end
    B.prev_m = cmd.m_body; B.prev_r = cmd.cmd_r; B.prev_g = cmd.cmd_g; B.prev_d = cmd.duty;
end
