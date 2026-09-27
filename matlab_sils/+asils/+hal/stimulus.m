function S = stimulus(rec)
%ASILS.HAL.STIMULUS  The lab stimulus a HILS rig must reproduce for a run:
%   the geomagnetic field in the body frame (Helmholtz-cage command, T), the
%   Sun direction in the body frame (Sun-simulator pointing), the body rate
%   (air-bearing reference) and the sunlit fraction (lamp on/off), at the
%   recording rate. Write it with asils.hal.write_stimulus for the rig.
    S.t = rec.t; S.B_body_T = rec.B; S.sun_body = rec.sun_body; S.w_body = rec.w; S.lamp = rec.nu;
end
