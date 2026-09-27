function n = arrayNormal(axis, sHat_b)
%SRP.ARRAYNORMAL  Best-lighting normal of a solar array that pivots about AXIS.
%   The array rotates about AXIS (body frame) so its normal lies in the plane
%   spanned by AXIS and the Sun, as close to the Sun as the axis allows.
    axis = axis(:)/norm(axis);
    proj = sHat_b(:) - dot(sHat_b, axis)*axis;   % Sun projected onto plane _|_ axis
    if norm(proj) < 1e-9                          % Sun along axis: panel edge-on
        t = [1;0;0]; if abs(axis(1)) > 0.9, t = [0;1;0]; end
        proj = t - dot(t,axis)*axis;
    end
    n = proj/norm(proj);
end
