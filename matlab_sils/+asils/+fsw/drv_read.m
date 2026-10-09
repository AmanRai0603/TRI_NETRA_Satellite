function [z, F, bus] = drv_read(F, bus)
%ASILS.FSW.DRV_READ  A read of every sensor over the bus: the runtime of fsw-rs/src/drv.rs (Drv::read) in MATLAB.
%   The bus transfers, the UART frame assembly that keeps a frame not yet whole for the next read, and the
%   last reading held when a read brings none are the runtime's (code); the decoding is the design's
%   (asils.alg.drivers.drv_read, crc16: fsw/pseudocode/09). bus: asils.hal.bus (the engine's adcs_fsw_abi Bus).
%   Owner: Agastya. Copyright (c) 2026 Agastya. All rights reserved.
    p = F.p; z = F.z;
    % a transfer that fails brings no reading: its status byte is cleared (a part that is not there: no transfer)
    mag = zeros(7, 1); if ~isempty(bus.mag), mag = bus.mag(:); end
    gyro = zeros(13, 1); if p.has_gyro ~= 0 && ~isempty(bus.gyro), gyro = bus.gyro(:); end
    sun = zeros(7, 1); if p.has_sun ~= 0 && ~isempty(bus.sun), sun = bus.sun(:); end
    es = zeros(7, 1); if p.has_es ~= 0 && ~isempty(bus.es), es = bus.es(:); end
    st = zeros(96, 1); st_len = 0; gps = zeros(96, 1); gps_len = 0;
    if p.has_st ~= 0, [st, st_len, F, bus] = uart_frame_(F, bus, 2, 144); end      % port 1, EB 90
    if p.has_gps ~= 0, [gps, gps_len, F, bus] = uart_frame_(F, bus, 3, 145); end   % port 2, EB 91
    [can, bus] = can_batch_(bus);
    n_can = size(can, 1);
    [ids, dlc, data] = can_words_(can, false);
    [mag_ok, b, gyro_ok, w, sun_ok, s, es_ok, nadir, st_ok, st_valid, q_st, gps_ok, r, v, h, delta] = ...
        asils.alg.drivers.drv_read(mag, gyro, sun, es, st, st_len, gps, gps_len, ids, dlc, data, n_can, ...
        p.has_gyro ~= 0, p.has_sun ~= 0, p.has_es ~= 0, p.has_st ~= 0, p.has_gps ~= 0, p.nr, p.rot_gi);
    z.mag_ok = mag_ok; if mag_ok, z.b = b; end
    z.gyro_ok = gyro_ok; if gyro_ok, z.w = w; end
    z.sun_ok = sun_ok; if sun_ok, z.sun = s; end
    z.es_ok = es_ok; if es_ok, z.nadir = nadir; end
    z.st_ok = st_ok; z.st_valid = st_valid;
    if st_len > 0, z.q_st = q_st; end
    z.gps_ok = gps_ok; if gps_ok, z.r = r; z.v = v; end
    % the momentum devices' telemetry: every frame waiting, 8 at a time in the order they came
    z = hold_telemetry_(p, z, can, h, delta);
    while n_can == 8
        [can, bus] = can_batch_(bus);
        n_can = size(can, 1);
        [h, delta] = telemetry_(p, can, false);
        z = hold_telemetry_(p, z, can, h, delta);
    end
end

function [f, n, F, bus] = uart_frame_(F, bus, port, sync2)
% Pull the bytes waiting; the newest complete, CRC-good frame with sync2, as the stream of that one frame
% (EB sync2 n payload CRC) the design's frame search reads. A frame not yet whole stays for the next read.
    RXCAP = 256;
    b = F.rx{port};
    q = bus.uart{port};
    if ~isempty(q)
        bus.uart{port} = zeros(1, 0);
        if numel(b) + numel(q) <= RXCAP
            b = [b; q(:)];
        else                               % a full buffer starts again (the runtime's wrap), byte by byte
            for x = reshape(q, 1, [])
                if numel(b) >= RXCAP, b = zeros(0, 1); end
                b(end + 1, 1) = x; %#ok<AGROW>
            end
        end
    end
    f = zeros(96, 1); n = 0;
    while true
        len = numel(b);
        s = 0;
        while s + 1 < len && ~(b(s + 1) == 235 && b(s + 2) == sync2), s = s + 1; end
        if s + 3 > len, break, end
        k = b(s + 3);
        if s + 3 + k + 2 > len
            if s > 0, b = b(s + 1:end); end
            break
        end
        crc = b(s + 4 + k) + 256*b(s + 5 + k);
        if k <= 64
            pl = zeros(64, 1); pl(1:k) = b(s + 4:s + 3 + k);
            if asils.alg.drivers.crc16(pl, k) == crc
                f = zeros(96, 1); f(1:5 + k) = b(s + 1:s + 5 + k); n = 5 + k;
            end
        end
        b = b(s + 6 + k:end);
    end
    F.rx{port} = b;
end

function [can, bus] = can_batch_(bus)
% The CAN frames waiting, at most 8, in the order they came (rows: id, dlc, data 1 to 8).
    m = min(8, size(bus.can_rx, 1));
    can = bus.can_rx(1:m, :);
    bus.can_rx = bus.can_rx(m + 1:end, :);
end

function [ids, dlc, data] = can_words_(can, invert)
    ids = zeros(8, 1); dlc = zeros(8, 1); data = zeros(8, 8);
    m = size(can, 1);
    if m == 0, return, end
    ids(1:m) = can(:, 1); dlc(1:m) = can(:, 2);
    if invert, data(1:m, :) = 255 - can(:, 3:10); else, data(1:m, :) = can(:, 3:10); end
end

function [h, delta] = telemetry_(p, can, invert)
% The momentum and gimbal angle the design's read decodes from CAN frames alone.
    [ids, dlc, data] = can_words_(can, invert);
    [~, ~, ~, ~, ~, ~, ~, ~, ~, ~, ~, ~, ~, ~, h, delta] = asils.alg.drivers.drv_read(zeros(7, 1), zeros(13, 1), zeros(7, 1), ...
        zeros(7, 1), zeros(96, 1), 0, zeros(96, 1), 0, ids, dlc, data, size(can, 1), false, false, false, false, false, p.nr, p.rot_gi);
end

function z = hold_telemetry_(p, z, can, h, delta)
% The values the frames carried replace the held ones; the rest are held. Which they carried: the same frames with
% every data byte inverted decode each carried value to another, so a carried value is not 0 in both reads.
    if isempty(can), return, end
    [hi, di] = telemetry_(p, can, true);
    for i = 1:8, if h(i) ~= 0 || hi(i) ~= 0, z.h(i) = h(i); end, end
    for j = 1:4, if delta(j) ~= 0 || di(j) ~= 0, z.delta(j) = delta(j); end, end
end
