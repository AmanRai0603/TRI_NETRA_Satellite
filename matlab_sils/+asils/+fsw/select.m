function alg = select(dev, S)
%ASILS.FSW.SELECT  Which algorithm fills each flight-software slot, checked against the hardware the product carries:
%   the engine's config.rs select, in MATLAB (reading and refusing are code; the defaults are the design's).
%
%   The choice for a run is, in order: the scenario's [fsw.algorithms] table (a trade or a test fixes it), the
%   product's [selected] table (the promoted outcome of a trade), else the slot's default, fswchoice's
%   (asils.models.fswchoice.fsw_default_<slot>: the first of its defaults the product can fly, else none). An algorithm
%   the flight software does not fly, or one the hardware cannot fly, is refused by name. The slots and their
%   algorithms in the order the design's choices list them are the flight software's (the engine's FLOWN); the blob's
%   law numbers (`adcs params`) are made from the same choice.
%   Owner: Agastya. Copyright (c) 2026 Agastya. All rights reserved.
    R = asils.util.root();
    flown = {'detumble', {'bdot_gyro', 'bdot_mag', 'bdot_bangbang', 'genbdot_l1'};
             'attitude', {'mekf'};
             'pointing', {'pid', 'lqr', 'smc'};
             'mtq_pointing', {'mtq_pd', 'mtq_lqr', 'mtq_smc', 'mtq_rate_damp', 'mtq_lovera2004', 'mtq_celani2015', ...
                              'mtq_avanzini2021', 'mtq_celani2026', 'mtq_tango2013'};
             'sun_acquisition', {'sunspin_l1l2', 'sunspin_l1l2_e2', 'sunspin_damped', 'sunspin_deruiter2011', 'sun_boresight_celani2026'};
             'allocation', {'rotor_pinv', 'idmas_split', 'cmg_sr', 'vscmg_sr'};
             'thrusters', {'rcs_pwm'}};
    has = caps_(dev);
    pick = struct();
    if isfield(dev, 'selected') && isstruct(dev.selected), pick = merge_(pick, dev.selected); end
    if isfield(S, 'fsw') && isfield(S.fsw, 'algorithms') && isstruct(S.fsw.algorithms), pick = merge_(pick, S.fsw.algorithms); end
    if isfield(pick, 'sun_spin')                 % legacy slot name
        if ~isfield(pick, 'sun_acquisition'), pick.sun_acquisition = pick.sun_spin; end
        pick = rmfield(pick, 'sun_spin');
    end
    f = fieldnames(pick);
    for i = 1:numel(f)
        j = find(strcmp(flown(:, 1), f{i}), 1);
        if isempty(j)
            error('asils:select:slot', 'algorithm slot "%s": no such slot; the slots are %s', f{i}, strjoin(flown(:, 1)', ', '));
        end
        if ~any(strcmp(flown{j, 2}, pick.(f{i})))
            error('asils:select:unknown', 'algorithm %s (%s): the flight software does not fly it (it flies %s)', pick.(f{i}), f{i}, strjoin(flown{j, 2}, ', '));
        end
    end
    alg = struct();
    for j = 1:size(flown, 1)
        sl = flown{j, 1}; ids = flown{j, 2};
        if isfield(pick, sl)
            id = pick.(sl); A = load_(R, id);
            miss = setdiff(A.needs, has);
            if ~isempty(miss), error('asils:select:hardware', 'algorithm %s (%s) cannot fly on %s: it needs %s', id, sl, dev.id, strjoin(miss, ', ')); end
            if ~strcmp(A.slot, sl), error('asils:select:slot', 'algorithm %s does %s, not %s', id, A.slot, sl); end
            alg.(sl) = id;
        else
            % the slot's default, fswchoice's: the first of its defaults the product can fly, else none ("")
            can = zeros(numel(ids), 1);
            for k = 1:numel(ids), A = load_(R, ids{k}); can(k) = isempty(setdiff(A.needs, has)); end
            o = feval(['asils.models.fswchoice.fsw_default_' sl], can);
            if o < numel(ids), alg.(sl) = ids{o + 1}; else, alg.(sl) = ''; end
        end
    end
    alg.caps = has;
end

function p = merge_(p, s)
    f = fieldnames(s);
    for i = 1:numel(f), if ischar(s.(f{i})), p.(f{i}) = s.(f{i}); end, end
end

function A = load_(R, id)
    f = fullfile(R, 'data', 'algorithms', [id '.json']);
    assert(exist(f, 'file') == 2, 'asils:select:unknown', 'no algorithm %s in the registry', id);
    A = asils.util.readjson(f);
    n = asils.util.getf(A, 'needs', {});
    if ischar(n), n = {n}; end
    A.needs = n;
    A.slot = asils.util.getf(A, 'slot', '');
end

function c = caps_(dev)
%CAPS_  What the product can do, in the vocabulary of the algorithms' needs (the engine's Dev::caps).
    c = {};
    if dev.mtq.fitted, c{end+1} = 'coils'; end
    if dev.mag.fitted, c{end+1} = 'magnetometer'; end
    if dev.gyro.fitted, c{end+1} = 'gyro'; end
    sun = dev.sun.fitted || dev.css.fitted;
    if sun, c{end+1} = 'sun'; end
    if dev.st.fitted, c{end+1} = 'star_tracker'; end
    if dev.gyro.fitted && (dev.st.fitted || (dev.mag.fitted && sun)), c{end+1} = 'attitude'; end
    if dev.rcs.fitted, c{end+1} = 'rcs'; end
    x = dev.mex.rec;
    if x.n > 0
        c{end+1} = 'momentum';
        k = x.kind(1:x.n);                      % rotorset's RotorKind: 0 wheel, 1 ring, 2 CMG, 3 VSCMG
        if any(k == 0 | k == 1), c{end+1} = 'wheels_or_rings'; end
        if any(k == 1), c{end+1} = 'rings'; end
        if any(k == 2), c{end+1} = 'cmg'; end
        if any(k == 3), c{end+1} = 'vscmg'; end
    end
end
