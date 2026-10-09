function out = gps_frame(fix, r, v, s)
%ASILS.HAL.GPS_FRAME  The GNSS receiver's UART frame: fix u8, r int32 [cm] x3, v int32 [mm/s] x3 (the counts
%   asils.models.emucodec.fix_counts).
%   Owner: Agastya. Copyright (c) 2026 Agastya. All rights reserved.
    [cr, cv] = asils.models.emucodec.fix_counts(r, v, s);
    a = asils.hal.le32(cr).'; b = asils.hal.le32(cv).';
    out = asils.hal.frame(145, [double(fix); a(:); b(:)]);
end
