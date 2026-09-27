function dcs = elastic2(rMoon_ecef, rSun_ecef, k2, mu, Re)
%SOLIDTIDES.ELASTIC2  Basic degree-2 solid tide, single real Love number k2.
%   Simplest model (elastic Earth). dcs.dC/dS (5x5, only n=2 populated).
    if nargin<3||isempty(k2), k2=0.30; end
    c=de440.constants();
    if nargin<4||isempty(mu), mu=c.GM_earth; end
    if nargin<5||isempty(Re), Re=c.Re_earth; end
    GM=[c.GM_moon,c.GM_sun]; dC=zeros(5); dS=zeros(5);
    B={rMoon_ecef(:),rSun_ecef(:)};
    for b=1:2
        rb=B{b}; r=norm(rb); phi=asin(rb(3)/r); lam=atan2(rb(2),rb(1));
        P=solidtides.normLegendre(2,sin(phi));
        for m=0:2
            fac=k2/5*(GM(b)/mu)*(Re/r)^3*P(3,m+1);
            dC(3,m+1)=dC(3,m+1)+fac*cos(m*lam); dS(3,m+1)=dS(3,m+1)+fac*sin(m*lam);
        end
    end
    dcs.dC=dC; dcs.dS=dS;
end
