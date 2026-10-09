function k = knobs(s)
%ASILS.SIZING.KNOBS  What the convergence loop may change between iterations of the sizing (the engine's adcs-design
%   Knobs): per-part authority scales (mtq, mtqp, rw, cmg, vscmg, fmr, rcs; 1 = the law's size), the momentum margin
%   k_h (empty: the design's), the torque margin k_tau, the fluid loop's pump exchange rate fmr_lambda [kg/W], a star
%   tracker on a coarse product too, the star-tracker heads, the flow sensor's noise [m/s], the gyro grade, a spare ring.
%   k = asils.sizing.knobs()                       the design's (sizedemand's sizing_knobs)
%   k = asils.sizing.knobs(struct('k_tau', 2, 'scale', struct('rw', 1.5)))
%   A key the sizing does not read, or a value outside the design's range, is refused by name.
%   Owner: Agastya. Copyright (c) 2026 Agastya. All rights reserved.
    [k_tau, lambda, heads, sigma, grade, k_lo, k_hi, l_lo, l_hi, h_lo, h_hi, s_lo, s_hi, g_lo, g_hi] = ...
        asils.models.sizedemand.sizing_knobs();
    k = struct('scale', struct(), 'k_h', [], 'k_tau', k_tau, 'fmr_lambda', lambda, 'star_tracker', false, ...
        'st_heads', heads, 'fmr_flow_sigma', sigma, 'gyro_grade', grade, 'fmr_spare', false);
    if nargin < 1 || isempty(s), return; end
    keys = fieldnames(k);
    f = fieldnames(s);
    for i = 1:numel(f)
        assert(any(strcmp(f{i}, keys)), 'asils:sizing:refused', 'knobs: %s is not a knob the sizing reads (%s)', ...
            f{i}, strjoin(keys', ', '));
    end
    if isfield(s, 'scale') && ~isempty(s.scale)
        assert(isstruct(s.scale), 'asils:sizing:refused', 'knobs: scale must be an object of part kind -> factor');
        g = fieldnames(s.scale);
        for i = 1:numel(g)
            x = s.scale.(g{i});
            assert(isnumeric(x) && isscalar(x) && isfinite(x) && x > 0, 'asils:sizing:refused', ...
                'knobs: scale.%s is not a positive factor', g{i});
            k.scale.(g{i}) = x;
        end
    end
    if isfield(s, 'k_h') && ~isempty(s.k_h), k.k_h = num_(s, 'k_h', k_lo, k_hi, 0); end
    k.k_tau = num_(s, 'k_tau', k_lo, k_hi, k.k_tau);
    k.fmr_lambda = num_(s, 'fmr_lambda', l_lo, l_hi, k.fmr_lambda);
    k.st_heads = fix(num_(s, 'st_heads', h_lo, h_hi, k.st_heads));
    k.fmr_flow_sigma = num_(s, 'fmr_flow_sigma', s_lo, s_hi, k.fmr_flow_sigma);
    k.gyro_grade = num_(s, 'gyro_grade', g_lo, g_hi, k.gyro_grade);
    for b = {'star_tracker', 'fmr_spare'}
        if isfield(s, b{1}) && ~isempty(s.(b{1}))
            assert(islogical(s.(b{1})) && isscalar(s.(b{1})), 'asils:sizing:refused', 'knobs: %s is not true or false', b{1});
            k.(b{1}) = s.(b{1});
        end
    end
end

function x = num_(s, key, lo, hi, d)
    if ~isfield(s, key) || isempty(s.(key)), x = d; return; end
    x = s.(key);
    assert(isnumeric(x) && isscalar(x) && isfinite(x) && x >= lo && x <= hi, 'asils:sizing:refused', ...
        'knobs: %s = %g is not a number from %g to %g', key, x, lo, hi);
end
