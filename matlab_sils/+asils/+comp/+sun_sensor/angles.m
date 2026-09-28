function [s_head, ok] = angles(I, p)
%ASILS.COMP.SUN_SENSOR.ANGLES  Four quadrant currents -> Sun unit vector in the
%   head frame (BASELINE method, in-house algorithm to come): the normalised
%   differences give the spot displacement (exact for a square spot inside the
%   detector), tan(alpha) = (a/2) (I_x+ - I_x-)/sum / h, the same for beta.
%   ok when the total current exceeds p.min_frac of full Sun.
    tot = sum(I); ok = tot > p.min_frac; s_head = [0; 0; 1];
    if ~ok, return, end
    rx = ((I(1) + I(4)) - (I(2) + I(3)))/tot; ry = ((I(1) + I(2)) - (I(3) + I(4)))/tot;
    s_head = [p.a/2*rx/p.h; p.a/2*ry/p.h; 1]; s_head = s_head/norm(s_head);
end
