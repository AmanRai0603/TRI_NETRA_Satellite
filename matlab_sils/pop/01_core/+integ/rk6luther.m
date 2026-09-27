function [T, Y, D] = rk6luther(f, t0, tf, y0, opts)
%INTEG.RK6LUTHER  Luther 7-stage order-6 fixed-step RK (first-order system).
%   Tableau matches Hipparchus LutherIntegrator; higher order than RK4 for the
%   same step, useful when you want low truncation error without adaptivity.
%   opts.h : fixed step [s].
    q = sqrt(21);
    A = zeros(7);
    A(2,1)=1;  A(3,1:2)=[3/8,1/8];  A(4,1:3)=[8/27,2/27,8/27];
    A(5,1:4)=[3*(3*q-7)/392, -8*(7-q)/392, 48*(7-q)/392, -3*(21-q)/392];
    A(6,1:5)=[-5*(231+51*q)/1960, -40*(7+q)/1960, -320*q/1960, 3*(21+121*q)/1960, 392*(6+q)/1960];
    A(7,1:6)=[15*(22+7*q)/180, 120/180, 40*(7*q-5)/180, -63*(3*q-2)/180, -14*(49+9*q)/180, 70*(7-q)/180];
    c=[0;1;1/2;2/3;(7-q)/14;(7+q)/14;1];  b=[1/20;0;16/45;0;49/180;49/180;1/20];
    h=opts.h; y=y0(:);
    N=max(1,round((tf-t0)/h)); h=(tf-t0)/N;
    T=t0+(0:N)'*h; Y=zeros(N+1,numel(y0)); D=zeros(N+1,numel(y0));
    Y(1,:)=y.'; D(1,:)=f(t0,y).';
    K=zeros(numel(y0),7);
    for step=1:N
        t=T(step);
        for i=1:7
            yi=y;
            for j=1:i-1, yi=yi+h*A(i,j)*K(:,j); end
            K(:,i)=f(t+c(i)*h, yi);
        end
        y=y+h*(K*b);
        Y(step+1,:)=y.'; D(step+1,:)=f(T(step+1),y).';
    end
end
