function E = env(O, t, jd0, r, v, gh, nmax)
%ASILS.ORBIT.ENV  The environment at time t on the POP orbit (r, v): the field in J2000, the Sun and the Moon from the
%   spacecraft, the sunlit fraction, the atmosphere-relative velocity, the density and the Sun's pressure (the engine's
%   run.rs Truth::env). The models are the design's, generated into +asils/+models.
%   E = asils.orbit.env(O, t, P.jd0, r, v, gh, P.env.igrf_nmax)
%   Owner: Agastya. Copyright (c) 2026 Agastya. All rights reserved.
    X = asils.orbit.context(O, t);
    % the field in J2000, through POP's Earth frame: on build A C is already GCRF to ITRF, the precession in it; only the Gmst
    % build turns about the pole of J2000 and needs the precession put in front of it (env's truthfield)
    cp = X.C;
    if strcmp(O.W.build, 'Gmst'), cp = asils.la.mm(X.C, asils.models.caltime.precession_iau76(jd0 + t/86400.0)); end
    [lat, lon, hh] = asils.models.geodesy.geodetic_wgs84(asils.la.mv(cp, r));
    E.b_eci = asils.models.truthfield.field_eci_at(lat, lon, hh, cp, gh, nmax);
    E.sun_rel = X.sun_eci - r; E.moon_rel = X.moon_eci - r;
    E.nu = asils.models.shadow.shadow_fraction(r, X.sun_eci);
    E.v_rel = asils.models.orbitfast.corotating_velocity(r, v, O.omega_e);
    E.rho = X.rho; E.p_srp = X.P_srp;
    E.v_earth = X.v_earth;          % the geocentre's velocity from the solar-system barycentre [m/s]: the star tracker's aberration adds it to the spacecraft's
end
