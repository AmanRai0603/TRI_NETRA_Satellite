function B = error_budget(cfg, arcSec, verbose)
%VALIDATION.ERROR_BUDGET  What is each force ACTUALLY worth, in metres, on THIS arc?
%   B = validation.error_budget(cfg, arcSec)
%
%   ---------------------------------------------------------------------------
%   WHY MEASURED AND NOT ESTIMATED
%   ---------------------------------------------------------------------------
%   The tempting shortcut is dx = 0.5*a*t^2. It is right for a SECULAR
%   acceleration -- a drag deficit points along-track and accumulates -- and badly
%   WRONG for a periodic one. Solid tides at 3.05e-07 m/s^2 "predict" 17.8 m over a
%   3 h arc by that formula; the real figure for LEO is decimetres, because the tide
%   reverses twice a day and the orbit never gets to integrate it. Third-body
%   "predicts" 62 m by the same arithmetic and is mostly absorbed into the orbit's
%   own shape.
%
%   So this does not estimate. It PROPAGATES the arc with the force on, propagates
%   it again with the force off, and reports the position difference that actually
%   results. That costs one propagation per term and answers the question the
%   arithmetic cannot.
%
%   THE POINT: when a residual is 2.23 m, the only useful question is which terms
%   are big enough to be responsible. A term worth 0.1 m is not your problem no
%   matter how wrong it is; a term worth 5 m is, even if it looks well-modelled.
%
%   B.name / B.dx_m / B.dr_rtn   per-force position effect over the arc
    if nargin<2 || isempty(arcSec), arcSec = 3*3600; end
    if nargin<3, verbose = true; end

    TERMS = {'drag','thirdbody','srp','erp','relativity','solidtides','oceantides'};

    c0 = cfg;
    c0.tspan  = arcSec;
    c0.output = struct('times', (0:60:arcSec).');
    sol0 = op.propagate(c0);           % everything the caller asked for: the baseline

    B = struct('name',{},'dx_m',{},'was_on',{},'dr_rtn',{});
    for i = 1:numel(TERMS)
        nm = TERMS{i};
        if ~isfield(c0.forces, nm), continue, end
        wasOn = subsref_default(c0.forces.(nm), 'on', false);
        c1 = c0;
        c1.forces.(nm).on = ~wasOn;    % flip it: on->off measures what it CONTRIBUTES,
                                       % off->on measures what OMITTING it COSTS
        try
            sol1 = op.propagate(c1);
            d  = sol1.r - sol0.r;
            dx = sqrt(mean(sum(d.^2,2)));
            rt = validation.rtn_project(d, sol0.r, sol0.v);
            B(end+1) = struct('name',nm, 'dx_m',dx, 'was_on',wasOn, ...
                              'dr_rtn',[sqrt(mean(rt(:,1).^2)) sqrt(mean(rt(:,2).^2)) sqrt(mean(rt(:,3).^2))]); %#ok<AGROW>
        catch err
            if verbose, fprintf('  [%s: %s]\n', nm, regexprep(err.message,'\n.*','')); end
        end
    end

    if ~isempty(B)
        [~, ord] = sort([B.dx_m], 'descend');
        B = B(ord);
    end

    if ~verbose, return, end
    fprintf('\n  ---- ERROR BUDGET: measured, not estimated (%.1f h arc) ----\n', arcSec/3600);
    fprintf('  Each row = propagate twice, once with the force flipped, and take the\n');
    fprintf('  position difference. dx=0.5*a*t^2 is only valid for SECULAR terms and\n');
    fprintf('  overestimates periodic ones by an order of magnitude or more.\n\n');
    fprintf('  %-12s | %-6s | %-11s | radial   along    cross  [m RMS]\n','force','state','|dr| RMS');
    fprintf('  %s\n', repmat('-',1,72));
    for i = 1:numel(B)
        fprintf('  %-12s | %-6s | %9.4f m | %7.4f %7.4f %7.4f\n', B(i).name, ...
                subsref_tern(B(i).was_on,'ON','OFF'), B(i).dx_m, B(i).dr_rtn);
    end
    fprintf('\n  ON  = what this force CONTRIBUTES (turning it off moves the orbit this far)\n');
    fprintf('  OFF = what OMITTING it COSTS (turning it on would move the orbit this far)\n');
    fprintf('  Any OFF row larger than your residual is a candidate for the residual.\n');
end
