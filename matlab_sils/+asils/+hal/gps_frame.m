function out = gps_frame(fix, r, v, s, jd)
%ASILS.HAL.GPS_FRAME  The GNSS receiver's UART frame: fix u8, r int32 [cm] x3, v int32 [mm/s] x3 (the counts
%   asils.models.emucodec.fix_counts) and, when the receiver gives its date (jd, a Julian date), the date after the velocity:
%   int32 whole seconds and int32 microseconds from JD 2451545.0 (J2000.0) (= emu.rs gps_frame).
%   Owner: Agastya. Copyright (c) 2026 Agastya. All rights reserved.
    [cr, cv] = asils.models.emucodec.fix_counts(r, v, s);
    a = asils.hal.le32(cr).'; b = asils.hal.le32(cv).';
    pl = [double(fix); a(:); b(:)];
    if nargin > 4
        ds = (jd - 2451545.0)*86400.0; sec = floor(ds); us = round((ds - sec)*1e6);
        if us >= 1e6, sec = sec + 1; us = us - 1e6; end
        c = asils.hal.le32([sec; us]).';
        pl = [pl; c(:)];
    end
    out = asils.hal.frame(145, pl);
end
