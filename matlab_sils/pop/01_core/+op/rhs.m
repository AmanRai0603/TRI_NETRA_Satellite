function dydt = rhs(t, y, W)
%OP.RHS  First-order state derivative for the Cowell propagator.
%   dydt = op.rhs(t, y, W),  y=[r;v] (6x1) -> [v; a(t,r,v)].
    r=y(1:3); v=y(4:6);
    dydt=[v; op.accel(t,r,v,W)];
end
