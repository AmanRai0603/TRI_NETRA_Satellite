function A = projected_area(facets, R_bi, vhat, sunHat_eci)
%VALIDATION.PROJECTED_AREA  A(t) = sum_k A_k * max(0, n_k . vhat).
%   The area the flow actually sees. R_bi maps BODY normals to inertial, so this is
%   where the attitude enters the drag: with R_bi frozen at eye(3) the body normals
%   sit on the inertial axes and A collapses -- to Aref if you are lucky, to ZERO if
%   the plate happens to face across the flow.
%
%   max(0,.) not abs(.): a facet turned BEHIND the flow is shadowed, not
%   double-counted. That is the difference between a box and a box-shaped sail.
%   SOLAR ARRAYS. A facet with .type=='array' PIVOTS to track the Sun, so its normal
%   is not the stored .n -- it is srp.arrayNormal(axis, sunHat_body). Pass sunHat_eci
%   and this follows them.
%
%   *** drag.force DOES NOT DO THIS. *** It reads facets(k).n for every facet,
%   including arrays (see drag/force.m: n = opts.R_bi*facets(k).n). So for a
%   satellite with Sun-tracking arrays, the drag model treats them as fixed panels
%   pointing wherever .n was last set. That is a real limitation, not a rounding
%   error: an array edge-on to the flow versus face-on is the difference between
%   ~0 and its full area. Bodies-only geometries (CHAMP, GRACE: body-mounted cells,
%   no deployed arrays) are unaffected. Flagged in CODEBASE_AUDIT.md.
%
%   Called WITHOUT sunHat_eci this reproduces drag.force's behaviour exactly, so the
%   diagnostic matches the force. Called WITH it, you see what the arrays are really
%   doing -- and the gap between the two is the size of the limitation above.
    if nargin < 4, sunHat_eci = []; end
    A = 0;
    vh = vhat(:)/norm(vhat);
    for k = 1:numel(facets)
        f = facets(k);
        nb = f.n(:);
        if ~isempty(sunHat_eci) && isfield(f,'type') && strcmp(f.type,'array')
            sHat_b = R_bi.' * (sunHat_eci(:)/norm(sunHat_eci));
            nb = srp.arrayNormal(f.axis, sHat_b);
        end
        n = R_bi*nb; n = n/norm(n);
        c = dot(n, vh);
        if isfield(f,'double') && f.double, c = abs(c); end
        A = A + f.A * max(0, c);
    end
end
