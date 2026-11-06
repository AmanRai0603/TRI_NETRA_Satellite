function margin_floor(lever, k)
%ASILS.SIZING.MARGIN_FLOOR  A momentum margin below the floor (design/sizedemand.pc, DR-design-003) is refused, by the name of
%   the lever that set it; the engine's adcs-design margin_floor.
%   Owner: Agastya. Copyright (c) 2026 Agastya. All rights reserved.
    assert(asils.models.sizedemand.margin_keeps_floor(k), 'asils:sizing:refused', ...
        '%s leaves the momentum margin at %g, below the floor %g (DR-design-003: the stated margin is the least the sizing keeps)', ...
        lever, k, asils.models.sizedemand.momentum_margin_floor());
end
