function q = conj(q)
%ASILS.QUAT.CONJ  Quaternion conjugate (scalar-last).
    q = [-q(1); -q(2); -q(3); q(4)];
end
