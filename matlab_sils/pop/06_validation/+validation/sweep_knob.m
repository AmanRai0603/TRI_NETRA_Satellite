function [F, SC, IM, IN] = sweep_knob(knob, val, F, SC, IM, IN)
%VALIDATION.SWEEP_KNOB  Apply ONE named sweep knob to a comparison config.
%   [F, SC, IM, IN] = validation.sweep_knob(knob, val, F, SC, IM, IN)
%
%   Explicit switch, no eval: the legal knobs stay visible in the source, and a
%   typo fails loudly instead of silently creating a struct field nothing reads.
%   Same idea as EXAMPLE_16U's applyKnob, but a package function rather than a
%   script local -- see the note at the bottom of compare_OD.m for why that matters.
%
%   F  = cfg.forces struct       SC = spacecraft struct (canonical .mass/.Aref, and
%                                     .Cd .Cr .facets .attitude)
%   IM = integrator method name  IN = integrator options struct
%
%   ---------------------------------------------------------------------------
%   THE MODEL KNOBS WERE MISSING
%   ---------------------------------------------------------------------------
%   This switch knew FORCES.drag.model but NOT srp.model, erp.model, the GSI
%   parameters, the attitude or the geometry. So every model wired into validate_OD
%   -- box-wing SRP, box-wing ERP, the four GSI models, measured vs ram attitude --
%   was unreachable from compare_OD. The comparator could not compare the models.
%
%   That is worse than it sounds: the SWEEP is the only mechanism in this toolbox
%   that catches a consistently-wrong parameter (the fixtures are built from the
%   engine's own output and are blind to one -- how Cd=2.2 hid inside a perfect
%   ratio 1.000). A knob that cannot be swept cannot be caught being wrong.
    switch knob
        case 'FORCES.drag.atmos',      F.drag.atmos     = val;
        case 'FORCES.drag.model',      F.drag.model     = val;   % cannonball|sentman|dria|cll|sesam
        case 'FORCES.srp.model',       F.srp.model      = val;   % cannonball|boxwing
        case 'FORCES.erp.model',       F.erp.model      = val;   % knocke|simple|ceres|boxwing
        case 'FORCES.erp.nrings',      F.erp.nrings     = val;   % Earth-cap resolution
        case 'FORCES.erp.nseg',        F.erp.nseg       = val;   %   (boxwing cost)
        % --- GSI: the gas-surface physics. Sweeping aT against metric [3] is the
        % point of having these models: 'sentman' takes the aT you type, while
        % 'dria'/'sesam' DERIVE it and ignore this.
        case 'FORCES.drag.gsi.aT',     F.drag.gsi.aT    = val;
        case 'FORCES.drag.gsi.Tw',     F.drag.gsi.Tw    = val;
        case 'FORCES.drag.gsi.sig_n',  F.drag.gsi.sig_n = val;   % cll only
        case 'FORCES.drag.gsi.sig_t',  F.drag.gsi.sig_t = val;   % cll only
        case 'FORCES.drag.corotate',   F.drag.corotate  = val;
        case 'FORCES.gravity.degree',  F.gravity.degree = val; F.gravity.order = val;
        case 'FORCES.gravity.model',   F.gravity.model  = val;
        case 'FORCES.thirdbody.on',    F.thirdbody.on   = val;
        case 'FORCES.srp.on',          F.srp.on         = val;
        case 'FORCES.erp.on',          F.erp.on         = val;
        case 'FORCES.relativity.on',   F.relativity.on  = val;
        case 'FORCES.solidtides.on',   F.solidtides.on  = val;
        case 'FORCES.oceantides.on',   F.oceantides.on  = val;
        case 'SC.Cd',                  SC.Cd            = val;
        % CANONICAL names first. 'SC.Aref_m2'/'SC.mass_kg' are the OLD names and
        % are still accepted, because they are documented in compare_OD/EXAMPLE_16U
        % and in your notes -- but they set the same field. One quantity, one place.
        case {'SC.Aref','SC.Aref_m2'},  SC.Aref_m2       = val;
        case {'SC.mass','SC.mass_kg'},  SC.mass_kg       = val;
        case 'SC.Cr',                  SC.Cr            = val;
        % --- attitude and geometry. Without these the panel models cannot be swept
        % at all, and they are exactly the toggles whose IMPACT you want to measure.
        %   SC.attitude : [] | 'ram' | 3x3 DCM | @(utc) | struct(.mjd,.q)
        %   SC.facets   : srp-format facet array (ONE set: drag reads .n/.A, SRP
        %                 reads the optics too). Two sets would be two spacecraft.
        case 'SC.attitude',            SC.attitude      = val;
        case 'SC.facets',              SC.facets        = val;
        case 'INTEG_METHOD',           IM               = val;
        case 'INTEG.rtol',             IN.rtol          = val;
        case 'INTEG.atol',             IN.atol          = val;
        otherwise
            error('validation:sweep_knob', ['unknown knob "%s".\nLegal:\n' ...
              '  atmosphere : FORCES.drag.atmos, FORCES.drag.corotate\n' ...
              '  models     : FORCES.drag.model (cannonball|sentman|dria|cll|sesam)\n' ...
              '               FORCES.srp.model  (cannonball|boxwing)\n' ...
              '               FORCES.erp.model  (knocke|simple|ceres|boxwing)\n' ...
              '               FORCES.erp.nrings, FORCES.erp.nseg\n' ...
              '  gas-surface: FORCES.drag.gsi.{aT,Tw,sig_n,sig_t}\n' ...
              '  gravity    : FORCES.gravity.degree, FORCES.gravity.model\n' ...
              '  on/off     : FORCES.{thirdbody,srp,erp,relativity,solidtides,oceantides}.on\n' ...
              '  spacecraft : SC.Cd, SC.Cr, SC.Aref (alias Aref_m2), SC.mass (alias mass_kg),\n' ...
              '               SC.attitude, SC.facets\n' ...
              '  integrator : INTEG_METHOD, INTEG.rtol, INTEG.atol'], knob);
    end
end
