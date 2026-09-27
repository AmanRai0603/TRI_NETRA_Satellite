function out = propagate(a, ecc0, inc0, Om0, w0, perturbers, tspan, nsteps)
%THIRDBODY.SECULAR.PROPAGATE  Doubly-averaged (Lidov-Kozai) secular propagation.
%   Integrates the (j,e) vectors under the vectorial DA quadrupole for one or
%   more perturbers. RK4. Semi-major axis a is constant (secular quadrupole).
%
%   perturbers : struct array, each with .nhat (perturber pole unit vector, in
%                the element frame) and .phiQ (from thirdbody.secular.phiQuad).
%   tspan [s], nsteps. Returns t, e, inc, Om, w histories and the Kozai integral
%   sqrt(1-e^2) cos(i) (should stay constant for a single perturber).
    [j,e] = thirdbody.secular.elem2vectors(a, ecc0, inc0, Om0, w0);
    dt = tspan/nsteps;
    N = nsteps+1;
    out.t=zeros(N,1); out.e=zeros(N,1); out.inc=zeros(N,1);
    out.Om=zeros(N,1); out.w=zeros(N,1); out.kozai=zeros(N,1); out.a=a;
    for k = 1:N
        [ec,ic,Om,w] = thirdbody.secular.vectors2elem(j,e);
        out.t(k)=(k-1)*dt; out.e(k)=ec; out.inc(k)=ic; out.Om(k)=Om; out.w(k)=w;
        out.kozai(k)=sqrt(1-ec^2)*cos(ic);
        [a1,b1]=sumRates(j,e,perturbers);
        [a2,b2]=sumRates(j+0.5*dt*a1, e+0.5*dt*b1, perturbers);
        [a3,b3]=sumRates(j+0.5*dt*a2, e+0.5*dt*b2, perturbers);
        [a4,b4]=sumRates(j+dt*a3,     e+dt*b3,     perturbers);
        j = j + dt/6*(a1+2*a2+2*a3+a4);
        e = e + dt/6*(b1+2*b2+2*b3+b4);
    end
end

function [dj,de] = sumRates(j,e,P)
    dj=[0;0;0]; de=[0;0;0];
    for m = 1:numel(P)
        [d1,d2] = thirdbody.secular.kozaiRates(j,e,P(m).nhat,P(m).phiQ);
        dj=dj+d1; de=de+d2;
    end
end
