function SC = spacecraft(name, overrides)
%SAT.SPACECRAFT  THE spacecraft property struct. One name per quantity, one source.
%   (Named sat.spacecraft, not sat.properties: `properties` is a MATLAB/Octave
%   built-in and shadowing it warns on every path add.)
%   SC = sat.spacecraft(name)              from itsg_catalog.csv
%   SC = sat.spacecraft(name, overrides)   catalog, then overridden field by field
%   SC = sat.spacecraft([], overrides)     no catalog: overrides must supply everything
%
%   ---------------------------------------------------------------------------
%   WHY THIS EXISTS
%   ---------------------------------------------------------------------------
%   The reference area had THREE names in this tree and the mass had TWO, for one
%   physical quantity each:
%
%       area :  validation.itsg_catalog -> .area
%               sat.catalog             -> .Aref
%               validation.sweep_knob   -> .Aref_m2
%               cfg.spacecraft / physics-> .Aref
%       mass :  catalog + physics       -> .mass
%               sweep_knob / EXAMPLE_16U-> .mass_kg
%
%   Nothing was broken by that, because validate_OD:360 carried a translation line
%   (cfg.spacecraft.Aref = SC.area). It worked because somebody remembered. That is
%   precisely the shape of the Cd bug: Cd lived in cfg.spacecraft and the physics
%   read cfg.forces.drag, the two never met, and CHAMP silently flew at 2.2 instead
%   of 3.0 for as long as nobody checked. A translation line is a bug waiting for
%   the next person who writes a script and does not know it is needed.
%
%   So: ONE canonical name per quantity, and the physics layer's names win because
%   they are the ones the forces actually read.
%
%   ---------------------------------------------------------------------------
%   THE CANONICAL CONTRACT  (== cfg.spacecraft, unchanged)
%   ---------------------------------------------------------------------------
%     .mass      [kg]      required by drag, srp, erp
%     .Aref      [m^2]     reference area (cannonball; also the plate area)
%     .Cd        [-]       cannonball drag coefficient. IGNORED by the panel models,
%                          which compute Cd(t) from the GSI physics instead.
%     .Cr        [-]       cannonball reflectivity. IGNORED by srp 'boxwing', which
%                          uses the per-facet optics instead.
%     .facets    []        srp-format facets (.type .n .A .alpha .rho_s .rho_d
%                          .axis .double). Drag reads .n/.A; SRP reads all of it.
%                          ONE set: two would be two spacecraft.
%     .attitude  []        [] | 'ram' | 3x3 DCM | @(utc) | struct(.mjd,.q)
%     .R_bi      3x3       fallback attitude if .attitude is empty
%
%   WHICH FIELDS A MODEL ACTUALLY READS -- this is the point of the toggles:
%
%     drag 'cannonball'      : mass, Aref, Cd            (attitude NOT read)
%     drag 'sentman'         : mass, facets, attitude    + gsi.aT   -> Cd(t), A(t)
%     drag 'dria'/'sesam'    : mass, facets, attitude    + atm.n,T  -> Cd(t), A(t)
%     drag 'cll'             : mass, facets, attitude    + gsi.sig_* -> Cd(t), A(t)
%     srp  'cannonball'      : mass, Aref, Cr            (attitude NOT read)
%     srp  'boxwing'         : mass, facets, attitude    -> A(t), optics
%     erp  'knocke|simple|ceres': mass, Aref, Cr         (attitude NOT read -- there
%                                 is no erp box-wing in this tree)
%
%   So Cd is not "wrong" when unused by a panel model -- it is not consulted. Ask
%   sat.report(SC, cfg) to print which fields the CURRENT toggles will actually read.
    if nargin < 2, overrides = struct(); end

    SC = struct('mass',[], 'Aref',[], 'Cd',[], 'Cr',[], ...
                'facets',[], 'attitude',[], 'R_bi',eye(3), 'source','');

    if ~isempty(name)
        C = validation.itsg_catalog(name);
        SC.mass = C.mass;
        SC.Aref = C.Aref;      % canonical -- itsg_catalog used to call this .area
        SC.Cd   = C.Cd;
        SC.Cr   = C.Cr;
        SC.source = sprintf('itsg_catalog.csv:%s', C.name);
    end

    % overrides, field by field, so a partial override does not wipe the catalog
    fn = fieldnames(overrides);
    for i = 1:numel(fn)
        if ~isempty(overrides.(fn{i}))
            SC.(fn{i}) = overrides.(fn{i});
            if ~strcmp(fn{i},'source')
                SC.source = [SC.source ' +' fn{i}];
            end
        end
    end

    need = {'mass','Aref'};
    for i = 1:numel(need)
        if isempty(SC.(need{i})) || ~isfinite(SC.(need{i})) || SC.(need{i}) <= 0
            error('sat:properties:missing', ...
              ['spacecraft.%s is required and must be positive (got %s). Give a ' ...
               'catalog name, or pass it in overrides.'], need{i}, mat2str(SC.(need{i})));
        end
    end
end
