function b = budget(fill, lookup)
%ASILS.SIZING.BUDGET  The ADCS's mass / power / volume per fill (design/sizebudget.pc: sizebudget's budget_line and
%   budget_total; the engine's adcs-design budget). fill: a cell of the product's fill entries; lookup: a part number ->
%   its part record.
%   Owner: Agastya. Copyright (c) 2026 Agastya. All rights reserved.
    n = numel(fill);
    items = cell(1, n); ms = zeros(n, 1); ps = ms; vs = ms;
    for i = 1:n
        f = fill{i};
        pt = lookup(f.part);
        nm = struct(); if isfield(pt, 'nominal'), nm = pt.nominal; end
        [nn, m, p, v] = asils.models.sizebudget.budget_line(slot_of_(f.slot), count_(f, 'axes_body'), count_(f, 'spin_axes_body'), ...
            count_(f, 'boresights_body'), count_(f, 'normals_body'), num_(nm, 'mass_kg'), num_(nm, 'power_steady_W'), ...
            num_(nm, 'power_W'), num_(nm, 'power_at_max_W'), num_(nm, 'volume_L'));
        items{i} = struct('slot', f.slot, 'part', f.part, 'n', nn, 'mass_kg', m, 'power_W', p, 'volume_L', v);
        ms(i) = m; ps(i) = p; vs(i) = v;
    end
    [m, p, v] = asils.models.sizebudget.budget_total(ms, ps, vs, n);
    b = struct('mass_kg', m, 'power_W', p, 'volume_L', v, 'items', {items});
end

function s = slot_of_(w)
% a fill slot's word (adcs-product/1) as sizebudget's Slot
    S = {'coils', 'wheels', 'rings', 'cmg', 'vscmg', 'rcs', 'star_tracker', 'magnetometer', 'sun_sensors', 'gyro', 'gnss', ...
         'earth_sensor', 'coarse_sun_sensors'};
    s = find(strcmp(w, S), 1) - 1;
    if isempty(s), s = 13; end
end
function n = count_(f, key)
% the entries of a fill's list of directions (a cell of rows), -1 when it has none
    n = -1;
    if isfield(f, key)
        x = f.(key);
        if iscell(x), n = numel(x); elseif isnumeric(x) && ~isempty(x), n = size(x, 1); end
    end
end
function x = num_(s, k)
    x = NaN;
    if isfield(s, k) && isnumeric(s.(k)) && isscalar(s.(k)), x = double(s.(k)); end
end
