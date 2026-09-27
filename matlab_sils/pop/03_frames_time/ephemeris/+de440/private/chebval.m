function [val, der] = chebval(c, x, radius)
%CHEBVAL  Chebyshev series value and time-derivative at x in [-1,1].
%   der is divided by RADIUS so it is per-second when RADIUS is the record
%   half-length in seconds (Type 2 SPK convention).
    n = numel(c);
    T = zeros(n,1); T(1) = 1;
    if n > 1; T(2) = x; end
    for i = 3:n; T(i) = 2*x*T(i-1) - T(i-2); end
    val = c.' * T;

    dT = zeros(n,1);
    if n > 1; dT(2) = 1; end
    if n > 2; dT(3) = 4*x; end
    for i = 4:n; dT(i) = 2*x*dT(i-1) + 2*T(i-1) - dT(i-2); end
    der = (c.' * dT) / radius;
end
