function y = bus_on(B, t, dev, kinds)
%ASILS.FAULTS.BUS_ON  Whether a bus fault of one of the kinds is in force at the device at time t (the engine's BusFaults::on).
%   Owner: Agastya. Copyright (c) 2026 Agastya. All rights reserved.
    y = false;
    for i = 1:numel(B.w)
        x = B.w(i);
        if strcmp(x.dev, dev) && t >= x.t0 && t < x.t1 && any(strcmp(x.kind, kinds)), y = true; return, end
    end
end
