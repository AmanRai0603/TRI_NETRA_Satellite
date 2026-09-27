function sol = run(method, f, t0, tf, y0, opts, tout)
%INTEG.RUN  Uniform driver: integrate with any method, sample at output times.
%   sol = integ.run(method, f, t0, tf, y0, opts[, tout])
%     method : 'rk4' | 'rk45' | 'rk78' | 'rk6luther' | 'nystrom4' | 'gaussJackson8'
%              | 'ode45' | 'ode78' | 'ode89' | 'ode113' (MATLAB ODE suite)
%     f      : @(t,y) first-order RHS, y=[r;v] (6x1)
%     opts   : integrator options (.h for fixed; .rtol/.atol/... for adaptive)
%     tout   : (optional) column of times at which to report the state; if
%              omitted, the raw integrator nodes are returned.
%   Returns sol with fields:
%     .t (Kx1), .y (Kx6), .r (Kx3), .v (Kx3), .nodes (raw node count),
%     .method, .nfev (if the method reports it via a persistent counter).
    fixed = {'rk4','rk6luther','nystrom4','gaussjackson8'};
    switch lower(method)
        case 'rk4',           fn=@integ.rk4;
        case 'rk45',          fn=@integ.rk45;
        case 'rk78',          fn=@integ.rk78;
        case 'rk6luther',     fn=@integ.rk6luther;
        case 'nystrom4',      fn=@integ.nystrom4;
        case 'gaussjackson8', fn=@integ.gaussJackson8;
        case 'ode45',         fn=@(f,a,b,y,o,tq) integ.odesuite('ode45', f,a,b,y,o,tq);
        case 'ode78',         fn=@(f,a,b,y,o,tq) integ.odesuite('ode78', f,a,b,y,o,tq);
        case 'ode89',         fn=@(f,a,b,y,o,tq) integ.odesuite('ode89', f,a,b,y,o,tq);
        case 'ode113',        fn=@(f,a,b,y,o,tq) integ.odesuite('ode113',f,a,b,y,o,tq);
        otherwise
            valid = 'rk4, rk45, rk78, rk6luther, nystrom4, gaussJackson8, ode45, ode78, ode89, ode113';
            error('integ:run:method','unknown integrator method "%s". Valid methods: %s', method, valid);
    end
    % Fixed-step methods need opts.h; if the caller gave adaptive opts (rtol/atol)
    % or nothing, supply a sensible default step so they don't crash on a missing
    % field. Adaptive methods (rk45/rk78) ignore .h.
    if any(strcmpi(method, fixed)) && (~isfield(opts,'h') || isempty(opts.h))
        opts.h = min(30, max(1, (tf-t0)/1000));
        warning('integ:run:defaultStep', ['fixed-step method "%s" had no opts.h; using h=%.3f s ' ...
            '(set cfg.integrator.h to control it, or use rk78/rk45 for adaptive stepping).'], method, opts.h);
    end
    isODE = any(strncmpi(method,'ode',3));
    if isODE
        tq = []; if nargin>=7, tq = tout; end
        [T,Y,D] = fn(f,t0,tf,y0,opts,tq);     % solver reports AT tout (native interp)
    else
        [T,Y,D] = fn(f,t0,tf,y0,opts);
    end
    if nargin>=7 && ~isempty(tout)
        tout=tout(:);
        Yq = integ.hermite(T,Y,D,tout);
        sol.t=tout; sol.y=Yq;
    else
        sol.t=T; sol.y=Y;
    end
    sol.r=sol.y(:,1:3); sol.v=sol.y(:,4:6);
    sol.nodes=numel(T); sol.method=method;
    sol.raw=struct('T',T,'Y',Y,'D',D);
end
