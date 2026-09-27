function C = star_catalogue(n)
%ASILS.DEVICES.STAR_CATALOGUE  A deterministic synthetic star catalogue:
%   n unit vectors (ECI) on a Fibonacci sphere (uniform sky density, like the
%   ~4000 stars to magnitude 6) with magnitudes 1..6 drawn from a fixed
%   low-discrepancy sequence, so every run and every machine sees the same sky
%   and the run's random stream is not touched.
    k = (0:n-1)' + 0.5;
    z = 1 - 2*k/n; ph = pi*(1 + sqrt(5))*k;
    rr = sqrt(1 - z.^2);
    C.r = [rr.*cos(ph), rr.*sin(ph), z]';           % 3 x n
    C.mag = 1 + 5*mod(k*0.6180339887, 1)';          % 1 x n
end
