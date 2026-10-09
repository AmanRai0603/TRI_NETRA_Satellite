function r = mv(m, v)
%ASILS.LA.MV  A 3 x 3 matrix times a 3-vector, each row summed left to right (the toolbox's, la.rs mv).
%   Owner: Agastya. Copyright (c) 2026 Agastya. All rights reserved.
    r = [m(1,1)*v(1) + m(1,2)*v(2) + m(1,3)*v(3); m(2,1)*v(1) + m(2,2)*v(2) + m(2,3)*v(3); m(3,1)*v(1) + m(3,2)*v(2) + m(3,3)*v(3)];
end
