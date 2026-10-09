function r = mm(a, b)
%ASILS.LA.MM  The product of two 3 x 3 matrices, each element summed left to right (the toolbox's, la.rs mm).
%   Owner: Agastya. Copyright (c) 2026 Agastya. All rights reserved.
    r = zeros(3, 3);
    for i = 1:3
        for j = 1:3, r(i, j) = a(i, 1)*b(1, j) + a(i, 2)*b(2, j) + a(i, 3)*b(3, j); end
    end
end
