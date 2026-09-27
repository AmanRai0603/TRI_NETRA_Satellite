function a = lenseThirring(rSat, vSat, mu, Jvec, gamma)
%RELATIVITY.LENSETHIRRING  Frame-dragging from Earth's rotation (IERS 2010).
%   a = (1+gamma)(mu/(c^2 r^3))[ (3/r^2)(r x v)(r.J) + (v x J) ]
%   Jvec = Earth angular momentum per unit mass [m^2/s], along spin axis (~+z).
    if nargin<3||isempty(mu), Kc=de440.constants(); mu=Kc.mu_earth; end   % no chained-call indexing (Octave / MATLAB<R2019b)
    if nargin<4||isempty(Jvec), Jvec=[0;0;9.8e8]; end
    if nargin<5||isempty(gamma), gamma=1; end
    Kc=de440.constants(); c=Kc.c; r=rSat(:); v=vSat(:); rn=norm(r); J=Jvec(:);
    a=(1+gamma)*mu/(c^2*rn^3)*( 3/rn^2*cross(r,v)*dot(r,J) + cross(v,J) );
end
