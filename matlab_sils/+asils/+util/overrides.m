function [sc, eng, hal] = overrides(s)
%ASILS.UTIL.OVERRIDES  A run's overrides (a struct, '__' for '.') as the engine's words: the scenario's paths
%   ({path, value} rows: fsw.rw_bandwidth, initial.rate.magnitude_deg_s, time.dt_s, ...) and the engine's settings
%   (engine.<name>). The twin's older words are read as the engine's: sim.duration_s -> engine.duration_s,
%   env.F107/F107a/Kp/ap -> engine.f107/f107a/kp/ap, sc.mass_kg/refl/m_res/cm_offset_m -> engine.<the same>,
%   sc.sigma_n -> engine.accommodation, orbit.ltan_h/alt_km -> engine.ltan_h/alt_km, orbit.u0_deg -> initial.arg_lat_deg,
%   scenario.<path> -> <path>. A word the engine does not read is refused when the engine is asked (adcs params).
%   hal.<name> (backend, realtime, stimulus, host, port_tx, port_rx) is the twin's own boundary (asils.hal.open): it
%   stays with the twin (hal, a struct) and never reaches the engine.
%   Owner: Agastya. Copyright (c) 2026 Agastya. All rights reserved.
    sc = cell(0, 2); eng = cell(0, 2); hal = struct();
    if isempty(s), return, end
    legacy = {'sim.duration_s', 'engine.duration_s'; 'env.F107', 'engine.f107'; 'env.F107a', 'engine.f107a'; 'env.Kp', 'engine.kp';
              'env.ap', 'engine.ap'; 'sc.mass_kg', 'engine.mass_kg'; 'sc.refl', 'engine.refl'; 'sc.m_res', 'engine.m_res';
              'sc.cm_offset_m', 'engine.cm_offset_m'; 'sc.sigma_n', 'engine.accommodation'; 'sc.sigma_t', '';
              'orbit.ltan_h', 'engine.ltan_h'; 'orbit.alt_km', 'engine.alt_km'; 'orbit.period_s', ''; 'orbit.u0_deg', 'initial.arg_lat_deg';
              'sim.dt', 'time.dt_s'; 'sim.record_dt', 'time.record_dt_s'};
    f = fieldnames(s);
    for i = 1:numel(f)
        k = strrep(f{i}, '__', '.'); x = s.(f{i});
        j = find(strcmp(legacy(:, 1), k), 1);
        if ~isempty(j)
            k = legacy{j, 2};
            if isempty(k), continue, end            % carried by another word (sigma_t with sigma_n, the period with the altitude)
        end
        if strncmp(k, 'scenario.', 9), k = k(10:end); end
        if strncmp(k, 'hal.', 4), hal.(k(5:end)) = x; continue, end
        if strncmp(k, 'engine.', 7), eng(end+1, :) = {k, x}; else, sc(end+1, :) = {k, x}; end %#ok<AGROW>
    end
end
