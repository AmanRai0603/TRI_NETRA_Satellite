function z = unpack_sensors(b, z, L)
%ASILS.HAL.UNPACK_SENSORS  Bytes -> sensor frame (inverse of pack_sensors).
    p = 1;
    [v, p] = rd_(b, p, 'int32', 3); z.w = double(v)*L.gyro;
    [v, p] = rd_(b, p, 'int16', 3); z.B = double(v)*L.mag;
    [v, p] = rd_(b, p, 'int16', 3); z.sun = double(v)*L.sun;
    fl = b(p); p = p + 1;
    z.sun_ok = bitand(fl, 1) > 0; z.st_ok = bitand(fl, 2) > 0; z.gps_ok = bitand(fl, 4) > 0; z.clean = bitand(fl, 8) > 0;
    [v, p] = rd_(b, p, 'int32', 4); q = double(v)*L.q;
    if z.st_ok, z.q_st(:, 1) = q; end
    [v, p] = rd_(b, p, 'int32', 3); r = double(v)*L.pos;
    [v, p] = rd_(b, p, 'int32', 3); vv = double(v)*L.vel;
    if z.gps_ok, z.r_gps = r; z.v_gps = vv; end
    [v, p] = rd_(b, p, 'int32', numel(z.h)); z.h = double(v)*L.h;
    [v, ~] = rd_(b, p, 'int32', numel(z.delta)); z.delta = double(v)*L.delta;
end
function [v, p] = rd_(b, p, cls, n)
    w = 4; if strcmp(cls, 'int16'), w = 2; end
    if n == 0, v = zeros(0,1); return, end
    v = typecast(uint8(b(p:p+w*n-1)), cls)'; p = p + w*n;
end
