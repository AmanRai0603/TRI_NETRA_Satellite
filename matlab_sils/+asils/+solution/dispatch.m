function out = dispatch(caseId, familyId)
%ASILS.SOLUTION.DISPATCH  Package a case's solution for the next rungs.
%   out = asils.solution.dispatch('ais_3u')              % the recommended family
%   out = asils.solution.dispatch('ais_3u', 'mtq_fmr')   % a named one
%   Writes <repo>/dist/dispatch/<case>/<family>/:
%     product.json        the sized hardware: product + every part descriptor
%     modes.json          per mission mode: the method, controller state,
%                         algorithms, and the evidence (requirement vs worst run)
%     fsw_params.json     the flight-software parameters the SILS flew
%     adcs_fsw_params.h   the same as C #defines, for the OBC build
%     hal_map.json        each fitted component: HAL output, its chain and status
%     README.md           what this is and the rung plan SILS -> OILS -> HILS
%   Requires asils.solution.collect(case) to have run.
    R = asils.util.root();
    Sol = asils.util.readjson(fullfile(R, 'store', 'solutions', caseId, 'solution.json'));
    if nargin < 2 || isempty(familyId), familyId = Sol.recommended; end
    E = Sol.families.(familyId);
    out = fullfile(fileparts(R), 'dist', 'dispatch', caseId, familyId);
    if ~exist(out, 'dir'), mkdir(out); end
    % ---- hardware
    pid = sprintf('SZ-%s-%s', caseId, familyId);
    pr = asils.util.readjson(fullfile(R, 'store', 'sized', caseId, 'products', [pid '.json']));
    fills = pr.fill; if ~iscell(fills), fills = num2cell(fills); end
    parts = struct();
    for i = 1:numel(fills)
        f = fullfile(R, 'store', 'sized', caseId, 'parts', [fills{i}.part '.json']);
        if exist(f, 'file') ~= 2, f = fullfile(R, 'data', 'parts', [fills{i}.part '.json']); end
        parts.(strrep(fills{i}.part, '-', '_')) = asils.util.readjson(f);
    end
    write_(fullfile(out, 'product.json'), struct('product', pr, 'parts', parts, 'budget', ...
        struct('mass_kg', E.mass_kg, 'power_nominal_W', E.power_nominal_W, 'volume_L', E.volume_L)));
    % ---- modes, with evidence
    MT = asils.fsw.modes(); ids = Sol.mode_ids; if ~iscell(ids), ids = cellstr(ids); end
    modes = struct();
    for m = 1:numel(ids)
        M = asils.solution.mode(ids{m}); me = E.methods.(ids{m});
        opt = [];
        for i = 1:numel(M.options), if strcmp(M.options{i}.id, me.option), opt = M.options{i}; end, end
        O = Sol.modes.(ids{m}).options; if ~iscell(O), O = num2cell(O); end
        ev = struct();
        for i = 1:numel(O)
            if strcmp(O{i}.id, me.option), ev = O{i}.metrics; end
        end
        algs = struct();
        f = dir(fullfile(R, 'store', 'solutions', caseId, ids{m}, sprintf('%s_s*.mat', strrep(me.option, '+', '_'))));
        if ~isempty(f), L = load(fullfile(f(1).folder, f(1).name)); if isfield(L.s, 'alg'), algs = L.s.alg; end, end
        st = ''; if ~isempty(opt), st = opt.fsw_mode; end
        modes.(ids{m}) = struct('label', M.label, 'goal', M.goal, 'method', me.option, 'fsw_state', st, ...
            'feasible', me.feasible, 'algorithms', algs, 'navigation', {M.navigation}, 'exit', M.exit, ...
            'next', M.next, 'evidence', ev);
    end
    write_(fullfile(out, 'modes.json'), struct('case', caseId, 'family', familyId, 'modes', modes, ...
        'controller_states', struct('state', {MT.state}, 'mission_mode', {MT.mission})));
    % ---- flight-software parameters (as flown in the nadir-pointing test of this family)
    S = asils.solution.scenario(caseId, 'nadir_pointing', E.methods.nadir_pointing.option);
    P = asils.config(S, fullfile(R, 'cases', [caseId '.csv']));
    F = P.fsw;
    keep = {'mtq_period', 'mtq_meas', 'bdot_k', 'detumble_exit', 'detumble_hold_s', 'mtq', 'rw', 'ss', 'cmg', 'rcs', ...
            'rcsd', 'sa', 'capture_deg', 'capture_rate_deg_s', 'dump_k', 'h_bias', 'm_res_est', 'mekf', 'st_coast_s', ...
            'igrf_nmax', 'rate_lpf_s', 'fdir_s', 'alg'};
    fp = struct();
    for k = keep, if isfield(F, k{1}), fp.(k{1}) = F.(k{1}); end, end
    write_(fullfile(out, 'fsw_params.json'), fp);
    header_(fullfile(out, 'adcs_fsw_params.h'), fp, caseId, familyId);
    % ---- HAL / component map
    dev = asils.product.load(pid);
    comps = dir(fullfile(R, 'data', 'components', '*.json')); hm = struct();
    for i = 1:numel(comps)
        C = asils.util.readjson(fullfile(comps(i).folder, comps(i).name));
        if fitted_(C.id, dev), hm.(C.id) = C; end
    end
    write_(fullfile(out, 'hal_map.json'), hm);
    readme_(fullfile(out, 'README.md'), caseId, familyId, E, modes, ids, Sol);
    fprintf('dispatched %s / %s -> %s\n', caseId, familyId, out);
end

function tf = fitted_(id, dev)
    k = dev.mex.kind;
    switch id
        case 'star_tracker', tf = dev.st.fitted;
        case 'sun_sensor', tf = dev.sun.fitted;
        case 'coarse_sun_sensor', tf = dev.css.fitted;
        case 'earth_sensor', tf = dev.es.fitted;
        case 'magnetometer', tf = dev.mag.fitted;
        case 'gyro', tf = dev.gyro.fitted;
        case 'gnss', tf = dev.gps.fitted;
        case 'magnetorquer', tf = dev.mtq.fitted;
        case 'fluid_loop', tf = any(strcmp(k, 'fmr'));
        case 'rcs', tf = dev.rcs.fitted;
        case 'reaction_wheel', tf = any(strcmp(k, 'rw'));
        case 'cmg', tf = any(strcmp(k, 'cmg'));
        case 'vscmg', tf = any(strcmp(k, 'vscmg'));
        otherwise, tf = false;
    end
end

function header_(file, fp, caseId, familyId)
%HEADER_  Numeric parameters flattened to C #defines (ADCS_<PATH>), vectors as
%   brace initialisers; strings as comments.
    fid = fopen(file, 'w');
    fprintf(fid, '/* adcs_fsw_params.h -- generated by asils.solution.dispatch for %s / %s.\n', caseId, familyId);
    fprintf(fid, ' * The flight-software parameters the SILS flew. Copyright (c) 2026 Agastya. */\n');
    fprintf(fid, '#ifndef ADCS_FSW_PARAMS_H\n#define ADCS_FSW_PARAMS_H\n\n');
    emit_(fid, fp, 'ADCS');
    fprintf(fid, '\n#endif\n'); fclose(fid);
end

function emit_(fid, s, pre)
    f = fieldnames(s);
    for i = 1:numel(f)
        v = s.(f{i}); name = upper([pre '_' f{i}]);
        if isstruct(v) && isscalar(v), emit_(fid, v, name);
        elseif ischar(v), fprintf(fid, '/* %s = "%s" */\n', name, v);
        elseif islogical(v) || isnumeric(v)
            v = double(v(:)');
            if isempty(v), continue, end
            if numel(v) == 1, fprintf(fid, '#define %s (%.9g)\n', name, v);
            else, fprintf(fid, '#define %s {%s}\n', name, strjoin(arrayfun(@(x) sprintf('%.9g', x), v, 'UniformOutput', false), ', ')); end
        end
    end
end

function readme_(file, caseId, familyId, E, modes, ids, Sol)
    fid = fopen(file, 'w');
    fprintf(fid, '# Dispatch: %s for case %s\n\n**Owner: Agastya.** Copyright (c) 2026 Agastya. All rights reserved.\n\n', familyId, caseId);
    fprintf(fid, '%s (%s). Modes passed: %d of %d. Recommended for this case: **%s**. %s\n\n', E.label, E.role, E.passes, numel(ids), Sol.recommended, Sol.verdict);
    fprintf(fid, 'Budget (sized): %.3f kg, %.2f W nominal, %.3f L.\n\n', E.mass_kg, E.power_nominal_W, E.volume_L);
    fprintf(fid, '| mode | method | FSW state | feasible |\n|---|---|---|---|\n');
    for m = 1:numel(ids)
        x = modes.(ids{m}); fprintf(fid, '| %s | %s | %s | %s |\n', x.label, x.method, x.fsw_state, tf_(x.feasible));
    end
    fprintf(fid, '\n## Files\n\n- `product.json`: the sized hardware\n- `modes.json`: method, algorithms and SILS evidence per mode\n');
    fprintf(fid, '- `fsw_params.json`, `adcs_fsw_params.h`: flight parameters\n- `hal_map.json`: components on the HAL and their chains\n\n');
    fprintf(fid, '## Rungs\n\n1. **SILS** (this evidence): `asils.solution.run(''%s'')`, collected by `asils.solution.collect`.\n', caseId);
    fprintf(fid, '2. **Algorithm hardening**: the component chains (`+asils/+comp`) ported to the units and the OBC, node by node.\n');
    fprintf(fid, '3. **OILS**: the flight OBC runs the C flight software behind `adcs_hal.h`; `hal__backend = udp`, `hal__realtime = 1`.\n');
    fprintf(fid, '4. **HILS**: each component replaces its model one at a time (docs/OILS_HILS.md).\n');
    fclose(fid);
end

function s = tf_(b)
    if b, s = 'yes'; else, s = 'no'; end
end

function write_(f, s)
    fid = fopen(f, 'w'); fprintf(fid, '%s', jsonencode(s)); fclose(fid);
end
