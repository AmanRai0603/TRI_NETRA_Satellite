function Z = size_all(caseId, opts)
%ASILS.SIZING.SIZE_ALL  Every actuator option sized to a case and one product per family (data/families.json), with
%   its mass / power / volume budget: the engine's adcs-design size_all (`adcs size <case>`), in MATLAB. Writes
%     <out>/parts/<id>.json      the sized parts (and the graded gyro when the knobs ask for one)
%     <out>/products/<id>.json   SZ-<case>-<family>
%     <out>/sizing.json          demand, knobs, parts, per-family budgets (adcs-sizing/1)
%   with <out> = store/sized/<case> (where asils.product.load and the engine's product::find look), or opts.out.
%   Z = asils.sizing.size_all('ais_3u')
%   Z = asils.sizing.size_all('ais_3u', struct('knobs', struct('k_tau', 2), 'out', tempname(), 'quiet', true))
%   opts.demand: the case's demand already surveyed with the same knobs (asils.sizing.demand), not flown again.
%
%   Every law is the design's (docs/S7_INVENTORY.md S7.15), generated into +asils/+models: the demand
%   (asils.sizing.demand: sizedemand), the coils (sizemtq), the catalogue's rotors (sizerotor), the fluid loop and its
%   pump (sizefmr, sizepump, sizering), the thrusters (sizercs), the sensors a product carries and how every unit is
%   mounted (sizesensors, sizemtq, sizerotor, sizefmr), the budget (sizebudget). Here only the steps, the catalogue, the
%   parts and the files.
%   Owner: Agastya. Copyright (c) 2026 Agastya. All rights reserved.
    if nargin < 2, opts = struct(); end
    R = asils.util.root();
    k = asils.sizing.knobs(asils.util.getf(opts, 'knobs', struct()));
    Dm = asils.util.getf(opts, 'demand', []);        % a survey already flown for this case and these knobs
    if isempty(Dm), Dm = asils.sizing.demand(fullfile(R, 'cases', [caseId '.csv']), k); end
    cs = Dm.case; bx = Dm.box_m(:); fine = strcmp(Dm.class, 'fine');
    out = asils.util.getf(opts, 'out', fullfile(R, 'store', 'sized', caseId));
    [pm, pmp] = asils.sizing.mtq(Dm, k);
    fm = asils.sizing.fmr(Dm, k, bx);
    keys = {'mtq', 'mtqp', 'rw', 'cmg', 'vscmg', 'fmr_x', 'fmr_y', 'fmr_z', 'rcs'};
    vals = {pm, pmp, asils.sizing.rotor(Dm, k, 'rw'), asils.sizing.rotor(Dm, k, 'cmg'), asils.sizing.rotor(Dm, k, 'vscmg'), ...
            fm{1}, fm{2}, fm{3}, asils.sizing.rcs(Dm, k, bx)};
    if numel(fm) > 3, keys{end+1} = 'fmr_s'; vals{end+1} = fm{4}; end
    for d = {'parts', 'products'}, if ~exist(fullfile(out, d{1}), 'dir'), mkdir(fullfile(out, d{1})); end, end
    by_pn = containers.Map();
    parts = struct();
    for i = 1:numel(keys)
        p = vals{i};
        write_(fullfile(out, 'parts', [p.part_number '.json']), p);
        by_pn(p.part_number) = p;
        parts.(keys{i}) = p;
    end
    pn = @(key) parts.(key).part_number;
    % a better gyro than the catalogue's precision unit when the loop asks for it (design/sizesensors.pc)
    gyro_id = 'TRN-GYRO-P1';
    if asils.models.sizesensors.gyro_graded(fine, k.gyro_grade)
        g = asils.util.readjson(asils.product.find(R, 'parts', 'TRN-GYRO-P1'));
        gr = k.gyro_grade;
        for key = {'arw_rad_per_sqrt_s', 'rrw_rad_per_s_sqrt_s', 'bias_instability_rad_s'}
            if isfield(g.nominal, key{1}) && isnumeric(g.nominal.(key{1})), g.nominal.(key{1}) = asils.models.sizesensors.gyro_noise(g.nominal.(key{1}), gr); end
        end
        for key = {'mass_kg', 'power_W'}
            if isfield(g.nominal, key{1}) && isnumeric(g.nominal.(key{1})), g.nominal.(key{1}) = asils.models.sizesensors.gyro_load(g.nominal.(key{1}), gr); end
        end
        gyro_id = sprintf('SZ-%s-GYRO', cs);
        g.part_number = gyro_id; g.status = 'sized'; g.source = 'asils.sizing (gyro grade)';
        g.name = sprintf('Gyro, noise x%g of TRN-GYRO-P1 (fibre-optic class) — %s', gr, cs);
        write_(fullfile(out, 'parts', [gyro_id '.json']), g);
        by_pn(gyro_id) = g;
    end
    lookup = @(id) lookup_(by_pn, R, id);
    Fam = asils.util.readjson(fullfile(R, 'data', 'families.json'));
    fams = Fam.family; if ~iscell(fams), fams = num2cell(fams); end
    % how each kind of unit is mounted and which sensors a product carries
    coil_axes = rows_(asils.models.sizemtq.mtq_axes());
    [wheel_axes, gimbal_axes, spin_axes] = asils.models.sizerotor.rotor_mounting();
    ring_axes = asils.models.sizefmr.fmr_axes();
    st_fit = asils.models.sizesensors.sensor_star_tracker(fine, k.star_tracker);
    [n_heads, heads, residual] = asils.models.sizesensors.sensor_heads(fine, k.st_heads);
    bs = asils.models.sizesensors.sensor_boresight(fine).';
    [normals, sun_axis] = asils.models.sizesensors.sensor_sun();
    families = struct();
    for i = 1:numel(fams)
        fa = fams{i}; id = fa.id;
        acts = fa.actuators; if ~iscell(acts), acts = cellstr(acts); end
        algs = {'bdot', 'mekf'};
        if isequal(acts(:)', {'mtq'}), coil = pn('mtqp'); else, coil = pn('mtq'); end
        fill = {struct('slot', 'coils', 'part', coil, 'axes_body', {coil_axes})};
        for a = acts(:)'
            switch a{1}
                case 'rw', fill{end+1} = struct('slot', 'wheels', 'part', pn('rw'), 'axes_body', {rows_(wheel_axes)}); %#ok<AGROW>
                case 'fmr'
                    rk = {'fmr_x', 'fmr_y', 'fmr_z'};
                    for j = 1:3, fill{end+1} = struct('slot', 'rings', 'part', pn(rk{j}), 'axes_body', {{ring_axes(j, :)}}); end %#ok<AGROW>
                    if k.fmr_spare
                        fill{end+1} = struct('slot', 'rings', 'part', pn('fmr_s'), 'axes_body', {{asils.models.sizering.spare_axis().'}}); %#ok<AGROW>
                    end
                    algs{end+1} = 'idmas_split'; %#ok<AGROW>
                case {'cmg', 'vscmg'}
                    fill{end+1} = struct('slot', a{1}, 'part', pn(a{1}), 'gimbal_axes_body', {rows_(gimbal_axes)}, ...
                        'spin_axes_body', {rows_(spin_axes)}); %#ok<AGROW>
                case 'rcs', fill{end+1} = struct('slot', 'rcs', 'part', pn('rcs')); algs{end+1} = 'rcs_pwm'; %#ok<AGROW>
            end
        end
        if st_fit
            fill{end+1} = struct('slot', 'star_tracker', 'part', 'SYN-ST-1', 'boresights_body', {rows_(heads(1:n_heads, :))}, ...
                'calibrated_residual_rad', residual); %#ok<AGROW>
        end
        if fine, gyro = gyro_id; else, gyro = 'SYN-GYRO-1'; end
        fill = [fill, {struct('slot', 'magnetometer', 'part', 'SYN-MAG-1'), ...
            struct('slot', 'sun_sensors', 'part', 'SYN-SUN-1', 'normals_body', {rows_(normals)}), ...
            struct('slot', 'gyro', 'part', gyro), struct('slot', 'gnss', 'part', 'TRN-GPS-1'), ...
            struct('slot', 'earth_sensor', 'part', 'SYN-ES-1', 'boresight_body', bs)}]; %#ok<AGROW>
        label = asils.util.getf(fa, 'label', id);
        pr = struct('schema', 'adcs-product/1', 'id', sprintf('SZ-%s-%s', cs, id), 'label', sprintf('%s — sized to %s', label, cs), ...
            'family', id, 'role', fa.role, 'classes', {{'cubesat_3u'}}, 'status', 'sized', 'origin', 'designed', ...
            'source', 'asils.sizing', 'algorithms', {algs}, 'sun_axis_body', sun_axis.', 'payload_boresight_body', bs, ...
            'fill', {fill}, 'knobs', k);
        write_(fullfile(out, 'products', [pr.id '.json']), pr);
        b = asils.sizing.budget(fill, lookup);
        families.(id) = struct('product', pr.id, 'role', fa.role, 'label', label, 'mass_kg', b.mass_kg, 'power_W', b.power_W, ...
            'volume_L', b.volume_L, 'items', {b.items});
    end
    Z = struct('schema', 'adcs-sizing/1', 'case', cs, 'class', Dm.class, 'star_tracker', logical(st_fit), 'demand', Dm, ...
        'knobs', k, 'families', families, 'parts', parts);
    write_(fullfile(out, 'sizing.json'), Z);
    if ~asils.util.getf(opts, 'quiet', false), asils.sizing.print(Z); end
end

function c = rows_(A)
% a list of directions as the files write it: one row each
    c = num2cell(A, 2).';
end
function p = lookup_(by_pn, R, id)
    if isKey(by_pn, id), p = by_pn(id); return; end
    p = asils.util.readjson(asils.product.find(R, 'parts', id));
end
function write_(f, s)
    fid = fopen(f, 'w'); fprintf(fid, '%s', jsonencode(s)); fclose(fid);
end
