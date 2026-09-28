function alg = select(dev, S)
%ASILS.FSW.SELECT  Resolve which algorithm fills each FSW slot, and check it
%   against the hardware the product carries.
%
%   One job, several algorithms, several kinds of hardware: every algorithm is
%   registered once in catalogue/algorithms/<id>.toml with its SLOT (the job:
%   detumble, attitude, pointing (momentum devices), mtq_pointing (coils),
%   sun_acquisition (alias sun_spin), allocation, thrusters) and what it
%   NEEDS (coils, magnetometer, gyro, sun, momentum, wheels_or_rings, rings,
%   cmg, vscmg, rcs, attitude). The choice for a run is, in order:
%     1 the scenario's [fsw.algorithms] table (a trade or a test fixes it)
%     2 the product's [selected] table (the promoted outcome of a trade)
%     3 the first compatible default below
%   and a choice the hardware cannot fly is refused by name, before the run.
%   The allocation slot follows the hardware (cmg_sr, vscmg_sr, idmas_split,
%   rotor_pinv) unless a scenario overrides it.
    R = asils.util.root();
    has = caps_(dev);
    dflt = struct('detumble', {{'bdot_gyro', 'bdot_mag'}}, 'attitude', {{'mekf'}}, ...
        'pointing', {{'pid'}}, 'mtq_pointing', {{'mtq_pd'}}, 'sun_acquisition', {{'sunspin_l1l2'}}, ...
        'allocation', {{'cmg_sr', 'vscmg_sr', 'idmas_split', 'rotor_pinv'}}, 'thrusters', {{'rcs_pwm'}});
    pick = struct();
    if isfield(dev, 'selected'), pick = dev.selected; end
    if isfield(S, 'fsw') && isfield(S.fsw, 'algorithms')
        f = fieldnames(S.fsw.algorithms);
        for i = 1:numel(f), pick.(f{i}) = S.fsw.algorithms.(f{i}); end
    end
    if isfield(pick, 'sun_spin')             % legacy slot name of the coils-only Sun acquisition law
        if ~isfield(pick, 'sun_acquisition'), pick.sun_acquisition = pick.sun_spin; end
        pick = rmfield(pick, 'sun_spin');
    end
    slots = fieldnames(dflt); alg = struct();
    for i = 1:numel(slots)
        sl = slots{i};
        if isfield(pick, sl)
            id = pick.(sl); A = load_(R, id);
            miss = setdiff(A.needs, has);
            assert(isempty(miss), 'asils:select:hardware', ...
                'Algorithm %s (%s) cannot fly on %s: it needs %s.', id, sl, dev.id, strjoin(miss, ', '));
            assert(strcmp(A.slot, sl), 'asils:select:slot', 'Algorithm %s does %s, not %s.', id, A.slot, sl);
            alg.(sl) = id;
        else
            alg.(sl) = '';
            c = dflt.(sl);
            for k = 1:numel(c)
                A = load_(R, c{k});
                if isempty(setdiff(A.needs, has)), alg.(sl) = c{k}; break, end
            end
        end
    end
    alg.caps = has;
end

function A = load_(R, id)
    f = fullfile(R, 'data', 'algorithms', [id '.json']);
    assert(exist(f, 'file') == 2, 'asils:select:unknown', 'No algorithm %s in the registry (catalogue/algorithms).', id);
    A = asils.util.readjson(f);
    if ~iscell(A.needs), A.needs = cellstr(A.needs); end
end

function c = caps_(dev)
%CAPS_  What the product can do, in the vocabulary of the algorithms' needs.
    c = {};
    if dev.mtq.fitted, c{end+1} = 'coils'; end
    if dev.mag.fitted, c{end+1} = 'magnetometer'; end
    if dev.gyro.fitted, c{end+1} = 'gyro'; end
    if dev.sun.fitted || dev.css.fitted, c{end+1} = 'sun'; end
    if dev.st.fitted, c{end+1} = 'star_tracker'; end
    if dev.gyro.fitted && (dev.st.fitted || (dev.mag.fitted && (dev.sun.fitted || dev.css.fitted)))
        c{end+1} = 'attitude';
    end
    if dev.rcs.fitted, c{end+1} = 'rcs'; end
    X = dev.mex;
    if X.fitted
        c{end+1} = 'momentum';
        k = X.kind;
        if any(strcmp(k, 'rw')) || any(strcmp(k, 'fmr')), c{end+1} = 'wheels_or_rings'; end
        if any(strcmp(k, 'fmr')), c{end+1} = 'rings'; end
        if any(strcmp(k, 'cmg')), c{end+1} = 'cmg'; end
        if any(strcmp(k, 'vscmg')), c{end+1} = 'vscmg'; end
    end
end
