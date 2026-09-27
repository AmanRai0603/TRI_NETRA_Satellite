function CAP = capability(SAT, verbose)
%VALIDATION.CAPABILITY  Which models can this satellite actually support?
%   CAP = validation.capability('CHAMP')
%
%   ---------------------------------------------------------------------------
%   THE POINT
%   ---------------------------------------------------------------------------
%   A model is not available just because the code for it exists. It is available
%   when the DATA it needs exists for THIS satellite. Box-wing SRP needs box
%   dimensions; nothing in this toolbox carries CHAMP's or GRACE's. Metric [3]
%   needs an accelerometer product. Metric [5] needs a TU Delft file for that exact
%   satellite and epoch.
%
%   Selecting a model whose inputs are missing does not degrade the answer, it
%   voids it -- and the failure modes here have all been silent ones:
%     - box-wing with no dimensions -> you invent a box, and you are now fitting
%     - panel drag with no attitude -> R_bi = eye(3) -> drag ~ 0, no error
%     - TU Delft for the wrong twin  -> GRACE-B validated against GRACE-A, 247 km
%
%   So this reads the catalog and says, per model, RUN or WHY NOT. validate_OD
%   prints it in the PLAN and refuses to start on a model whose inputs are absent.
%
%   CAP.drag / .srp / .erp   cellstr of runnable model names
%   CAP.metrics              cellstr of runnable metric ids
%   CAP.notes                why each unavailable thing is unavailable
    if nargin<2, verbose = true; end
    C = validation.itsg_catalog(SAT);

    hasGeom = C.has_geometry && ~isempty(C.geom_dims);
    hasAcc  = strcmpi(strtrim(C.has_acc),'yes');
    hasDen  = strcmpi(strtrim(C.has_density),'yes');
    hasTud  = C.has_tudelft;
    % Attitude ships with every ITSG satellite that has an accelerometer -- the ACC
    % product is useless without it, so they are published together.
    hasAtt  = hasAcc;

    CAP = struct();
    CAP.notes = {};

    % ---- drag ---------------------------------------------------------------
    CAP.drag = {'cannonball'};                     % needs only mass/Aref/Cd: always
    if hasAtt
        CAP.drag = [CAP.drag, {'sentman','dria','cll','sesam'}];
    else
        CAP.notes{end+1} = ['drag panel models (sentman/dria/cll/sesam): NO attitude ' ...
            'product -> R_bi would stay eye(3) and the drag collapses toward zero.'];
    end

    % ---- srp ----------------------------------------------------------------
    % 'boxwing' with GEOMETRY='aref_box' is ALWAYS runnable when there is an
    % attitude, because the box is ASSUMED rather than looked up. That is a real
    % option and it is listed -- but tagged, because a model whose geometry you
    % invented is not the same kind of answer as one whose geometry you measured.
    CAP.srp = {'cannonball'};
    if hasAtt
        CAP.srp{end+1} = 'boxwing(GEOMETRY=aref_box: ASSUMED shape)';
    end
    if hasGeom && hasAtt
        CAP.srp{end+1} = 'boxwing';
    else
        why = {};
        if ~hasGeom, why{end+1} = 'no box dimensions in the catalog'; end
        if ~hasAtt,  why{end+1} = 'no attitude product'; end
        CAP.notes{end+1} = ['srp boxwing: ' strjoin(why,' + ') '.'];
    end

    % ---- erp ----------------------------------------------------------------
    CAP.erp = {'knocke','simple','ceres'};
    if hasAtt
        CAP.erp{end+1} = 'boxwing(GEOMETRY=aref_box: ASSUMED shape)';
    end
    if hasGeom && hasAtt
        CAP.erp{end+1} = 'boxwing';
    else
        why = {};
        if ~hasGeom, why{end+1} = 'no box dimensions in the catalog'; end
        if ~hasAtt,  why{end+1} = 'no attitude product'; end
        CAP.notes{end+1} = ['erp boxwing: ' strjoin(why,' + ') '.'];
    end

    % ---- metrics ------------------------------------------------------------
    CAP.metrics = {'1 vs reducedDynamicOrbit','2 vs kinematicOrbit'};
    if hasAcc, CAP.metrics{end+1} = '3 vs nonConservativeForces';
    else,      CAP.notes{end+1} = 'metric [3]: no accelerometer product.'; end
    if hasDen, CAP.metrics{end+1} = '4 vs neutralDensity';
    else,      CAP.notes{end+1} = 'metric [4]: no ITSG density product.'; end
    if hasTud, CAP.metrics{end+1} = '5 vs TU Delft density';
    else,      CAP.notes{end+1} = 'metric [5]: no TU Delft coverage.'; end

    CAP.has = struct('geometry',hasGeom,'attitude',hasAtt,'acc',hasAcc, ...
                     'density',hasDen,'tudelft',hasTud);
    CAP.sat = C.name;

    if ~verbose, return, end
    fprintf('\n  ---- what %s can actually support (from itsg_catalog.csv) ----\n', C.name);
    fprintf('    data present : attitude=%d  acc=%d  ITSG-density=%d  TU-Delft=%d  geometry=%d\n', ...
            hasAtt, hasAcc, hasDen, hasTud, hasGeom);
    fprintf('    drag         : %s\n', strjoin(CAP.drag,', '));
    fprintf('    srp          : %s\n', strjoin(CAP.srp,', '));
    fprintf('    erp          : %s\n', strjoin(CAP.erp,', '));
    fprintf('    metrics      : %s\n', strjoin(CAP.metrics,', '));
    for i=1:numel(CAP.notes)
        fprintf('    [x] %s\n', CAP.notes{i});
    end
    if ~hasGeom
        fprintf(['    -> %s has NO measured dimensions. Two honest routes:\n' ...
                 '       (a) GEOMETRY=''plate''    -- drag only, A(t)=Aref*|n.vhat| from the\n' ...
                 '           attitude, no invented shape. Good for %s: it flies ram/nadir\n' ...
                 '           with small yaw. Useless for SRP (a plate edge-on has no lit area).\n' ...
                 '       (b) GEOMETRY=''aref_box'' -- ASSUME a box with the right ram face and\n' ...
                 '           an aspect ratio you choose. Then srp/erp boxwing run and you can\n' ...
                 '           see how much shape MOVES them. Tagged ASSUMED everywhere.\n' ...
                 '       What you cannot do is call (b) %s''s answer. Aref is one number;\n' ...
                 '       a box needs three; the shapes that match Aref at 0 deg differ 6x at 45.\n' ...
                 '       Put real dimensions in geom_dims_m and this note goes away.\n'], ...
                 C.name, C.name, C.name);
    end
end
