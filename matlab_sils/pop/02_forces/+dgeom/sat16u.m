function [facets, opts] = sat16u()
%SAT16U  Example 16U-class low LEO bus + two side panels. DRAG geometry.
%
%   *** DEPRECATED -- use sgeom.sat16u for BOTH drag and SRP. ***
%
%   This function and sgeom.sat16u described TWO DIFFERENT SPACECRAFT:
%       dgeom (here) : buildBox(0.34, 0.20, 0.20)   -> long axis on BODY X
%       sgeom        : Lx=0.20, Ly=0.20, Lz=0.34    -> long axis on BODY Z
%   Same "16U", but drag saw a 0.34 x 0.20 ram face and SRP saw 0.20 x 0.20. With
%   +x defined as ram, the drag version pointed the LONG axis into the flow -- a
%   16U flying broadside. The frontal area differed by 1.7x and nothing complained,
%   because the two geometries were never compared: drag read one, SRP read the
%   other, and each was internally consistent.
%
%   That is the "two facet sets = two spacecraft" failure in its purest form, and it
%   is why validate_OD now builds ONE srp-format set for both forces (srp facets
%   carry .n/.A, which is all drag needs, PLUS the optics SRP needs).
%
%   Kept only so old scripts do not break. It now returns the SAME geometry as
%   sgeom.sat16u, stripped to what drag reads.
    warning('dgeom:sat16u:deprecated', ...
      ['dgeom.sat16u is deprecated: it described a different 16U than ' ...
       'sgeom.sat16u (long axis on X vs Z). Returning sgeom.sat16u''s geometry ' ...
       'so drag and SRP see ONE spacecraft. Use sgeom.sat16u directly.']);
    sc = sgeom.sat16u();
    facets = sc.facets;                           % srp format; drag reads .n/.A
    opts.mass = sc.mass;
    opts.Aref = 0.20*0.20;                        % frontal (ram) area [m^2]
    opts.Mmol = 16.0;                             % atomic-O dominated
end
