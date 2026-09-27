function q = norm(q)
%ASILS.QUAT.NORM  Unit quaternion with non-negative scalar part.
    q = q(:) / sqrt(q'*q);
    if q(4) < 0, q = -q; end
end
