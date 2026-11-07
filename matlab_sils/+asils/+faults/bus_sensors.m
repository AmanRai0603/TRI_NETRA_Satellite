function [B, bus] = bus_sensors(B, t, bus, m)
%ASILS.FAULTS.BUS_SENSORS  Before the flight software's step: what the devices put on the buses this tick (written after the mark
%   m) as the bus faults leave it: a device that does not answer (bus_nack, bus_brownout), a register, frame or telemetry that stops
%   changing (bus_stuck: the last good one again), a UART frame with a wrong CRC (bus_crc: the last byte), the CAN controller's error
%   frame (bus_can_error: nothing received). The engine's BusFaults::sensors, in MATLAB, on the same bytes.
%   Owner: Agastya. Copyright (c) 2026 Agastya. All rights reserved.
    if isempty(B.w), return, end
    on = @(dev, kinds) asils.faults.bus_on(B, t, dev, kinds);
    dead = @(dev) on(dev, {'bus_nack', 'bus_brownout'});
    stuck = @(dev) on(dev, {'bus_stuck'});
    [bus.mag, B.mag] = block_(bus.mag, B.mag, dead('magnetometer'), stuck('magnetometer'));
    [bus.sun, B.sun] = block_(bus.sun, B.sun, dead('sun_sensor_plus_x'), stuck('sun_sensor_plus_x'));
    [bus.es, B.es] = block_(bus.es, B.es, dead('earth_sensor'), stuck('earth_sensor'));
    [bus.gyro, B.gyro] = block_(bus.gyro, B.gyro, dead('gyro'), stuck('gyro'));
    % the UART frames written this tick: silent, with a bad CRC (the frame's last byte), or the last one again
    [bus.uart{2}, B.st] = frame_(bus.uart{2}, B.st, m.st, dead('star_tracker'), on('star_tracker', {'bus_crc'}), stuck('star_tracker'));
    [bus.uart{3}, B.gnss] = frame_(bus.uart{3}, B.gnss, m.gnss, dead('gnss'), on('gnss', {'bus_crc'}), stuck('gnss'));
    % the rotors' telemetry frames: lost (no answer, an error frame) or the last of each identifier again
    cerr = on('momentum_devices', {'bus_can_error'}); cdead = dead('momentum_devices'); cstuck = stuck('momentum_devices');
    if cdead || cerr
        bus.can_rx = bus.can_rx(1:m.can, :);
    else
        for i = m.can + 1:size(bus.can_rx, 1)
            f = bus.can_rx(i, :);
            j = find(B.can(:, 1) == f(1), 1);
            if isempty(j)
                B.can(end+1, :) = f; %#ok<AGROW>
            elseif cstuck
                bus.can_rx(i, 3:10) = B.can(j, 3:10);
            else
                B.can(j, :) = f;
            end
        end
    end
end

function [blk, last] = block_(blk, last, dead, stuck)
    if dead
        blk = [];
    elseif stuck
        if isempty(last), last = blk; end
        blk = last;
    else
        last = blk;
    end
end

function [q, last] = frame_(q, last, from, dead, crc, stuck)
    if numel(q) <= from, return, end
    if dead, q = q(1:from); return, end
    if crc, q(end) = bitxor(q(end), 1); end
    if stuck
        if isempty(last), last = q(from + 1:end); end
        q = [q(1:from), last];
    else
        last = q(from + 1:end);
    end
end
