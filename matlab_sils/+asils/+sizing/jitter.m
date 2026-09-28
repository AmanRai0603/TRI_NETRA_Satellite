function j = jitter(rec, idx)
%ASILS.SIZING.JITTER  Pointing jitter from rotor imbalance, frequency domain
%   [arcsec, p99.73 over the window idx].
%   A rotor spinning at W with static imbalance U_s (kg m) at a lever d from
%   the centre of mass and dynamic imbalance U_d (kg m^2) applies a torque
%   (U_s d + U_d) W^2 at W. Above the attitude-control bandwidth w_bw the body
%   answers as a free rigid body, below it the loop rejects it:
%       theta = (U_s d + U_d) W^2 / (J_min max(W^2, w_bw^2))
%   (rigid body: no structural amplification -- a flexible mode near W would
%   add its gain; case flex.fmode when stated). Rotor speed W = h/J from the
%   recorded rotor momentum (wheels, VSCMG and CMG rotors). Fluid loops have
%   no rotating mass: zero. The rotors are summed root-sum-square.
%   d = 0.05 m (half the 3U cross-section) unless the product gives it.
    j = 0;
    X = rec.P.dev.mex;
    if ~X.fitted || ~isfield(X, 'Us'), return, end
    Jmin = min(eig(rec.P.sc.I)); wbw = 1.0; d = 0.05;
    th2 = zeros(1, numel(idx));
    for i = 1:numel(X.kind)
        if strcmp(X.kind{i}, 'fmr') || (X.Us(i) == 0 && X.Ud(i) == 0), continue, end
        W = abs(rec.h_w(i, idx))/X.J(i); W(~isfinite(W)) = 0;
        th = (X.Us(i)*d + X.Ud(i))*W.^2./(Jmin*max(W.^2, wbw^2));
        th2 = th2 + th.^2;
    end
    th = sort(sqrt(th2))*180/pi*3600;
    if isempty(th), return, end
    j = th(max(1, ceil(0.9973*numel(th))));
end
