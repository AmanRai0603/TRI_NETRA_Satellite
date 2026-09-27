function S = shape_leverage(cfg, aspects, verbose)
%VALIDATION.SHAPE_LEVERAGE  What is the ASSUMED box worth? Sweep it and see.
%   S = validation.shape_leverage(cfg, aspects)
%
%   ---------------------------------------------------------------------------
%   WHY THIS REFUSES TO LET YOU FIT
%   ---------------------------------------------------------------------------
%   GEOMETRY='aref_box' builds a box from Aref plus an aspect ratio YOU CHOOSE.
%   Aref is ONE number; a box needs THREE. The inversion is not unique, so the
%   aspect ratio is a free parameter -- and on GRACE-A, measured:
%
%       cannonball SRP               -> 0.922 of measured
%       boxwing, ASSUMED cube        -> 1.006      <-- looks perfect
%       boxwing, ASSUMED 2:1:1       -> 1.300
%       boxwing, ASSUMED 3.1x0.7x0.7 -> 2.300
%
%   A factor of 2.5, from nothing but the shape someone typed. A free parameter
%   with that much leverage can hit ANY target you point it at. "Box-wing SRP
%   closes the gap" would be arithmetically true and worthless: it would be
%   FITTING -- tuning a free parameter until the residual vanishes and calling the
%   vanishing a result.
%
%   So whenever an ASSUMED shape is in play, print the whole sweep. The SPREAD is
%   the size of the assumption, and a residual smaller than the spread is not a
%   finding. This function exists to make that impossible to skip.
%
%   The direction matters too. On GRACE-A the box CLOSEST to the real spacecraft
%   OVERSHOOTS by 2.3x -- so the honest reading is not "boxwing fixes it" but
%   "boxwing with a plausible shape overshoots, therefore Cr, the optics or the
%   area is wrong somewhere". That is a real lead. The cube was a mirage.
    if nargin<3, verbose = true; end
    if nargin<2 || isempty(aspects)
        aspects = {[1 1 1], [2 1 1], [3 1 1], [4.4 1 1], [1 2 1]};
    end
    S = struct('aspect',{},'dims',{},'a_srp',{},'a_ng',{});

    Aref = subsref_default(cfg.spacecraft,'Aref',1);
    opt  = struct('alpha',0.2,'rho_s',0.3,'rho_d',0.5);
    for k = 1:numel(aspects)
        as = aspects{k};
        L  = sqrt(Aref/(as(2)*as(3)));           % scale so the +x face area == Aref
        d  = [as(1)*L, as(2)*L, as(3)*L];
        c  = cfg;
        c.spacecraft.facets = srp.buildBox(d(1), d(2), d(3), opt);
        c.forces.srp.model  = 'boxwing';
        try
            W = op.buildWorld(c);
            [~, parts] = op.accel(0, c.r0, c.v0, W);
            ng = [0;0;0];
            for nm = {'drag','srp','erp'}
                if isfield(parts,nm{1}), ng = ng + parts.(nm{1}); end
            end
            sp = [0;0;0]; if isfield(parts,'srp'), sp = parts.srp; end
            S(end+1) = struct('aspect',as,'dims',d,'a_srp',norm(sp),'a_ng',norm(ng)); %#ok<AGROW>
        catch
        end
    end

    if ~verbose || isempty(S), return, end
    v = [S.a_ng];
    fprintf('\n  ---- SHAPE LEVERAGE: what is the ASSUMED box worth? ----\n');
    fprintf('    Aref = %.3f m^2 is ONE number. A box needs THREE. The aspect ratio\n', Aref);
    fprintf('    below is FREE -- nobody measured it. Here is what it buys:\n\n');
    fprintf('      %-16s %-22s %-11s %s\n','aspect','dims [m]','srp [m/s^2]','non-grav [m/s^2]');
    for k = 1:numel(S)
        fprintf('      %-16s %-22s %.3e   %.3e\n', mat2str(S(k).aspect), ...
                sprintf('%.2f x %.2f x %.2f', S(k).dims), S(k).a_srp, S(k).a_ng);
    end
    fprintf('\n    SPREAD: %.3e .. %.3e  = a factor of %.1f\n', min(v), max(v), max(v)/min(v));
    fprintf(['    That factor is the SIZE OF THE ASSUMPTION. A residual smaller than\n' ...
             '    this spread is not a finding -- you could reach it by retyping the\n' ...
             '    aspect ratio, which is fitting, not modelling.\n']);
end
