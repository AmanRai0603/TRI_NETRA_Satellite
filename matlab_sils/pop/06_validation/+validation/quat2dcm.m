function A = quat2dcm(q)
%VALIDATION.QUAT2DCM  Scalar-first quaternion -> rotation matrix.
%   A = validation.quat2dcm(q),  q = [q0 qx qy qz]  (Hamilton, scalar FIRST)
%
%   For the ITSG `attitude` product, q rotates SATELLITE -> CELESTIAL, so
%       a_celestial = A * a_satellite
%   and the inverse (celestial -> satellite) is A.' -- the transpose, not conj(q).
%
%   WARNING -- this differs from the Aerospace Toolbox `quat2dcm`, which returns
%   the TRANSPOSE of this (it maps inertial -> body). Do not swap one for the other.
%   The convention here is fixed by the ITSG product readme and by what makes the
%   accelerometer comparison in validate_OD come out at ratio 1.0.
%
%   No normalisation is done here: normalise before calling if the quaternion has
%   been interpolated.
    q0=q(1); q1=q(2); q2=q(3); q3=q(4);
    A = [1-2*(q2^2+q3^2), 2*(q1*q2-q0*q3), 2*(q1*q3+q0*q2); ...
         2*(q1*q2+q0*q3), 1-2*(q1^2+q3^2), 2*(q2*q3-q0*q1); ...
         2*(q1*q3-q0*q2), 2*(q2*q3+q0*q1), 1-2*(q1^2+q2^2)];
end
