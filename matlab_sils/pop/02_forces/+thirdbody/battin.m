function a = battin(rSat, rBody, GM)
%THIRDBODY.BATTIN  Numerically stable third-body acceleration (Battin's f(q)).
%   Mathematically identical to DIRECT but avoids the near-cancellation: it
%   never forms (rBody-rSat) and evaluates ((1+q)^{3/2}-1) in the stable form
%   q(3+3q+q^2)/(1+(1+q)^{3/2}). Full double precision for any rSat/rBody.
%   *** RECOMMENDED default *** (used by Orekit/GMAT-class tools).
    s = rSat(:); b = rBody(:);
    rb2 = dot(b,b);
    q   = dot(s, s - 2*b)/rb2;                    % = (|s|^2 - 2 s.b)/|b|^2
    f   = q*(3 + 3*q + q*q)/(1 + (1+q)^1.5);       % = (1+q)^{3/2} - 1, stably
    D3  = rb2*sqrt(rb2)*(1+q)^1.5;                 % = |rBody-rSat|^3, no differencing
    a   = -GM/D3 * (s + f*b);
end
