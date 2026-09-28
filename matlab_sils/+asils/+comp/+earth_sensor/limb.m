function L = limb(nadir_head, rho, p)
%ASILS.COMP.EARTH_SENSOR.LIMB  MODEL SIDE: the horizon directions the IR
%   detector array sees -- p.n points on the Earth-limb cone (half-angle rho =
%   asin(R_E/r)) around the true nadir, at evenly spaced azimuths, kept where
%   they fall inside the field (p.fov around the head z), with the horizon's
%   radiance-profile noise (p.noise, rad) on each.
    n = nadir_head/norm(nadir_head);
    e1 = cross(n, [0; 0; 1]); if norm(e1) < 1e-6, e1 = cross(n, [1; 0; 0]); end
    e1 = e1/norm(e1); e2 = cross(n, e1);
    L = zeros(3, 0);
    for az = (0:p.n-1)*2*pi/p.n
        l = cos(rho)*n + sin(rho)*(cos(az)*e1 + sin(az)*e2);
        if acos(l(3)) > p.fov, continue, end
        l = asils.quat.dcm(asils.quat.fromrotvec(p.noise*randn(3,1)))*l;
        L(:, end+1) = l/norm(l); %#ok<AGROW>
    end
end
