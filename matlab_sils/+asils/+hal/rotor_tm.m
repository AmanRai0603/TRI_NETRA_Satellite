function f = rotor_tm(i, h, delta, s)
%ASILS.HAL.ROTOR_TM  Rotor i's telemetry CAN frame (i from 0): [id 0x200 + i, dlc 8, h int32 [1e-9 N m s], gimbal angle
%   int32 [1e-7 rad]] (the counts asils.models.emucodec.rotor_counts).
%   Owner: Agastya. Copyright (c) 2026 Agastya. All rights reserved.
    [ch, cd] = asils.models.emucodec.rotor_counts(h, delta, s);
    x = asils.hal.le32([ch; cd]).';
    f = [512 + i, 8, x(:).'];
end
