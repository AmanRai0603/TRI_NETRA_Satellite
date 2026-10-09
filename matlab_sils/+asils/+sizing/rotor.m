function p = rotor(Dm, k, which)
%ASILS.SIZING.ROTOR  The select_rotor node (docs/NODES.md; design/sizerotor.pc): the benchmarks' momentum actuators are
%   bought, so they are chosen from the datasheet catalogue (data/catalogue, tools/catalogue.py), never sized by a law.
%   which: 'rw' (three orthogonal wheels), 'cmg' or 'vscmg' (a four-unit pyramid). sizerotor's rotor_need, rotor_fits,
%   rotor_pick, rotor_meets, rotor_vscmg, rotor_dispersion; the engine's adcs-design rotor.
%   p = asils.sizing.rotor(Dm, asils.sizing.knobs(), 'rw')
%   Owner: Agastya. Copyright (c) 2026 Agastya. All rights reserved.
    switch which                      % sizerotor's RotorUse: 0 rw, 1 cmg, 2 vscmg
        case 'rw', u = 0;
        case 'cmg', u = 1;
        otherwise, u = 2;
    end
    s = asils.sizing.scale_(k, which);
    [h_need, tau_need, units] = asils.models.sizerotor.rotor_need(u, Dm.h_req, Dm.tau_req, s);
    d = fullfile(asils.util.root(), 'data', 'catalogue');
    L = dir(fullfile(d, '*')); L = L(~[L.isdir]);
    names = sort({L.name});           % the engine's order: the file names, byte by byte
    assert(~isempty(names), 'asils:sizing:refused', 'catalogue: %s holds no model', d);
    cands = {};
    for i = 1:numel(names)
        c = asils.util.readjson(fullfile(d, names{i}));
        switch getf_(c, 'type', '')   % sizerotor's CatalogueKind: 0 reaction wheel, 1 cmg, 2 cmg cluster, 3 other
            case 'reaction_wheel', kind = 0;
            case 'cmg', kind = 1;
            case 'cmg_cluster', kind = 2;
            otherwise, kind = 3;
        end
        sel = getf_(c, 'selectable', false); if ~islogical(sel), sel = false; end
        if ~asils.models.sizerotor.rotor_fits(u, kind, sel), continue; end
        cands{end+1} = c; %#ok<AGROW>
    end
    assert(~isempty(cands), 'asils:sizing:refused', 'catalogue: no selectable %s model in %s', which, d);
    n = numel(cands);
    col = @(key) cellfun(@(c) der_(c, key), cands(:));
    [pick, met] = asils.models.sizerotor.rotor_pick(col('h_max_Nms'), col('torque_max_Nm'), col('mass_kg'), col('power_steady_W'), col('volume_L'), ...
        n, h_need, tau_need);
    pk = cands{pick + 1};
    if met
        gap = [];
    else
        gap = sprintf('no catalogue %s meets h %.3e N m s, tau %.3e N m per unit; the largest is fitted', which, h_need, tau_need);
    end
    nm = pk.derived;
    switch which
        case 'rw', kind = 'reaction_wheel'; suffix = '';
        case 'cmg', kind = 'cmg'; suffix = '';
        otherwise, kind = 'vscmg'; suffix = '-VSCMG';
    end
    if asils.models.sizerotor.rotor_vscmg(u), nm.rotor_momentum_Nms = pk.derived.vscmg_rotor_momentum_Nms; end
    p = struct('part_number', [pk.part_number suffix], 'kind', kind, ...
        'name', sprintf('%s %s (%s, one of %d)', pk.vendor, pk.model, upper(which), units), 'status', 'catalogue', ...
        'source', {getf_(pk, 'source_url', [])}, 'made', 'bought', 'descriptor_version', 1, 'vendor', pk.vendor, 'model', pk.model, ...
        'verification', {getf_(pk, 'verification', [])}, 'assumptions', {getf_(pk, 'assumptions', [])});
    p.nominal = nm;
    [td, tm, ts, fd, flo, fhi, md, mm, ms] = asils.models.sizerotor.rotor_dispersion();
    p.dispersion = struct('torque_scale', asils.sizing.spread_(td, tm, ts), 'friction_scale', asils.sizing.spread_(fd, flo, fhi), ...
        'axis_misalignment_rad', asils.sizing.spread_(md, mm, ms));
    cl = cell(1, n);
    for i = 1:n
        c = cands{i};
        cl{i} = struct('part_number', c.part_number, 'vendor', c.vendor, 'model', c.model, 'h_Nms', der_(c, 'h_max_Nms'), ...
            'tau_Nm', der_(c, 'torque_max_Nm'), 'mass_kg', der_(c, 'mass_kg'), 'power_W', der_(c, 'power_steady_W'), ...
            'meets', asils.models.sizerotor.rotor_meets(der_(c, 'h_max_Nms'), der_(c, 'torque_max_Nm'), h_need, tau_need));
    end
    p.sizing = struct('node', 'select_rotor', 'h_req_Nms', Dm.h_req, 'tau_req_Nm', Dm.tau_req, 'scale', s, 'units', units, ...
        'need_per_unit', struct('h_Nms', h_need, 'tau_Nm', tau_need), 'gap', gap, ...
        'rule', 'lightest selectable catalogue model meeting the per-unit need (then steady power, volume)', 'candidates', {cl});
end

function x = der_(c, key)
    x = NaN;
    if isfield(c, 'derived') && isfield(c.derived, key) && isnumeric(c.derived.(key)) && ~isempty(c.derived.(key))
        x = double(c.derived.(key));
    end
end
function x = getf_(s, k, d)
    if isfield(s, k), x = s.(k); else, x = d; end
end
