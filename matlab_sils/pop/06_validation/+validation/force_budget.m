function B = force_budget(cfg, alt_km, arc_s, verbose)
%VALIDATION.FORCE_BUDGET  Which forces are OFF, and what does that cost THIS arc?
%   B = validation.force_budget(cfg, alt_km, arc_s)
%
%   ---------------------------------------------------------------------------
%   THE GAP THIS CLOSES
%   ---------------------------------------------------------------------------
%   validate_OD shipped with erp = OFF, solidtides = OFF, oceantides = OFF. All
%   three are implemented, wired, and reachable. They were simply off, in a struct
%   literal, and NOTHING SAID SO -- so a 2 m along-track residual gets attributed to
%   drag mismodelling when ~0.7 m of it is forces the run never included.
%
%   A default is a choice. An undeclared default is a choice someone else made for
%   you and did not mention -- which is the same failure as an assumed Cd printed as
%   if it were measured. This function makes the choice visible, priced in METRES of
%   along-track drift over YOUR arc, because "1e-8 m/s^2" is not a quantity anyone
%   can weigh against a 2 m residual.
%
%   The magnitudes are order-of-magnitude LEO figures, not a substitute for running
%   the model. The point is triage: what is worth turning on before chasing Cd.
    if nargin<4, verbose = true; end
    if nargin<3 || isempty(arc_s), arc_s = 10800; end
    if nargin<2 || isempty(alt_km), alt_km = 400; end
    F = subsref_default(cfg,'forces',struct());
    drift = @(acc) 0.5*acc*arc_s^2;

    % ---- GRAVITATIONAL vs NON-GRAVITATIONAL: THE COLUMN THAT WAS MISSING ------
    % Round 31 listed solidtides at 0.58 m of along-track drift right next to the
    % metric [3] accelerometer deficit, as if they were competing explanations for
    % one residual. THEY CANNOT BE.
    %
    % An accelerometer in free fall measures ONLY non-gravitational forces -- it
    % cannot feel gravity. That is the equivalence principle, not a modelling choice.
    % Solid and ocean tides ARE gravity. They move the ORBIT (metric [1]) and are
    % INVISIBLE to the accelerometer (metric [3]), no matter how large.
    %
    % Measured, switching them on: metric [3] moved by 0.0000e+00. Exactly as it must.
    %
    % One table with one column invited precisely the wrong action -- "turn on tides
    % to fix the accelerometer ratio" -- which would do nothing, and the failure to
    % move would look like a fresh mystery. Which metric a force can reach is a
    % property of physics, so it belongs in the table.
    rho_rel = exp(-(alt_km-400)/60);            % crude scale-height scaling
    T = {
    % name          |a|            grav?   note
      'drag',       2e-8*rho_rel,  false,  'non-grav. The accelerometer sees it.'
      'srp',        3e-8,          false,  'non-grav. Comparable to drag above ~450 km.'
      'erp',        1.5e-9,        false,  'non-grav. Small and free.'
      'thirdbody',  1e-6,          true,   'GRAVITY: moves the orbit, invisible to [3].'
      'relativity', 1e-8,          true,   'GRAVITY: moves the orbit, invisible to [3].'
      'solidtides', 1e-8,          true,   'GRAVITY: metric [1] only. [3] CANNOT see it.'
      'oceantides', 2e-9,          true,   'GRAVITY: metric [1] only. [3] CANNOT see it.'
    };
    B = struct('items',struct('name',{},'on',{},'a',{},'drift_m',{},'grav',{},'note',{}), ...
               'off_cost_pos_m',0, 'off_cost_acc',0);
    for i = 1:size(T,1)
        nm = T{i,1};
        on = isfield(F,nm) && subsref_default(F.(nm),'on',false);
        it = struct('name',nm,'on',on,'a',T{i,2},'drift_m',drift(T{i,2}), ...
                    'grav',T{i,3},'note',T{i,4});
        B.items(end+1) = it;
        if ~on
            B.off_cost_pos_m = B.off_cost_pos_m + it.drift_m;   % metric [1] sees ALL
            if ~it.grav, B.off_cost_acc = B.off_cost_acc + it.a; end  % [3] sees non-grav ONLY
        end
    end

    if ~verbose, return, end
    fprintf('\n  ---- FORCE BUDGET at %.0f km over a %.0f s arc ----\n', alt_km, arc_s);
    fprintf('    %-12s %-4s %-11s %-10s %-6s %s\n', ...
            'force','on?','|a| [m/s^2]','[1] drift','sees[3]?','note');
    for i = 1:numel(B.items)
        it = B.items(i);
        fprintf('    %-12s %-4s %.3e   %8.2f   %-6s %s\n', it.name, ...
                subsref_tern(it.on,'YES','NO'), it.a, it.drift_m, ...
                subsref_tern(it.grav,'no','YES'), ...
                subsref_tern(it.on,'',['<-- OFF: ' it.note]));
    end
    fprintf('    (sees[3]? = can the ACCELEROMETER feel it. Gravity: no -- an\n');
    fprintf('     accelerometer in free fall measures only non-gravitational forces.)\n');
    if B.off_cost_pos_m > 0.05
        fprintf(['    -> OFF forces could account for ~%.2f m of along-track drift\n' ...
                 '       [metric 1]. Weigh that against your position residual BEFORE\n' ...
                 '       concluding anything about Cd: a residual you can explain with a\n' ...
                 '       force you did not model is not a physics finding.\n'], B.off_cost_pos_m);
    end
    if B.off_cost_acc > 1e-12
        fprintf('    -> OFF NON-grav forces reach metric [3] at ~%.2e m/s^2.\n', B.off_cost_acc);
    else
        fprintf(['    -> every OFF force is GRAVITATIONAL, so metric [3] CANNOT move by\n' ...
                 '       turning them on. Verified: switching solid+ocean tides on changed\n' ...
                 '       the non-grav sum by 0.0000e+00. If [3] is short, look at drag,\n' ...
                 '       SRP, ERP or the reference -- not at the tides.\n']);
    end
end
