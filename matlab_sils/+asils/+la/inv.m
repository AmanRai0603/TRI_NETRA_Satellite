function r = inv(m)
%ASILS.LA.INV  The inverse of a 3 x 3 matrix by its cofactors over its determinant (the toolbox's, la.rs inv).
%   Owner: Agastya. Copyright (c) 2026 Agastya. All rights reserved.
    d = m(1,1)*(m(2,2)*m(3,3) - m(2,3)*m(3,2)) - m(1,2)*(m(2,1)*m(3,3) - m(2,3)*m(3,1)) + m(1,3)*(m(2,1)*m(3,2) - m(2,2)*m(3,1));
    r = [(m(2,2)*m(3,3) - m(2,3)*m(3,2))/d, (m(1,3)*m(3,2) - m(1,2)*m(3,3))/d, (m(1,2)*m(2,3) - m(1,3)*m(2,2))/d;
         (m(2,3)*m(3,1) - m(2,1)*m(3,3))/d, (m(1,1)*m(3,3) - m(1,3)*m(3,1))/d, (m(1,3)*m(2,1) - m(1,1)*m(2,3))/d;
         (m(2,1)*m(3,2) - m(2,2)*m(3,1))/d, (m(1,2)*m(3,1) - m(1,1)*m(3,2))/d, (m(1,1)*m(2,2) - m(1,2)*m(2,1))/d];
end
