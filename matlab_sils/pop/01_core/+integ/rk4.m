function [T, Y, D] = rk4(f, t0, tf, y0, opts)
%INTEG.RK4  Classical fixed-step 4th-order Runge-Kutta (first-order system).
%   [T,Y,D] = integ.rk4(f, t0, tf, y0, opts)
%     f    : @(t,y) state derivative, y is 6x1 [r;v], returns 6x1 [v;a].
%     opts : .h fixed step [s] (required).
%   Returns node times T (Kx1), states Y (Kx6), derivatives D (Kx6).
%   Simple, robust, fully deterministic; good baseline / regression integrator.
    h = opts.h;  y = y0(:);
    N = max(1, round((tf-t0)/h));  h = (tf-t0)/N;         % land exactly on tf
    T = t0 + (0:N)'*h;   Y = zeros(N+1,numel(y0));  D = zeros(N+1,numel(y0));
    Y(1,:) = y.';  D(1,:) = f(t0,y).';
    for k = 1:N
        t = T(k);
        k1 = f(t,       y);
        k2 = f(t+h/2,   y+h/2*k1);
        k3 = f(t+h/2,   y+h/2*k2);
        k4 = f(t+h,     y+h*k3);
        y  = y + h/6*(k1+2*k2+2*k3+k4);
        Y(k+1,:) = y.';  D(k+1,:) = f(T(k+1),y).';
    end
end
