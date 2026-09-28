function I = currents(s_head, p)
%ASILS.COMP.SUN_SENSOR.CURRENTS  MODEL SIDE: a square aperture (side p.a) at
%   height p.h over a four-quadrant photodiode. The Sun (unit vector in the head
%   frame, z = boresight) casts a square spot displaced by h tan(alpha),
%   h tan(beta); each quadrant's current is its share of the spot area times
%   cos(incidence), plus noise (p.noise, fraction of the full-Sun current).
%   I = [++; -+; --; +-] (quadrants in x, y).
    if s_head(3) <= 0, I = zeros(4,1); return, end
    dx = p.h*s_head(1)/s_head(3); dy = p.h*s_head(2)/s_head(3);
    ov = @(lo, hi, a, b) max(0, min(hi, b) - max(lo, a));        % overlap of [lo,hi] with [a,b]
    xp = ov(0, Inf, dx - p.a/2, dx + p.a/2); xm = ov(-Inf, 0, dx - p.a/2, dx + p.a/2);
    yp = ov(0, Inf, dy - p.a/2, dy + p.a/2); ym = ov(-Inf, 0, dy - p.a/2, dy + p.a/2);
    A = [xp*yp; xm*yp; xm*ym; xp*ym]/p.a^2;
    I = s_head(3)*A + p.noise*randn(4,1);
end
