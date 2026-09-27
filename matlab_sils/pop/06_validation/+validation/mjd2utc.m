function u = mjd2utc(mjd)
%VALIDATION.MJD2UTC  Modified Julian Date -> UTC calendar vector [Y Mo D H Mi S].
%   u = validation.mjd2utc(mjd)
%
%   MJD 0 = 1858-11-17 00:00 = datenum 678942, so datenum = mjd + 678942.
%
%   PRECISION NOTE. For a 2007 epoch datenum ~ 7.33e5, where eps(datenum) is
%   ~1.2e-10 d ~ 1e-5 s. So this is good to ~10 microseconds -- irrelevant against
%   a 10 s product sampling, but do not build a clock out of it. Anything needing
%   sub-microsecond time should go through timeconv (two-part JD), not through here.
%
%   This is deliberately NOT leap-second aware: ITSG epochs are already UTC, and the
%   engine's UTC->TT/TAI chain lives in timeconv. This only reformats.
    dn = mjd + 678942;
    v  = datevec(dn);
    u  = v(1:6);
end
