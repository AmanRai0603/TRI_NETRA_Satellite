function f = frame_record(O, t_end)
%ASILS.ORBIT.FRAME_RECORD  What a run records of the Earth frame and of the data's validity at the end of its span (the
%   engine's run.rs Truth::frame_record; the manifest's `frame`): the frame build, the leap-second table's status (1 when
%   the end is after the day its file expires, timescales.convert_utc's leap_status) and the EOP table's extrapolation flag
%   (0 for a build with no EOP table). A run past the data's validity warns and records; it is not refused (DR-env-002).
%   Owner: Agastya. Copyright (c) 2026 Agastya. All rights reserved.
    W = O.W;
    u = asils.models.timescales.addsec(W.epoch, t_end);
    T = asils.models.timescales.convert_utc(u(1), u(2), u(3), u(4), u(5), u(6), 0);
    f = struct('build', W.build, 'eop_flag', 0, 'leap_status', T.leap_status, 'end_mjd_utc', T.utc_mjd);
end
