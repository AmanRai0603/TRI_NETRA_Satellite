function q = fromrotvec(phi)
%ASILS.QUAT.FROMROTVEC  Quaternion of a rotation vector phi [rad] (scalar-last).
    a = sqrt(phi'*phi);
    if a < 1e-12
        q = [0.5*phi; 1]; q = q/sqrt(q'*q);
    else
        q = [sin(a/2)*phi/a; cos(a/2)];
    end
end
