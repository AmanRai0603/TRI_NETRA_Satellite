function out = frame(sync2, pl)
%ASILS.HAL.FRAME  A UART frame: EB sync2 n payload CRC-16 (little-endian), the rig's framing (the engine's emu.rs frame).
%   Owner: Agastya. Copyright (c) 2026 Agastya. All rights reserved.
    c = asils.hal.crc16(pl);
    out = [235; sync2; numel(pl); pl(:); mod(c, 256); floor(c/256)];
end
