function [C, info] = eci2ecef(W, utc)
%ASILS.ORBIT.ECI2ECEF  The Earth frame of the in-loop orbit at a UTC instant utc = [Y Mo D H Mi S] (r_ecef = C r_eci): the
%   engine's adcs-pop frames::eci2ecef on the world's build W.build (env_earth_frame_build). 'A': the IAU 2006/2000A CIO chain
%   (env_iau2006's cio_c2t) with the EOP of the snapshot linearly interpolated (env_eop's eop_linear, no tidal terms), GCRF
%   to ITRF; 'Gmst': IAU 1982 GMST about the pole of J2000 (env_earth_frames' eci2ecef_gmst, dUT1 = 0, no polar motion).
%   info: eop_flag (1 when the EOP table is extrapolated) and leap_status (1 after the leap-second table's expiry).
%   Owner: Agastya. Copyright (c) 2026 Agastya. All rights reserved.
    if strcmp(W.build, 'Gmst')
        T = asils.models.timescales.convert_utc(utc(1), utc(2), utc(3), utc(4), utc(5), utc(6), 0);
        C = asils.models.earthframes.eci2ecef_gmst(T.gmst_rad, 0, 0);
        info = struct('eop_flag', 0, 'leap_status', T.leap_status);
    elseif strcmp(W.build, 'A')
        E = W.eop;
        mjd = asils.models.iau2006.mjd_of(utc);
        [dut1, xp, yp, dx, dy, dat, flag] = asils.models.eop.eop_linear(E.mjd, E.xp, E.yp, E.dx, E.dy, E.ut, E.ls_mjd, E.ls_dat, mjd, false);
        C = asils.models.iau2006.cio_c2t(utc, dut1, xp, yp, dx, dy, dat);
        info = struct('eop_flag', flag, 'leap_status', double(mjd > E.leap_expiry));
    else
        error('asils:orbit', 'the Earth frame build %s: the twin flies Gmst or A', W.build);
    end
end
