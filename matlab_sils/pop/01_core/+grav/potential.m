function U = potential(r, mu, Re, Cbar, Sbar, nmax)
%GRAV.POTENTIAL  Geopotential U(r) from fully-normalised coefficients.
%   U = grav.potential(r, mu, Re, Cbar, Sbar, nmax)
%   Returns the gravitational potential [m^2/s^2] with the convention a=+grad(U)
%   (so acceleration points toward the body for the central term).  Uses
%   fully-normalised associated Legendre functions via the standard forward
%   column recursion.  Provided mainly for gradient-consistency testing of
%   grav.sphericalHarmonic; not pole-safe (avoid exact geographic poles).
    x=r(1);y=r(2);z=r(3); rn=norm(r);
    phi=asin(z/rn); lam=atan2(y,x);
    sp=sin(phi); cp=cos(phi);
    P=normLegendre(sp,cp,nmax);
    cml=zeros(nmax+1,1); sml=zeros(nmax+1,1);
    for m=0:nmax, cml(m+1)=cos(m*lam); sml(m+1)=sin(m*lam); end
    U=0; rr=Re/rn;
    for n=0:nmax
        rn_pow=rr^n; inner=0;
        for m=0:n
            inner=inner+P(n+1,m+1)*(Cbar(n+1,m+1)*cml(m+1)+Sbar(n+1,m+1)*sml(m+1));
        end
        U=U+rn_pow*inner;
    end
    U=mu/rn*U;
end

function P=normLegendre(sp,cp,N)
% Fully-normalised associated Legendre functions Pbar(n,m) at sin(phi)=sp.
    P=zeros(N+1,N+1);
    P(1,1)=1;
    if N>=1
        P(2,1)=sqrt(3)*sp;
        P(2,2)=sqrt(3)*cp;
    end
    for n=2:N
        % sectorial
        P(n+1,n+1)=sqrt((2*n+1)/(2*n))*cp*P(n,n);
        for m=0:n-1
            a=sqrt((2*n+1)*(2*n-1)/((n-m)*(n+m)));
            b=sqrt((2*n+1)*(n+m-1)*(n-m-1)/((2*n-3)*(n-m)*(n+m)));
            if m==n-1, P(n+1,m+1)=a*sp*P(n,m+1);
            else,      P(n+1,m+1)=a*sp*P(n,m+1)-b*P(n-1,m+1); end
        end
    end
end
