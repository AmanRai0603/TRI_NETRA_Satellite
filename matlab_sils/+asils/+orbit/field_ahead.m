function b = field_ahead(O, t, jd0, gh, nmax)
%ASILS.ORBIT.FIELD_AHEAD  The field in J2000 [T] at time t, ahead of the orbit and without moving it (S12.7a, E8: the field at the
%   next refresh, which the field between two refreshes is interpolated to). The position is the current step's cubic Hermite and
%   the Earth frame the context's, both taken past the step's end when t is (the engine's InLoop::peek, run.rs Truth::field_ahead):
%   O is not advanced (O is a value; nothing is returned of it), so the next asils.orbit.state finds the nodes it would have found.
%   b = asils.orbit.field_ahead(O, t, P.jd0, gh, P.env.igrf_nmax)
%   Owner: Agastya. Copyright (c) 2026 Agastya. All rights reserved.
    h = O.t1 - O.t0; s = (t - O.t0)/h;
    s2 = s*s; s3 = s2*s;
    h00 = 2*s3 - 3*s2 + 1; h10 = s3 - 2*s2 + s; h01 = -2*s3 + 3*s2; h11 = s3 - s2;
    r = h00*O.y0(1:3) + h10*h*O.y0(4:6) + h01*O.y1(1:3) + h11*h*O.y1(4:6);
    b = asils.orbit.field_at(O, asils.orbit.context(O, t), t, jd0, r, gh, nmax);
end
