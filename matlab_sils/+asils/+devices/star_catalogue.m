function C = star_catalogue(n)
%ASILS.DEVICES.STAR_CATALOGUE  A deterministic synthetic star catalogue:
%   n unit vectors (ECI) on a Fibonacci sphere (uniform sky density, like the
%   ~4000 stars to magnitude 6) with magnitudes 1..6 drawn from a fixed
%   low-discrepancy sequence, so every run and every machine sees the same sky
%   and the run's random stream is not touched.
%   The lattice is jittered deterministically (0.35 of the mean spacing) and
%   the magnitudes come from an unrelated sequence: a bare Fibonacci lattice
%   is locally regular and point-symmetric, which no real sky is, and it makes
%   star identification ambiguous (asils.comp.star_tracker found 180 deg
%   flips on it).
    k = (0:n-1)' + 0.5;
    z = 1 - 2*k/n; ph = pi*(1 + sqrt(5))*k;
    rr = sqrt(1 - z.^2);
    r = [rr.*cos(ph), rr.*sin(ph), z];
    h1 = mod(sin(k*12.9898)*43758.5453, 1) - 0.5; h2 = mod(sin(k*78.233)*12345.6789, 1) - 0.5;
    e1 = [-sin(ph), cos(ph), zeros(n,1)];                       % local east
    e2 = cross(r, e1, 2);                                       % local north
    dsp = 0.35*sqrt(4*pi/n);
    r = r + dsp*(h1.*e1 + h2.*e2);
    C.r = (r./sqrt(sum(r.^2, 2)))';                             % 3 x n
    C.mag = 1 + 5*mod(k*sqrt(2) + 0.5*h1, 1)';                  % 1 x n
end
