function a = accel(rSat, rBody, GM, model, nmax)
%THIRDBODY.ACCEL  Third-body acceleration with switchable model.
%   a = thirdbody.accel(rSat, rBody, GM[, model][, nmax])
%   model : 'battin' (default) | 'direct' | 'tidal' | 'legendre'
    if nargin < 4 || isempty(model), model = 'battin'; end
    switch lower(model)
        case 'battin',  a = thirdbody.battin(rSat, rBody, GM);
        case 'direct',  a = thirdbody.direct(rSat, rBody, GM);
        case 'tidal',   a = thirdbody.tidal(rSat, rBody, GM);
        case 'legendre'
            if nargin < 5 || isempty(nmax), nmax = 4; end
            a = thirdbody.legendre(rSat, rBody, GM, nmax);
        otherwise, error('thirdbody:accel','unknown model "%s"', model);
    end
end
