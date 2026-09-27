function [T, Y, D] = odesuite(solverName, f, t0, tf, y0, opts, tout)
%INTEG.ODESUITE  Adapter for MATLAB's ODE-suite solvers into the [T,Y,D] format.
%   Wraps ode45 / ode78 / ode89 / ode113 (variable-order/step, MATLAB built-ins)
%   so they plug into integ.run and the Hermite dense-output resampler.
%     opts.rtol (1e-10) .atol (1e-12) .hmax (optional MaxStep)
%   ode78/ode89 require MATLAB R2021b+; ode45/ode113 are available everywhere.
    switch lower(solverName)
        case 'ode45',  solver=@ode45;   case 'ode78',  solver=@ode78;
        case 'ode89',  solver=@ode89;   case 'ode113', solver=@ode113;
        otherwise, error('integ:odesuite','unknown ODE solver "%s"', solverName);
    end
    if exist(func2str(solver),'file')~=2 && exist(func2str(solver),'builtin')~=5
        error('integ:odesuite:missing', ['%s is not available in this MATLAB ' ...
            '(ode78/ode89 need R2021b+). Use ode45, ode113, or rk78.'], func2str(solver));
    end
    rtol=getf(opts,'rtol',1e-10); atol=getf(opts,'atol',1e-12);
    o = odeset('RelTol',rtol,'AbsTol',atol);
    if isfield(opts,'hmax')&&~isempty(opts.hmax), o=odeset(o,'MaxStep',opts.hmax); end
    % IMPORTANT: give the solver the OUTPUT TIMES directly. MATLAB's ODE solvers
    % evaluate a vector tspan with their OWN high-order interpolant (the same one
    % deval uses), which is far better than re-interpolating their sparse nodes
    % with a cubic Hermite afterwards. ode113 in particular is a variable-order
    % Adams method that takes very long steps -- resampling its nodes with a cubic
    % would inject large error at exactly the times we report.
    if nargin>=7 && ~isempty(tout) && numel(tout)>=3
        tq = tout(:);
        if abs(tq(1)-t0)>1e-9, tq = [t0; tq]; end
        if abs(tq(end)-tf)>1e-9, tq = [tq; tf]; end
        [T, Y] = solver(@(t,y) f(t,y), tq, y0(:), o);   % native interpolant AT tq
        T = T(:);
    else
        sol = solver(@(t,y) f(t,y), [t0 tf], y0(:), o);
        T = sol.x(:); Y = sol.y.';
    end
    D = zeros(size(Y));                         % derivative at the reported times
    for k=1:numel(T), D(k,:) = f(T(k), Y(k,:).').'; end
end
function val=getf(s,f,d), if isfield(s,f)&&~isempty(s.(f)), val=s.(f); else, val=d; end, end
