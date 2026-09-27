function dcs = iers2010(rMoon_ecef, rSun_ecef, mu, Re)
%SOLIDTIDES.IERS2010  Solid Earth tide geopotential corrections (IERS 2010 step 1,
%   frequency-independent, anelastic nominal Love numbers). Degrees 2,3 plus
%   degree-4 induced by degree-2. Bodies must be in the EARTH-FIXED (ITRF) frame.
%
%   dcs.dC, dcs.dS : (5x5) normalized coefficient corrections (n,m -> index n+1,m+1).
%
%   ΔC̄nm − iΔS̄nm = (k_nm/(2n+1)) Σ_j (GM_j/GM)(Re/r_j)^{n+1} P̄nm(sinφ_j) e^{−imλ_j}
    if nargin<3||isempty(mu), c=de440.constants(); mu=c.GM_earth; Re=c.Re_earth;
    elseif nargin<4, Kc=de440.constants(); Re=Kc.Re_earth; end   % no chained-call indexing
    c=de440.constants(); GMm=c.GM_moon; GMs=c.GM_sun;
    % nominal Love numbers (IERS 2010 Table 6.3)
    k=zeros(4); k(3,1)=0.29525;k(3,2)=0.29470;k(3,3)=0.29801;           % n=2 (index 3)
    k(4,1)=0.093;k(4,2)=0.093;k(4,3)=0.093;k(4,4)=0.094;                % n=3
    kp=[-0.00087,-0.00079,-0.00057];                                    % k2+_m (deg4)
    dC=zeros(5); dS=zeros(5);
    for body=1:2
        if body==1, rb=rMoon_ecef(:); GMb=GMm; else, rb=rSun_ecef(:); GMb=GMs; end
        r=norm(rb); phi=asin(rb(3)/r); lam=atan2(rb(2),rb(1));
        P=solidtides.normLegendre(4, sin(phi));
        for n=2:3
            for m=0:n
                fac=k(n+1,m+1)/(2*n+1)*(GMb/mu)*(Re/r)^(n+1)*P(n+1,m+1);
                dC(n+1,m+1)=dC(n+1,m+1)+fac*cos(m*lam);
                dS(n+1,m+1)=dS(n+1,m+1)+fac*sin(m*lam);
            end
        end
        % degree-4 from degree-2 tides
        for m=0:2
            fac=kp(m+1)/5*(GMb/mu)*(Re/r)^3*P(3,m+1);
            dC(5,m+1)=dC(5,m+1)+fac*cos(m*lam);
            dS(5,m+1)=dS(5,m+1)+fac*sin(m*lam);
        end
    end
    dcs.dC=dC; dcs.dS=dS;
end
