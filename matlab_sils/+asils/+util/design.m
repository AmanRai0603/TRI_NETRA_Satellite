function D = design(new)
%ASILS.UTIL.DESIGN  The design database the twin reads its inputs from (docs/S7_INVENTORY.md S7.18), as the engine's
%   adcs-sim source.rs does: with one in use, every input under data/ and cases/ (the scenarios, products, parts,
%   algorithms, the catalogue, the classes, the stated values, the cases) is the design's, never the folder's, and a
%   path the design does not hold is refused by name, so a run cannot quietly mix the two. The engine is asked for the
%   flight software's blob with the same design (TRINETRA_DESIGN). With none, the folder's files (matlab_sils/data,
%   matlab_sils/cases: tools/from_design.py's export), as before.
%
%   D = asils.util.design()      the design in use, or [] (the folder's files)
%   asils.util.design(D)         use D: trinetra.open's (D.file, D.inputs: path -> text, D.fingerprint, D.generated)
%   asils.util.design([])        back to the folder's files
%   trinetra.run and trinetra.campaign set it for their run and put back what was in use before.
%   Owner: Agastya. Copyright (c) 2026 Agastya. All rights reserved.
    persistent cur
    if nargin
        if ~isempty(new)
            assert(isstruct(new) && isfield(new, 'file') && isfield(new, 'inputs'), 'asils:design:what', ...
                'asils.util.design takes what trinetra.open gives (a design''s file and inputs), or []');
        end
        cur = new;
    end
    D = cur;
end
