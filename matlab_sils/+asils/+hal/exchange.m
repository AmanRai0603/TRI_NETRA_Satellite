function [F, bus, H] = exchange(H, F, bus)
%ASILS.HAL.EXCHANGE  One tick across the flight-software boundary: the bus's sensor bytes to the flight software, its
%   command bytes back. 'sils': the twin's flight software steps on the bus (asils.fsw.step). 'loopback': the same, each
%   side packed into its link frame and unpacked again first (asils.hal.link). 'udp': the sensor frame goes to the OBC,
%   whose command frame (the last one waiting) comes back; the twin's flight software does not run.
%   Owner: Agastya. Copyright (c) 2026 Agastya. All rights reserved.
    switch H.backend
        case 'sils'
            [F, bus] = asils.fsw.step(F, bus);
        case 'loopback'
            b = asils.hal.link('pack_sensors', bus);
            bus = asils.hal.link('unpack_sensors', b, bus);
            [F, bus] = asils.fsw.step(F, bus);
            c = asils.hal.link('pack_commands', bus);
            bus = asils.hal.link('unpack_commands', c, bus);
            H.bytes = H.bytes + numel(b) + numel(c);
        case 'udp'
            b = asils.hal.link('pack_sensors', bus);
            write(H.sock, b, 'uint8', H.host, H.port_tx);
            bus.uart{2} = zeros(1, 0); bus.uart{3} = zeros(1, 0); bus.can_rx = zeros(0, 10);
            if H.sock.NumBytesAvailable > 0
                bus = asils.hal.link('unpack_commands', read(H.sock, H.sock.NumBytesAvailable, 'uint8'), bus);
            end
            H.bytes = H.bytes + numel(b);
    end
end
