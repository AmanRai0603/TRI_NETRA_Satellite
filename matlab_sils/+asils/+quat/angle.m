function th = angle(qa, qb)
%ASILS.QUAT.ANGLE  Rotation angle [rad] between two attitudes (0..pi).
    d = abs(qa(:)'*qb(:));
    th = 2*acos(min(1, d));
end
