function a = schwarzschild(rSat, vSat, mu, gamma, beta)
%RELATIVITY.SCHWARZSCHILD  Dominant GR correction from Earth's mass (IERS 2010).
%   a = (mu/(c^2 r^3))[ (2(beta+gamma)mu/r - gamma v^2) r + 2(1+gamma)(r.v) v ]
%   GR: gamma=beta=1. rSat,vSat ECI [m,m/s]; mu = GM_earth.
    if nargin<3||isempty(mu),    Kc=de440.constants(); mu=Kc.mu_earth; end   % no chained-call indexing (Octave / MATLAB<R2019b)
    if nargin<4||isempty(gamma), gamma=1; end
    if nargin<5||isempty(beta),  beta=1; end
    Kc=de440.constants(); c=Kc.c; r=rSat(:); v=vSat(:); rn=norm(r);
    a=mu/(c^2*rn^3)*( (2*(beta+gamma)*mu/rn - gamma*dot(v,v))*r + 2*(1+gamma)*dot(r,v)*v );
end
