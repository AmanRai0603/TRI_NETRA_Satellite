function [n, ok] = horizon(L, rho)
%ASILS.COMP.EARTH_SENSOR.HORIZON  Limb directions -> nadir unit vector
%   (BASELINE method, in-house algorithm to come): the limb is a cone of known
%   half-angle rho (from the onboard altitude) about nadir, so l_i . n = cos(rho)
%   for every limb point -- a linear least-squares problem in n, normalised.
%   ok with three or more points.
    ok = size(L, 2) >= 3; n = [0; 0; 1];
    if ~ok, return, end
    n = (L')\(cos(rho)*ones(size(L, 2), 1));
    n = n/norm(n);
end
