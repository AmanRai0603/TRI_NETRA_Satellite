function [a,e,inc,RAAN,argp,nu] = rv2coe(r, v, mu)
%OP.RV2COE  Classical Keplerian elements from an ECI state vector.
%   [a,e,inc,RAAN,argp,nu] = op.rv2coe(r, v, mu)
%   Angles in radians.  Handles circular/equatorial edge cases by convention
%   (undefined angles set to 0).  Standard Vallado algorithm.
    r=r(:); v=v(:); rn=norm(r); vn=norm(v);
    h=cross(r,v); hn=norm(h);
    K=[0;0;1]; nvec=cross(K,h); nn=norm(nvec);
    evec=((vn^2-mu/rn)*r - dot(r,v)*v)/mu;  e=norm(evec);
    xi=vn^2/2 - mu/rn;
    if abs(e-1)>1e-10, a=-mu/(2*xi); else, a=Inf; end
    inc=acos(max(-1,min(1,h(3)/hn)));
    if nn>1e-12
        RAAN=acos(max(-1,min(1,nvec(1)/nn))); if nvec(2)<0, RAAN=2*pi-RAAN; end
    else, RAAN=0; end
    if nn>1e-12 && e>1e-12
        argp=acos(max(-1,min(1,dot(nvec,evec)/(nn*e)))); if evec(3)<0, argp=2*pi-argp; end
    else, argp=0; end
    if e>1e-12
        nu=acos(max(-1,min(1,dot(evec,r)/(e*rn)))); if dot(r,v)<0, nu=2*pi-nu; end
    else  % circular: use argument of latitude
        nu=acos(max(-1,min(1,dot(nvec,r)/(max(nn,eps)*rn)))); if r(3)<0, nu=2*pi-nu; end
    end
end
