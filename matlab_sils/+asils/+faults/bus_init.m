function B = bus_init(faults)
%ASILS.FAULTS.BUS_INIT  The bus faults of a scenario (bus_nack, bus_crc, bus_stuck, bus_delay, bus_brownout, bus_can_error: O11,
%   docs/S11_INVENTORY.md) as the windows they act in, and what each device last put on its bus: the engine's busfault.rs
%   BusFaults::new, in MATLAB. A window holds the kind, the device (a name of the device map, fsw/devices/example_3u.toml), its start
%   and end [s] (a brownout ends after its value; the others at end_s or never) and the delay [s].
%   Owner: Agastya. Copyright (c) 2026 Agastya. All rights reserved.
    KINDS = {'bus_nack', 'bus_crc', 'bus_stuck', 'bus_delay', 'bus_brownout', 'bus_can_error'};
    W = struct('kind', {}, 'dev', {}, 't0', {}, 't1', {}, 'delay', {});
    for i = 1:numel(faults)
        if iscell(faults), f = faults{i}; else, f = faults(i); end
        if ~any(strcmp(f.kind, KINDS)), continue, end
        v = asils.util.getf(f, 'value', 0); v = v(1);
        t1 = asils.util.getf(f, 'end_s', Inf); if isempty(t1), t1 = Inf; end
        if strcmp(f.kind, 'bus_brownout'), t1 = f.t_s + v; end
        d = 0; if strcmp(f.kind, 'bus_delay'), d = v; end
        W(end+1) = struct('kind', f.kind, 'dev', f.device, 't0', f.t_s, 't1', t1, 'delay', d); %#ok<AGROW>
    end
    B = struct('w', W, 'mag', [], 'sun', [], 'es', [], 'gyro', [], 'st', zeros(1, 0), 'gnss', zeros(1, 0), 'can', zeros(0, 10), ...
               'prev_m', zeros(3, 1), 'prev_r', zeros(8, 1), 'prev_g', zeros(4, 1), 'prev_d', zeros(6, 1));
end
