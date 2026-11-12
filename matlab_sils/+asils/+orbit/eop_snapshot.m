function E = eop_snapshot()
%ASILS.ORBIT.EOP_SNAPSHOT  The design's EOP and leap-second snapshot (env_eop_snapshot, DR-env-002) as the frame build A takes
%   it: the engine's adcs-pop Eop::from_snapshot. finals2000A.all's rows as the design's generated tables give them, each day
%   Bulletin B's pole, UT1 - UTC and celestial pole offsets where the day has them, else Bulletin A's (the offsets from the A
%   table by day, 0 where it has none), dX and dY from milliarcseconds to arcseconds; UT1 carried as UT1 - TAI (UT1 - UTC less
%   the leap-second table's TAI - UTC at the day, env_eop's own leap_at); the leap-second rows and their expiry.
%   Owner: Agastya. Copyright (c) 2026 Agastya. All rights reserved.
    A = asils.models.eopsnapshot.FINALS_A_PM_UT1();     % MJD, pole flag, x ["], y ["], UT1 flag, UT1 - UTC [s]
    N = asils.models.eopsnapshot.FINALS_A_NUT();        % MJD, nutation flag, dX [mas], dY [mas]
    B = asils.models.eopsnapshot.FINALS_B();            % MJD, x ["], y ["], UT1 - UTC [s], dX [mas], dY [mas]
    L = asils.models.eopsnapshot.LEAP_DAT();            % MJD, day, month, year, TAI - UTC [s]
    X = asils.models.eopsnapshot.LEAP_DAT_EXPIRY();     % year, month, day, MJD
    n = size(A, 1);
    mjd = A(:, 1); xp = zeros(n, 1); yp = xp; ut = xp; dx = xp; dy = xp;
    ls_mjd = L(:, 1); ls_dat = L(:, 5);
    for i = 1:n
        ib = find(B(:, 1) == mjd(i), 1);
        if ~isempty(ib)
            xp(i) = B(ib, 2); yp(i) = B(ib, 3); ut(i) = B(ib, 4); dx(i) = B(ib, 5); dy(i) = B(ib, 6);
        else
            xp(i) = A(i, 3); yp(i) = A(i, 4); ut(i) = A(i, 6);
            inut = find(N(:, 1) == mjd(i), 1);
            if ~isempty(inut), dx(i) = N(inut, 3); dy(i) = N(inut, 4); end
        end
    end
    dx = dx/1000; dy = dy/1000;
    for i = 1:n, ut(i) = ut(i) - asils.models.eop.leap_at(ls_mjd, ls_dat, mjd(i)); end
    E = struct('mjd', mjd, 'xp', xp, 'yp', yp, 'dx', dx, 'dy', dy, 'ut', ut, 'ls_mjd', ls_mjd, 'ls_dat', ls_dat, 'leap_expiry', X(4));
end
