function [T, Y, D] = rk45(f, t0, tf, y0, opts)
%INTEG.RK45  Adaptive Dormand-Prince 5(4) (the classic RKDP / ode45 core).
%   [T,Y,D] = integ.rk45(f, t0, tf, y0, opts)
%   opts (all optional): .rtol (1e-9) .atol (1e-12) .h0 .hmax .hmin .maxsteps
%   Returns the accepted-step nodes; use integ.hermite / integ.run to resample
%   onto arbitrary output times.  FSAL (k7 reused), PI-free 0.9 safety control.
    o = integ.adopt(opts, t0, tf);
    y = y0(:);  t = t0;  h = o.h0;
    T=t; Y=y.'; D=f(t,y).';
    k1 = f(t,y);
    if ~all(isfinite(k1)), error('integ:rk45:nonfinite','dynamics non-finite at t0 (NaN/Inf in initial acceleration)'); end
    steps=0; iter=0;
    while t < tf - 1e-9*max(1,abs(tf))
        iter=iter+1; if iter>o.maxsteps, error('integ:rk45:maxsteps','exceeded %d iterations near t=%.3f s (repeated step rejection -- NaN/Inf or tolerances too tight)', o.maxsteps, t); end
        if t+h > tf, h = tf-t; end
        [ynew, yerr, k7] = dp54_step(f,t,y,h,k1);
        if ~all(isfinite(ynew)), error('integ:rk45:nonfinite','state became non-finite at t=%.3f s (NaN/Inf in acceleration -- e.g. zero mass, singular geometry, bad density/gravity)', t); end
        sc  = o.atol + o.rtol*max(abs(y),abs(ynew));
        err = sqrt(mean((yerr./sc).^2));
        if err <= 1
            t=t+h; y=ynew; k1=k7;                 % FSAL
            T(end+1,1)=t; Y(end+1,:)=y.'; D(end+1,:)=f(t,y).'; %#ok
            steps=steps+1;
        end
        if err==0, fac=o.facmax; else, fac=min(o.facmax, max(o.facmin, 0.9*err^(-1/5))); end
        h = min(o.hmax, max(o.hmin, h*fac));
        if h<=o.hmin*1.0000001 && t<tf, h=o.hmin; end
        if steps>o.maxsteps, warning('integ:rk45:maxsteps','max steps hit'); break; end
    end
end
function [yn,ye,k7]=dp54_step(f,t,y,h,k1)
    k2=f(t+h/5,      y+h*(1/5*k1));
    k3=f(t+3*h/10,   y+h*(3/40*k1+9/40*k2));
    k4=f(t+4*h/5,    y+h*(44/45*k1-56/15*k2+32/9*k3));
    k5=f(t+8*h/9,    y+h*(19372/6561*k1-25360/2187*k2+64448/6561*k3-212/729*k4));
    k6=f(t+h,        y+h*(9017/3168*k1-355/33*k2+46732/5247*k3+49/176*k4-5103/18656*k5));
    yn=y+h*(35/384*k1+500/1113*k3+125/192*k4-2187/6784*k5+11/84*k6);
    k7=f(t+h,yn);
    ye=h*(71/57600*k1-71/16695*k3+71/1920*k4-17253/339200*k5+22/525*k6-1/40*k7);
end
