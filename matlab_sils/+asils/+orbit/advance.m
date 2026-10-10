function O = advance(O)
%ASILS.ORBIT.ADVANCE  One RK4 step of the precision orbit's equations of motion, t0 -> t1 = t0 + h.
%   Uses the full force model (asils.orbit.accel) for every stage, so the truth orbit carries every force env states.
%   The node-1 acceleration and context are kept: they become node 0 of the next step (4 accel calls per step).
    h = O.h; t = O.t0; y = O.y0; W = O.W;
    k1 = [y(4:6); O.a0];
    y2 = y + 0.5*h*k1; k2 = [y2(4:6); asils.orbit.accel(W, t + 0.5*h, y2(1:3), y2(4:6))];
    y3 = y + 0.5*h*k2; k3 = [y3(4:6); asils.orbit.accel(W, t + 0.5*h, y3(1:3), y3(4:6))];
    y4 = y + h*k3;
    k4 = [y4(4:6); asils.orbit.accel(W, t + h, y4(1:3), y4(4:6))];
    O.t1 = t + h;
    O.y1 = y + h/6*(k1 + 2*k2 + 2*k3 + k4);
    [O.a1, O.x1] = asils.orbit.node(O.t1, O.y1, W);
end
