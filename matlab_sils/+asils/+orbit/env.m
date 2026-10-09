function E = env(O, t, jd0, r, v, gh, nmax)
%ASILS.ORBIT.ENV  The environment at time t on the POP orbit (r, v): the field in J2000, the Sun and the Moon from the
%   spacecraft, the sunlit fraction, the atmosphere-relative velocity, the density and the Sun's pressure (the engine's
%   run.rs Truth::env). The models are the design's, generated into +asils/+models.
%   E = asils.orbit.env(O, t, P.jd0, r, v, gh, P.env.igrf_nmax)
%   Owner: Agastya. Copyright (c) 2026 Agastya. All rights reserved.
    X = asils.orbit.context(O, t);
    % the field in J2000: precession to the equator of date, then POP's Earth rotation (env's truthfield)
    cp = asils.la.mm(X.C, asils.models.caltime.precession_iau76(jd0 + t/86400.0));
    [lat, lon, hh] = asils.models.geodesy.geodetic_wgs84(asils.la.mv(cp, r));
    E.b_eci = asils.models.truthfield.field_eci_at(lat, lon, hh, cp, gh, nmax);
    E.sun_rel = X.sun_eci - r; E.moon_rel = X.moon_eci - r;
    E.nu = asils.models.shadow.shadow_fraction(r, X.sun_eci);
    E.v_rel = asils.models.orbitfast.corotating_velocity(r, v, O.omega_e);
    E.rho = X.rho; E.p_srp = X.P_srp;
end
