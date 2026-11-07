function m = bus_mark(bus)
%ASILS.FAULTS.BUS_MARK  Where the bytes of a tick start in the bus's queues, before the emulators write their frames (the engine's
%   BusFaults::mark): the star tracker's and the GNSS receiver's UART, and the CAN frames waiting.
%   Owner: Agastya. Copyright (c) 2026 Agastya. All rights reserved.
    m = struct('st', numel(bus.uart{2}), 'gnss', numel(bus.uart{3}), 'can', size(bus.can_rx, 1));
end
