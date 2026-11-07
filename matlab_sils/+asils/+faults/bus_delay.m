function d = bus_delay(B, t)
%ASILS.FAULTS.BUS_DELAY  The delay the answers carry at t [s]: the longest of the bus_delay faults in force (the engine's delay_s).
%   Owner: Agastya. Copyright (c) 2026 Agastya. All rights reserved.
    d = 0;
    for i = 1:numel(B.w)
        x = B.w(i);
        if strcmp(x.kind, 'bus_delay') && t >= x.t0 && t < x.t1, d = max(d, x.delay); end
    end
end
