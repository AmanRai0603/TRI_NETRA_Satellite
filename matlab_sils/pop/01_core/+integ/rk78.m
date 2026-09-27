function [T, Y, D] = rk78(f, t0, tf, y0, opts)
%INTEG.RK78  Adaptive Dormand-Prince 8(7) (Prince & Dormand 1981).
%   High-order adaptive integrator -- the recommended workhorse for precise
%   orbit propagation: near machine-limited accuracy at long arcs with far
%   fewer steps than RK45.  Same opts/returns as integ.rk45.
    o = integ.adopt(opts, t0, tf);
    [c,A,b8,b7] = dp87_tableau();
    y=y0(:); t=t0; h=o.h0;
    d0=f(t,y); if ~all(isfinite(d0)), error('integ:rk78:nonfinite','dynamics non-finite at t0 (NaN/Inf in the initial acceleration -- check spacecraft mass/area, config, or state)'); end
    T=t; Y=y.'; D=d0.';
    steps=0; iter=0;
    while t < tf - 1e-9*max(1,abs(tf))
        iter=iter+1;
        if iter>o.maxsteps
            error('integ:rk78:maxsteps',['exceeded %d iterations near t=%.3f s. The step is being rejected ' ...
                  'repeatedly -- usually a NaN/Inf or a discontinuity in the dynamics, or tolerances set too tight.'], o.maxsteps, t);
        end
        if t+h>tf, h=tf-t; end
        [y8,y7]=dp87_step(f,t,y,h,c,A,b8,b7);
        if ~all(isfinite(y8))
            error('integ:rk78:nonfinite',['state became non-finite at t=%.3f s (NaN/Inf in the acceleration). ' ...
                  'Common causes: zero/absent spacecraft mass, singular geometry, or a bad density/gravity input.'], t);
        end
        sc=o.atol+o.rtol*max(abs(y),abs(y8));
        err=sqrt(mean(((y8-y7)./sc).^2));
        if err<=1
            dnew=f(t+h,y8); if ~all(isfinite(dnew)), error('integ:rk78:nonfinite','dynamics non-finite at t=%.3f s',t+h); end
            t=t+h; y=y8;
            T(end+1,1)=t; Y(end+1,:)=y.'; D(end+1,:)=dnew.'; %#ok
            steps=steps+1;
        end
        if err==0, fac=o.facmax; else, fac=min(o.facmax,max(o.facmin,0.9*err^(-1/8))); end
        h=min(o.hmax,max(o.hmin,h*fac));
    end
end
function [x8,x7]=dp87_step(f,t,x,h,c,A,b8,b7)
    F=x*zeros(1,13); F(:,1)=f(t,x);
    for j=1:12, F(:,j+1)=f(t+c(j)*h, x+h*F*A(:,j)); end
    x8=x+h*F*b8; x7=x+h*F*b7;
end
function [c,a,b8,b7]=dp87_tableau()
    c=[1/18,1/12,1/8,5/16,3/8,59/400,93/200,5490023248/9719169821,13/20,1201146811/1299019798,1,1]';
    a=[1/18,0,0,0,0,0,0,0,0,0,0,0,0;
       1/48,1/16,0,0,0,0,0,0,0,0,0,0,0;
       1/32,0,3/32,0,0,0,0,0,0,0,0,0,0;
       5/16,0,-75/64,75/64,0,0,0,0,0,0,0,0,0;
       3/80,0,0,3/16,3/20,0,0,0,0,0,0,0,0;
       29443841/614563906,0,0,77736538/692538347,-28693883/1125000000,23124283/1800000000,0,0,0,0,0,0,0;
       16016141/946692911,0,0,61564180/158732637,22789713/633445777,545815736/2771057229,-180193667/1043307555,0,0,0,0,0,0;
       39632708/573591083,0,0,-433636366/683701615,-421739975/2616292301,100302831/723423059,790204164/839813087,800635310/3783071287,0,0,0,0,0;
       246121993/1340847787,0,0,-37695042795/15268766246,-309121744/1061227803,-12992083/490766935,6005943493/2108947869,393006217/1396673457,123872331/1001029789,0,0,0,0;
       -1028468189/846180014,0,0,8478235783/508512852,1311729495/1432422823,-10304129995/1701304382,-48777925059/3047939560,15336726248/1032824649,-45442868181/3398467696,3065993473/597172653,0,0,0;
       185892177/718116043,0,0,-3185094517/667107341,-477755414/1098053517,-703635378/230739211,5731566787/1027545527,5232866602/850066563,-4093664535/808688257,3962137247/1805957418,65686358/487910083,0,0;
       403863854/491063109,0,0,-5068492393/434740067,-411421997/543043805,652783627/914296604,11173962825/925320556,-13158990841/6184727034,3936647629/1978049680,-160528059/685178525,248638103/1413531060,0,0]';
    b8=[14005451/335480064,0,0,0,0,-59238493/1068277825,181606767/758867731,561292985/797845732,-1041891430/1371343529,760417239/1151165299,118820643/751138087,-528747749/2220607170,1/4]';
    b7=[13451932/455176623,0,0,0,0,-808719846/976000145,1757004468/5645159321,656045339/265891186,-3867574721/1518517206,465885868/322736535,53011238/667516719,2/45,0]';
end
