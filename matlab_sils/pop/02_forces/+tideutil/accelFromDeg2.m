function a = accelFromDeg2(rSat_ecef, dcs, mu, Re)
%TIDEUTIL.ACCELFROMDEG2  Acceleration perturbation from degree-2 ΔC̄2m/ΔS̄2m,
%   so you can see the IMPACT of a tide model on the orbit (ECEF).
%   Gradient of the degree-2 tesseral potential with the coefficient deltas.
    if nargin<3||isempty(mu), c=de440.constants(); mu=c.GM_earth; Re=c.Re_earth;
    elseif nargin<4, Kc=de440.constants(); Re=Kc.Re_earth; end   % no chained-call indexing
    r=rSat_ecef(:); rn=norm(r);
    % numerical gradient of U2 = mu/r * sum_{m} (Re/r)^2 P̄2m(sinφ)(ΔC cos mλ+ΔS sin mλ)
    h=1.0;  a=zeros(3,1);
    for k=1:3
        rp=r; rm=r; rp(k)=rp(k)+h; rm(k)=rm(k)-h;
        a(k)=(U2(rp)-U2(rm))/(2*h);
    end
    function u=U2(x)
        xr=norm(x); phi=asin(x(3)/xr); lam=atan2(x(2),x(1));
        P=solidtides.normLegendre(2,sin(phi)); u=0;
        for m=0:2
            u=u+mu/xr*(Re/xr)^2*P(3,m+1)*(dcs.dC(3,m+1)*cos(m*lam)+dcs.dS(3,m+1)*sin(m*lam));
        end
    end
end
