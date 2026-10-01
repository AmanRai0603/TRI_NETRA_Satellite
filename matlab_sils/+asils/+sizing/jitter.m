function th = jitter(rec, idx)
%ASILS.SIZING.JITTER  Pointing jitter from rotor imbalance, frequency domain [arcsec, each sample
%   of the window idx; the metric takes its statistic over them] (= the engine's metrics::jitter).
%   A rotor spinning at W with static imbalance U_s (kg m) at a lever d from the centre of mass and
%   dynamic imbalance U_d (kg m^2) applies a torque (U_s d + U_d) W^2 at W. Above the attitude
%   loop's bandwidth w_bw the body answers as a free rigid body, below it the loop rejects it:
%       theta = (U_s d + U_d) W^2 / (J_min max(W^2, w_bw^2))
%   (rigid body: no structural amplification). W = |h| / J from the recorded rotor momentum
%   (wheels, VSCMG and CMG rotors); fluid loops have no rotating mass. The rotors add root-sum-square.
%   d is half the class's smallest cross-section (the wheels sit inside the body), w_bw the
%   scenario's wheel-loop bandwidth (fsw.rw_bandwidth). Empty (not computed) when a rotor's part
%   does not state its imbalance.
    X = rec.P.dev.mex;
    if ~X.fitted, th = zeros(1, numel(idx)); return, end
    if any(isnan(X.Us) | isnan(X.Ud)), th = []; return, end
    Jmin = min(eig(rec.P.sc.I)); d = min(rec.P.sc.box_m)/2;
    wbw = asils.util.getf(rec.P.scenario.fsw, 'rw_bandwidth', 0.9);
    th2 = zeros(1, numel(idx));
    for i = 1:numel(X.kind)
        if strcmp(X.kind{i}, 'fmr') || (X.Us(i) == 0 && X.Ud(i) == 0), continue, end
        W = abs(rec.h_w(i, idx))/X.J(i); W(~isfinite(W)) = 0;
        t = (X.Us(i)*d + X.Ud(i))*W.^2./(Jmin*max(W.^2, wbw^2));
        th2 = th2 + t.^2;
    end
    th = sqrt(th2)*180/pi*3600;
end
