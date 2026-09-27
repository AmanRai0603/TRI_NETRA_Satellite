function [moved, rel, note] = try_knob(knob, v1, v2, base, K)
%VALIDATION.TRY_KNOB  Does this sweep knob actually MOVE the acceleration?
%   [moved, rel, note] = validation.try_knob(knob, v1, v2, base, K)
%
%   ---------------------------------------------------------------------------
%   WHY THIS IS THE RIGHT TEST FOR A COMPARATOR
%   ---------------------------------------------------------------------------
%   A knob that does not move the answer is one of two very different things:
%     (a) the selected model does not CONSULT it -- e.g. gsi.aT is read by
%         'sentman' and ignored by 'dria'/'sesam', which derive aT themselves;
%     (b) the wiring is BROKEN -- the knob is set and then dropped on the floor.
%   Those look identical in a sweep table: a flat column. (a) is correct and
%   expected; (b) is the bug that made EXAMPLE_16U's documented SC.Cd knob a no-op
%   for as long as nobody checked, and that made every model knob unreachable from
%   compare_OD until round 16.
%
%   So sweep each knob against a base config where it SHOULD matter, and report
%   whether it moved. Flat where the docs say it should move = a real finding.
    moved = false; rel = 0; note = '';
    try
        [F1, SC1, IM1, IN1] = validation.sweep_knob(knob, v1, base.F, base.SC, base.IM, base.IN);
        [F2, SC2, IM2, IN2] = validation.sweep_knob(knob, v2, base.F, base.SC, base.IM, base.IN);
    catch err
        note = ['sweep_knob REJECTED: ' regexprep(err.message,'\n.*','')];
        moved = false; rel = -1; return
    end
    try
        a1 = accelOf_(F1, SC1, IM1, IN1, base, K);
        a2 = accelOf_(F2, SC2, IM2, IN2, base, K);
        d  = norm(a1 - a2);
        m  = max(norm(a1), norm(a2));
        rel = d / max(m, realmin);
        moved = rel > 1e-12;
    catch err
        note = ['RUN FAILED: ' regexprep(err.message,'\n.*','')];
        rel = -1; moved = false;
    end
end

function a = accelOf_(F, SC, IM, IN, base, K)
    cfg = config.defaultConfig();
    cfg.epoch = base.epoch;
    aR  = K.Re_earth + base.alt_km*1000;
    r   = [aR;0;0]; v = [0; sqrt(K.mu_earth/aR); 0];
    cfg.r0=r; cfg.v0=v; cfg.tspan=60; cfg.output=struct('times',[0;60]);
    cfg.gravityField = struct('field','default','degree',6);
    cfg.spaceweather.manual = base.sw;
    cfg.integrator = IN; cfg.integrator.method = IM;
    cfg.spacecraft = struct('mass',SC.mass_kg,'Aref',SC.Aref_m2,'Cd',SC.Cd,'Cr',SC.Cr, ...
                            'R_bi',eye(3));
    if isfield(SC,'facets'),   cfg.spacecraft.facets   = SC.facets;   end
    if isfield(SC,'attitude'), cfg.spacecraft.attitude = SC.attitude; end
    cfg.forces = F;
    W = op.buildWorld(cfg);
    [~, parts] = op.accel(0, r, v, W);
    % compare the NON-GRAVITATIONAL sum: gravity swamps everything and would hide
    % a 100% change in drag behind its own 1e-7 share.
    a = [0;0;0];
    for nm = {'drag','srp','erp'}
        if isfield(parts, nm{1}), a = a + parts.(nm{1}); end
    end
    if isfield(parts,'gravity') && any(strcmp(fieldnames(F),'gravity'))
        % gravity knobs need gravity in the comparison, or they read as flat
        if isfield(F.gravity,'degree'), a = a + parts.gravity; end
    end
end
