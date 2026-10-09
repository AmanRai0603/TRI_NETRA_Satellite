function B = body(inertia, geo, flex)
%ASILS.PLANT.BODY  The body the plant carries (adcs-sim-core plant.rs Body): the inertia, its inverse (the toolbox's,
%   asils.la.inv), the rotors' geometry (asils.models.rotors.rotor_geometry) and the flexible mode; with the mode on, the
%   inverse of the reduced inertia J - delta delta' (dyn_flexible_mode's flex_reduced_inertia) the body equation divides by.
%   Owner: Agastya. Copyright (c) 2026 Agastya. All rights reserved.
    B.i = inertia; B.iinv = asils.la.inv(inertia); B.m = geo;
    if nargin < 3 || isempty(flex) || ~flex.on
        B.flex = asils.models.flexmode.Flex_zero(); B.minv = B.iinv;
    else
        B.flex = flex; B.minv = asils.la.inv(asils.models.flexmode.flex_reduced_inertia(inertia, flex.delta));
    end
end
