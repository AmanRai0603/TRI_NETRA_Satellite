function s = scale_(k, p)
%ASILS.SIZING.SCALE_  A part's authority scale: the loop's, or 1 (the law's size) where it sets none.
%   Owner: Agastya. Copyright (c) 2026 Agastya. All rights reserved.
    s = 1.0;
    if isfield(k.scale, p), s = k.scale.(p); end
end
