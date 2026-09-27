function a = legendre(rSat, rBody, GM, nmax)
%THIRDBODY.LEGENDRE  Third-body acceleration via Legendre series to degree NMAX.
%   Gradient of R = (GM/r_b) * sum_{n>=2} (rho/r_b)^n P_n(cos psi).
%   n=2 reproduces TIDAL; increasing NMAX converges to BATTIN. Use to SEE the
%   convergence / choose a truncation. NMAX default 4.
    if nargin < 4 || isempty(nmax), nmax = 4; end
    s = rSat(:); b = rBody(:); rho = norm(s); rb = norm(b);
    rs = s/rho; rbh = b/rb; u = dot(rs, rbh);           % u = cos(psi)
    P = zeros(nmax+1,1); dP = zeros(nmax+1,1);           % P(k)=P_{k-1}
    P(1) = 1; if nmax>=1, P(2) = u; dP(2) = 1; end
    for n = 2:nmax
        P(n+1)  = ((2*n-1)*u*P(n) - (n-1)*P(n-1))/n;     % Legendre recursion
        dP(n+1) = u*dP(n) + n*P(n);                       % derivative recursion
    end
    a = zeros(3,1);
    for n = 2:nmax
        a = a + GM/rb^(n+1) * rho^(n-1) * ( n*P(n+1)*rs + dP(n+1)*(rbh - u*rs) );
    end
end
