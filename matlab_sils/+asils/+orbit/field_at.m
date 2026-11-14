function b = field_at(O, X, t, jd0, r, gh, nmax)
%ASILS.ORBIT.FIELD_AT  The field in J2000 [T] at position r and time t through POP's Earth frame X.C (the orbit's context at t).
%   On build A, X.C is already GCRF to ITRF, the precession in it; only the Gmst build turns about the pole of J2000 and needs the
%   precession put in front of it (env's truthfield; the engine's run.rs pop_field).
%   Owner: Agastya. Copyright (c) 2026 Agastya. All rights reserved.
    cp = X.C;
    if strcmp(O.W.build, 'Gmst'), cp = asils.la.mm(X.C, asils.models.caltime.precession_iau76(jd0 + t/86400.0)); end
    [lat, lon, hh] = asils.models.geodesy.geodetic_wgs84(asils.la.mv(cp, r));
    b = asils.models.truthfield.field_eci_at(lat, lon, hh, cp, gh, nmax);
end
