function z = sensors(H, z, t)
%ASILS.HAL.SENSORS  Sensor frames across the FSW boundary (drivers' view).
    L = H.lsb;
    Q = @(x, l) round(x/l)*l;
    z.w = Q(z.w, L.gyro);
    z.B = Q(z.B, L.mag);
    z.sun = Q(z.sun, L.sun);
    if z.st_ok, z.q_st = Q(z.q_st, L.q); end
    if z.gps_ok, z.r_gps = Q(z.r_gps, L.pos); z.v_gps = Q(z.v_gps, L.vel); end
    z.h = Q(z.h, L.h); z.delta = Q(z.delta, L.delta);
    switch H.backend
        case 'loopback'
            z = asils.hal.unpack_sensors(asils.hal.pack_sensors(z, L), z, L);
        case 'udp'
            write(H.sock, asils.hal.pack_sensors(z, L), 'uint8', H.host, H.port_tx);
    end
end
