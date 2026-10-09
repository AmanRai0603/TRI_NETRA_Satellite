function s = spread_(dist, a, b)
%ASILS.SIZING.SPREAD_  A dispersion as the part files write it: the design's distribution (sizedemand's Dist: 0 normal,
%   1 uniform) and its two values.
%   Owner: Agastya. Copyright (c) 2026 Agastya. All rights reserved.
    if dist == 1
        s = struct('dist', 'uniform', 'lo', a, 'hi', b);
    else
        s = struct('dist', 'normal', 'mean', a, 'sigma', b);
    end
end
