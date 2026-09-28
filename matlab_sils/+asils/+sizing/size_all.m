function Z = size_all(caseId, opts)
%ASILS.SIZING.SIZE_ALL  Size every actuator option to a case and assemble one
%   product per family (catalogue/families.toml). Writes
%     store/sized/<case>/parts/<id>.json      sized parts
%     store/sized/<case>/products/<id>.json   SZ-<case>-<family>
%     store/sized/<case>/sizing.json          demand, parts, per-family budgets
%   Z = asils.sizing.size_all('ais_3u')
%
%   SENSOR SUITE (same for every family of a case, so families differ only in
%   actuators), chosen from the case's knowledge requirement:
%     fine  (req.ake <= 0.05 deg): two star-tracker heads (in-orbit calibrated),
%           precision gyro, magnetometer, six Sun heads, GNSS, Earth sensor;
%           payload +Y (camera), heads on -Y at +/-25 deg.
%     coarse: magnetometer, six Sun heads, MEMS gyro, GNSS, Earth sensor, no
%           star tracker; payload = the long axis +X on nadir (gravity-gradient
%           stable, docs/RESULTS.md). A mode lists the star tracker in its
%           navigation; it is used wherever the case's accuracy fits one.
%   Power face (Sun referencing and acquisition): -Z (sun_axis_body).
    if nargin < 2, opts = struct(); end
    R = asils.util.root();
    caseFile = fullfile(R, 'cases', [caseId '.csv']);
    Dm = asils.sizing.demand(caseFile);
    out = fullfile(R, 'store', 'sized', caseId);
    for d = {'parts', 'products'}, if ~exist(fullfile(out, d{1}), 'dir'), mkdir(fullfile(out, d{1})); end, end
    box = [0.34 0.10 0.10];
    P = struct();
    [P.mtq, P.mtqp] = asils.sizing.mtq(Dm, caseId);
    P.rw = asils.sizing.rw(Dm, caseId);
    P.cmg = asils.sizing.cmg(Dm, caseId, false);
    P.vscmg = asils.sizing.cmg(Dm, caseId, true);
    fm = asils.sizing.fmr(Dm, caseId, box); P.fmr_x = fm{1}; P.fmr_y = fm{2}; P.fmr_z = fm{3};
    P.rcs = asils.sizing.rcs(Dm, caseId, box);
    f = fieldnames(P);
    for i = 1:numel(f), write_(fullfile(out, 'parts', [P.(f{i}).part_number '.json']), P.(f{i})); end

    fine = isfinite(Dm.req.ake) && Dm.req.ake <= 0.05;
    Fam = asils.util.readjson(fullfile(R, 'data', 'families.json'));
    fams = Fam.family; if ~iscell(fams), fams = num2cell(fams); end
    I3 = [1 0 0; 0 1 0; 0 0 1];
    Z = struct('case', caseId, 'demand', Dm, 'class', ternary_(fine, 'fine', 'coarse'), 'families', struct());
    for i = 1:numel(fams)
        fa = fams{i}; acts = fa.actuators; if ~iscell(acts), acts = cellstr(acts); end
        pr = struct('schema', 'adcs-product/1', 'id', sprintf('SZ-%s-%s', caseId, fa.id), ...
            'label', sprintf('%s — sized to %s', fa.label, caseId), 'family', fa.id, 'role', fa.role, ...
            'classes', {{'cubesat_3u'}}, 'status', 'sized', 'origin', 'designed', 'source', 'asils.sizing', ...
            'algorithms', {{'bdot', 'mekf'}}, 'sun_axis_body', [0 0 -1]);
        coil = P.mtq.part_number; if isequal(acts, {'mtq'}), coil = P.mtqp.part_number; end   % coils-only: pointing-grade coils
        fill = {struct('slot', 'coils', 'part', coil, 'axes_body', I3)};
        for a = acts(:)'
            switch a{1}
                case 'rw',    fill{end+1} = struct('slot', 'wheels', 'part', P.rw.part_number, 'axes_body', I3); %#ok<AGROW>
                case 'fmr'
                    fill{end+1} = struct('slot', 'rings', 'part', P.fmr_x.part_number, 'axes_body', {{[1 0 0]}}); %#ok<AGROW>
                    fill{end+1} = struct('slot', 'rings', 'part', P.fmr_y.part_number, 'axes_body', {{[0 1 0]}}); %#ok<AGROW>
                    fill{end+1} = struct('slot', 'rings', 'part', P.fmr_z.part_number, 'axes_body', {{[0 0 1]}}); %#ok<AGROW>
                    pr.algorithms{end+1} = 'idmas_split';
                case {'cmg', 'vscmg'}
                    fill{end+1} = struct('slot', a{1}, 'part', P.(a{1}).part_number, ...
                        'gimbal_axes_body', [0.8165 0 0.5774; 0 0.8165 0.5774; -0.8165 0 0.5774; 0 -0.8165 0.5774], ...
                        'spin_axes_body', [0 1 0; -1 0 0; 0 -1 0; 1 0 0]); %#ok<AGROW>
                case 'rcs',   fill{end+1} = struct('slot', 'rcs', 'part', P.rcs.part_number); pr.algorithms{end+1} = 'rcs_pwm'; %#ok<AGROW>
            end
        end
        if fine
            pr.payload_boresight_body = [0 1 0];
            fill{end+1} = struct('slot', 'star_tracker', 'part', 'SYN-ST-1', ...
                'boresights_body', [0 -0.9063 0.4226; 0 -0.9063 -0.4226], 'calibrated_residual_rad', 1e-5); %#ok<AGROW>
            gyro = 'TRN-GYRO-P1';
        else
            pr.payload_boresight_body = [1 0 0];
            gyro = 'SYN-GYRO-1';
        end
        fill{end+1} = struct('slot', 'magnetometer', 'part', 'SYN-MAG-1'); %#ok<AGROW>
        fill{end+1} = struct('slot', 'sun_sensors', 'part', 'SYN-SUN-1', ...
            'normals_body', [1 0 0; -1 0 0; 0 1 0; 0 -1 0; 0 0 1; 0 0 -1]); %#ok<AGROW>
        fill{end+1} = struct('slot', 'gyro', 'part', gyro); %#ok<AGROW>
        fill{end+1} = struct('slot', 'gnss', 'part', 'TRN-GPS-1'); %#ok<AGROW>
        fill{end+1} = struct('slot', 'earth_sensor', 'part', 'SYN-ES-1', 'boresight_body', pr.payload_boresight_body); %#ok<AGROW>
        pr.fill = fill;
        write_(fullfile(out, 'products', [pr.id '.json']), pr);
        dev = asils.product.load(pr.id);
        Z.families.(fa.id) = struct('product', pr.id, 'role', fa.role, 'label', fa.label, ...
            'mass_kg', dev.budget.mass_kg, 'power_W', dev.budget.power_W, 'volume_L', dev.budget.volume_L, ...
            'items', {dev.budget.items});
    end
    Z.parts = P;
    write_(fullfile(out, 'sizing.json'), Z);
    if ~asils.util.getf(opts, 'quiet', false), asils.sizing.print(Z); end
end

function write_(f, s)
    fid = fopen(f, 'w'); fprintf(fid, '%s', jsonencode(s)); fclose(fid);
end
function x = ternary_(c, a, b)
    if c, x = a; else, x = b; end
end
