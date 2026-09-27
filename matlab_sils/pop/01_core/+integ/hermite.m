function Yq = hermite(T, Y, D, tq)
%INTEG.HERMITE  Piecewise cubic Hermite dense output from integrator nodes.
%   Yq = integ.hermite(T, Y, D, tq)
%     T (Nx1) node times, Y (Nx6) node states, D (Nx6) node derivatives
%     (D = dY/dt at the nodes, i.e. [v;a]); tq (Mx1) query times.
%   Uses value+slope at both ends of each interval -> 3rd-order accurate dense
%   output, exact at the nodes.  This lets ANY integrator return r,v at
%   arbitrary requested times (the "give me the state at time t" interface).
    T=T(:); tq=tq(:); M=numel(tq); ny=size(Y,2); Yq=zeros(M,ny);
    idx = min(max(discretize_(tq,T),1),numel(T)-1);
    for k=1:M
        i=idx(k); h=T(i+1)-T(i); s=(tq(k)-T(i))/h;
        h00=2*s^3-3*s^2+1; h10=s^3-2*s^2+s; h01=-2*s^3+3*s^2; h11=s^3-s^2;
        Yq(k,:)=h00*Y(i,:)+h10*h*D(i,:)+h01*Y(i+1,:)+h11*h*D(i+1,:);
    end
end
function b=discretize_(x,edges)
    b=zeros(numel(x),1);
    for j=1:numel(x)
        ii=find(edges<=x(j),1,'last'); if isempty(ii), ii=1; end
        b(j)=ii;
    end
end
