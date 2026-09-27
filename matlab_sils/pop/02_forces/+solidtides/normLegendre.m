function P = normLegendre(nmax, x)
%SOLIDTIDES.NORMLEGENDRE  Fully-normalized assoc. Legendre P̄_nm(x), n,m<=nmax.
%   Returns (nmax+1)x(nmax+1) lower-triangular matrix, geodesy 4pi-normalization.
    P=zeros(nmax+1); s=sqrt(1-x*x);
    P(1,1)=1;
    if nmax>=1, P(2,1)=sqrt(3)*x; P(2,2)=sqrt(3)*s; end
    for n=2:nmax
        for m=0:n
            i=n+1; j=m+1;
            if m==n
                P(i,j)=s*sqrt((2*n+1)/(2*n))*P(i-1,j-1);
            elseif m==n-1
                P(i,j)=x*sqrt(2*n+1)*P(i-1,j);
            else
                a=sqrt((2*n+1)*(2*n-1)/((n-m)*(n+m)));
                b=sqrt((2*n+1)*(n+m-1)*(n-m-1)/((2*n-3)*(n-m)*(n+m)));
                P(i,j)=a*x*P(i-1,j)-b*P(i-2,j);
            end
        end
    end
end
