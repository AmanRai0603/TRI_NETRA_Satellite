function [T, Y, D] = nystrom4(f, t0, tf, y0, opts)
%INTEG.NYSTROM4  Nystrom-Lear 4(4) fixed-step integrator for r'' = a(t,r,v).
%   Second-order method: propagates position and velocity with one accel eval
%   per stage, exploiting the special structure of orbital dynamics.  Accepts
%   the same first-order f as the RK methods (acceleration is f(t,[r;v])(4:6)).
%   opts.h : fixed step [s].
    accel = @(t,r,v) getA(f(t,[r(:);v(:)]));
    h=opts.h;  y=y0(:);  r=y(1:3); v=y(4:6);
    N=max(1,round((tf-t0)/h)); h=(tf-t0)/N;
    T=t0+(0:N)'*h; Y=zeros(N+1,6); D=zeros(N+1,6);
    Y(1,:)=[r.' v.'];  D(1,:)=[v.' accel(t0,r,v).'];
    s5=sqrt(5); d2=(5-s5)/10; d3=(5+s5)/10;
    a1=(3-s5)/20; b2=(3+s5)/20; c1=(-1+s5)/4; c3=(3-s5)/4;
    ah1=(5-s5)/10; bh1=-(5+3*s5)/20; bh2=(3+s5)/4;
    ch1=-(1-5*s5)/4; ch2=-(5+3*s5)/4; ch3=(5-s5)/2;
    al1=1/12; al2=(5+s5)/24; al3=(5-s5)/24;
    be1=1/12; be2=5/12; be3=5/12; be4=1/12;
    for k=1:N
        t=T(k);
        k1=h*accel(t,r,v);
        r2=r+d2*h*v+h*(a1*k1);            v2=v+ah1*k1;               k2=h*accel(t+d2*h,r2,v2);
        r3=r+d3*h*v+h*(b2*k2);            v3=v+bh1*k1+bh2*k2;        k3=h*accel(t+d3*h,r3,v3);
        r4=r+h*v+h*(c1*k1+c3*k3);         v4=v+ch1*k1+ch2*k2+ch3*k3; k4=h*accel(t+h,r4,v4); %#ok<NASGU>
        r=r+h*v+h*(al1*k1+al2*k2+al3*k3);
        v=v+be1*k1+be2*k2+be3*k3+be4*k4;
        Y(k+1,:)=[r.' v.'];  D(k+1,:)=[v.' accel(T(k+1),r,v).'];
    end
end
function a=getA(dydt), a=dydt(4:6); end
