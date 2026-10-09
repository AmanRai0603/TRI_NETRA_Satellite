function b = regs(ok, c)
%ASILS.HAL.REGS  An I2C register block: status (bit 0 valid), three int16 counts c, little-endian (the engine's emu.rs
%   reg7). The counts are the design's (asils.models.emucodec: mag_counts, unit_counts).
%   Owner: Agastya. Copyright (c) 2026 Agastya. All rights reserved.
    c = c(:); c(c < 0) = c(c < 0) + 65536;
    b = [double(ok); mod(c(1), 256); floor(c(1)/256); mod(c(2), 256); floor(c(2)/256); mod(c(3), 256); floor(c(3)/256)];
end
