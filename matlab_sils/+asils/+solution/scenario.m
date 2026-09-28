function S = scenario(caseId, modeId, optionId)
%ASILS.SOLUTION.SCENARIO  The in-memory scenario that flies one mission mode
%   (catalogue/modes/<mode>.toml) with one of its options, on the product
%   sized to the case for that option's family (asils.sizing.size_all).
%   S = asils.solution.scenario('ais_3u', 'nadir_pointing', 'fmr+rcs')
%   Every option of a mode starts from the same [test] conditions and is
%   judged by the same [[metrics]], so a mode's table compares actuators.
    M = asils.solution.mode(modeId);
    opt = [];
    for i = 1:numel(M.options)
        if strcmp(M.options{i}.id, optionId), opt = M.options{i}; end
    end
    assert(~isempty(opt), 'asils:solution:option', 'Mode %s has no option %s', modeId, optionId);
    R = asils.util.root();
    C = asils.case.read(fullfile(R, 'cases', [caseId '.csv']));
    a = 6378137 + C.v.orbit_alt*1e3; T = 2*pi*sqrt(a^3/3.986004418e14);
    orbits = asils.util.getf(opt, 'duration_orbits', M.test.duration_orbits);
    win = asils.util.getf(opt, 'window', M.test.window);
    S = struct();
    S.id = sprintf('%s__%s__%s', caseId, modeId, strrep(optionId, '+', '_'));
    S.label = sprintf('%s — %s with %s', caseId, M.label, optionId);
    S.case_id = caseId; S.mode = modeId; S.option = optionId;
    S.product = sprintf('SZ-%s-%s', caseId, opt.family);
    S.time = struct('duration_s', round(orbits*T), 'dt_s', opt.dt_s, 'record_dt_s', 1.0);
    S.initial = struct('attitude', M.test.attitude, 'rate', M.test.rate);
    g = M.guidance; if any(strcmp(g, {'none', 'sun_vector'})), g = 'nadir'; end
    S.fsw = struct('start_mode', opt.fsw_mode, 'guidance', struct('kind', g));
    if isfield(opt, 'algorithms'), S.fsw.algorithms = opt.algorithms; end
    S.fsw.rcs_dump = double(strcmp(asils.util.getf(opt, 'dump', ''), 'rcs'));
    if strcmp(opt.actuator, 'fmr'), S.fsw.dump_gain = 0.03; end     % fluid loops: small capacity, dump faster
    ms = M.metrics;
    for i = 1:numel(ms)
        if isfield(ms{i}, 'window') && strcmp(ms{i}.window, M.test.window), ms{i}.window = win; end
    end
    S.metrics = ms;
end
