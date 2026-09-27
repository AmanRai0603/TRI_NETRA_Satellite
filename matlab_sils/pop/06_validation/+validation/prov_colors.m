function C = prov_colors()
%VALIDATION.PROV_COLORS  ONE colour vocabulary for provenance, shared by every plot.
%
%   C = validation.prov_colors()
%   C.measured / C.derived / C.fetched / C.assumed / C.missing  -> [r g b]
%   C.label.<class>                                             -> the words to print
%
%   ---------------------------------------------------------------------------
%   WHY A FUNCTION AND NOT A COLOUR TYPED INTO EACH PLOT
%   ---------------------------------------------------------------------------
%   Three figures now colour things by provenance (show_provenance, show_model_io,
%   show_OD). If each carries its own literal, they drift, and then "orange" means
%   ASSUMED in one figure and DERIVED in the next -- at which point the colour is
%   worse than no colour, because you will read it confidently and be wrong.
%
%   This is the same rule the rest of the audit has been enforcing on physics:
%   one quantity, one definition, one place. It applies to a legend too.
%
%   THE CLASSES, and why the distinction is not pedantry:
%
%     measured  A real instrument, THIS satellite, THIS epoch. ITSG accelerometer,
%               attitude, reduced-dynamic and kinematic orbits; TU Delft density.
%               This is the only class you cannot argue with.
%
%     derived   Computed BY the physics FROM measured inputs -- Cd(t) from the GSI
%               models, A(t) from geometry x measured attitude. Not a measurement
%               and not a guess. Calling it either would be a lie in one direction
%               or the other: it inherits the measurement's authority for the parts
%               that are measured and the model's uncertainty for the rest.
%
%     fetched   Somebody else's measurement, retrieved from an open source. F10.7
%               (OMNI2/NASA), Kp/ap (GFZ), JB2008 indices (SET), EOP (IERS), DE440
%               (JPL), gravity (ICGEM). Real data, but not OF this satellite -- a
%               3-hour Kp index is a planetary average standing in for the local
%               state above one spacecraft.
%
%     assumed   A number WE chose because nobody published one. Cr, GSI wall
%               temperature, facet optics, an aref_box shape, the F10.7 lag
%               convention. EVERY ONE OF THESE IS A KNOB. If a residual moves when
%               you sweep it, the residual was telling you about your assumption,
%               not about the atmosphere.
%
%     missing   Needed, absent, blocking a model. A DATA limit, not a code limit.
    C = struct();
    C.measured = [0.13 0.55 0.24];    % green  -- solid ground
    C.derived  = [0.45 0.72 0.35];    % pale green -- grown from solid ground
    C.fetched  = [0.20 0.45 0.80];    % blue   -- real, but someone else's
    C.assumed  = [0.90 0.60 0.10];    % orange -- YOUR CHOICE. a knob.
    C.missing  = [0.80 0.20 0.20];    % red    -- absent

    C.label = struct( ...
        'measured', 'MEASURED (this sat, this epoch)', ...
        'derived',  'DERIVED (physics, from measured)', ...
        'fetched',  'FETCHED (open source, not this sat)', ...
        'assumed',  'ASSUMED (we chose it -- a KNOB)', ...
        'missing',  'MISSING (blocks a model)');

    C.order = {'measured','derived','fetched','assumed','missing'};
end
