function E = ephem(jd_tdb)
%ASILS.ORBIT.EPHEM  The force model's ephemeris at a TDB Julian date (POP's ephemInputs, the engine's adcs-pop
%   ephem::inputs): the four DE440 segments env's slice holds (de440.de440_segment: the Sun and the Earth-Moon
%   barycentre from the solar-system barycentre, the Earth and the Moon from the Earth-Moon barycentre), made
%   geocentric (de440.geocentric), and what the forces read of them (de440.ephem_inputs).
%   Owner: Agastya. Copyright (c) 2026 Agastya. All rights reserved.
    et = asils.models.de440.jd_to_et(jd_tdb);
    p = zeros(3, 4); v = zeros(3, 4);
    for s = 0:3                         % the slice's segments in its order: Sun, Earth-Moon barycentre, Earth, Moon
        [ok, ps, vs] = asils.models.de440.de440_segment(s, et);
        if ~ok
            error('asils:orbit', 'DE440: TDB JD %.6f is outside the design''s slice of DE440 (env_de440_slice)', jd_tdb);
        end
        p(:, s + 1) = ps; v(:, s + 1) = vs;
    end
    [rs, vs, rm, vm] = asils.models.de440.geocentric(p(:, 1), v(:, 1), p(:, 2), v(:, 2), p(:, 3), v(:, 3), p(:, 4), v(:, 4));
    E = asils.models.de440.ephem_inputs(jd_tdb, rs, vs, rm, vm);
end
