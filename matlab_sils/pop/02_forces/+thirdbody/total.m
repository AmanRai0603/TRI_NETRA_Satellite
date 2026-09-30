function [a, parts] = total(rSat, E, model, nmax)
%THIRDBODY.TOTAL  Sum of Sun + Moon third-body accelerations (the pair that
%   matters at Earth orbit; planets are ~1e-13 m/s^2, negligible at LEO).
%   a = thirdbody.total(rSat, E[, model][, nmax])   E = ephemInputs struct.
    if nargin < 3 || isempty(model), model = 'battin'; end
    if nargin < 4, nmax = 4; end
    parts.sun  = thirdbody.accel(rSat, E.sun_eci,  E.GM_sun,  model, nmax);
    parts.moon = thirdbody.accel(rSat, E.moon_eci, E.GM_moon, model, nmax);
    a = parts.sun + parts.moon;
end
