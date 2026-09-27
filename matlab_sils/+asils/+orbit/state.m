function [r, v, O] = state(O, t)
%ASILS.ORBIT.STATE  Truth position/velocity at time t [s after epoch], ECI.
%   Advances the in-loop propagator whenever t passes the current node, then
%   interpolates inside the step with cubic Hermite on (r,v) and (v,a) --
%   centimetre-level for h = 10 s in LEO.
    while t > O.t1 + 1e-9
        O.t0 = O.t1; O.y0 = O.y1; O.a0 = O.a1; O.x0 = O.x1;
        O = asils.orbit.advance(O);
    end
    h = O.t1 - O.t0; s = (t - O.t0)/h;
    s2 = s*s; s3 = s2*s;
    h00 = 2*s3 - 3*s2 + 1; h10 = s3 - 2*s2 + s; h01 = -2*s3 + 3*s2; h11 = s3 - s2;
    r = h00*O.y0(1:3) + h10*h*O.y0(4:6) + h01*O.y1(1:3) + h11*h*O.y1(4:6);
    v = h00*O.y0(4:6) + h10*h*O.a0 + h01*O.y1(4:6) + h11*h*O.a1;
end
