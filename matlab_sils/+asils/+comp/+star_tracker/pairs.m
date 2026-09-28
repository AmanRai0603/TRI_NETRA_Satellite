function K = pairs(cat, fov_half_rad)
%ASILS.COMP.STAR_TRACKER.PAIRS  Onboard pair table for identification: every
%   catalogue pair closer than the field diagonal, sorted by angle.
    c = cat.r'*cat.r; n = size(cat.r, 2);
    [i, j] = find(triu(c > cos(2*sqrt(2)*fov_half_rad), 1));
    a = acos(min(1, c(sub2ind([n n], i, j))));
    [a, o] = sort(a);
    K = struct('i', i(o), 'j', j(o), 'ang', a, 'mag', cat.mag, 'r', cat.r);
end
