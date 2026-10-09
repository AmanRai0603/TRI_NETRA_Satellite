function out = st_frame(ok, q, nh, s)
%ASILS.HAL.ST_FRAME  The star tracker's UART frame: nh, then per head valid u8 and q (x y z w) int32 Q30 (the counts
%   asils.models.emucodec.quat_counts). ok: nh flags; q: nh x 4.
%   Owner: Agastya. Copyright (c) 2026 Agastya. All rights reserved.
    nh = min(nh, 2);
    pl = zeros(1 + 17*nh, 1); pl(1) = nh;
    for h = 1:nh
        x = asils.hal.le32(asils.models.emucodec.quat_counts(q(h, :).', s)).';
        pl(2 + 17*(h - 1):1 + 17*h) = [double(ok(h)); x(:)];
    end
    out = asils.hal.frame(144, pl);
end
