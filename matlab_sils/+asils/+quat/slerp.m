function q = slerp(q0, q1, s)
%ASILS.QUAT.SLERP  Spherical interpolation q0 -> q1, s in [0,1]. Ported from quat.slerp.
    d = q0'*q1;
    if d < 0, q1 = -q1; d = -d; end
    if d > 0.9995
        q = q0 + s*(q1 - q0);
    else
        th = acos(d);
        q = (sin((1-s)*th)*q0 + sin(s*th)*q1)/sin(th);
    end
    q = q/sqrt(q'*q);
end
