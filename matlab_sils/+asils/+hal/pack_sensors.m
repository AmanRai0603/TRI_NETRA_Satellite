function b = pack_sensors(z, L)
%ASILS.HAL.PACK_SENSORS  Sensor frame -> bytes (little-endian), the OILS/HILS
%   link layout: [gyro i32x3][mag i16x3][sun i16x3][flags u8][st q i32x4]
%   [gps pos i32x3][gps vel i32x3][rotors i32 x nr][gimbals i32 x ng]
    i32 = @(x) typecast(int32(x(:)'), 'uint8');
    i16 = @(x) typecast(int16(x(:)'), 'uint8');
    fl = uint8(z.sun_ok + 2*z.st_ok + 4*z.gps_ok + 8*z.clean);
    q = z.q_st(:, 1); r = [0;0;0]; v = [0;0;0];
    if z.gps_ok, r = z.r_gps; v = z.v_gps; end
    b = [i32(z.w/L.gyro), i16(z.B/L.mag), i16(z.sun/L.sun), fl, i32(q/L.q), ...
         i32(r/L.pos), i32(v/L.vel), i32(z.h/L.h), i32(z.delta/L.delta)];
end
