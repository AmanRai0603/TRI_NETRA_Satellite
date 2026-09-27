function [dRTN, rms] = rtn_residual(sol, ref)
%VALIDATION.RTN_RESIDUAL  Position residual of a propagation vs a reference, in RTN.
%   [dRTN, rms] = validation.rtn_residual(sol, ref)
%   Interpolates the propagation `sol` to the reference epochs (handling the epoch
%   offset between them) and returns radial/along/cross residuals + RMS. Lets one
%   propagation be compared against several references (e.g. SP3 AND TU Delft).
    solEpoch = sol.cfg.epoch;
    off = (datenum(ref.epoch) - datenum(solEpoch))*86400;   % ref start rel. to sol start [s]
    n = numel(ref.t); dRTN = zeros(n,3); good = true(n,1);
    for k=1:n
        ts = off + ref.t(k);
        if ts < sol.t(1)-1e-6 || ts > sol.t(end)+1e-6, good(k)=false; continue; end
        rv = sol.stateAt(ts); rv = rv(:);            % force column [r;v] (6x1)
        rp = rv(1:3);
        rr = ref.r(k,:).'; vv = ref.v(k,:).';        % 3x1 columns
        dr = rp - rr;
        R = rr/norm(rr); Wn = cross(rr,vv); Wn = Wn/norm(Wn); Tt = cross(Wn,R);
        dRTN(k,:) = [dr.'*R, dr.'*Tt, dr.'*Wn];
    end
    dRTN = dRTN(good,:);
    f = @(x) sqrt(mean(x.^2));
    rms = struct('radial',f(dRTN(:,1)),'along',f(dRTN(:,2)),'cross',f(dRTN(:,3)), ...
                 'pos3D',f(vecnorm(dRTN,2,2)),'n',size(dRTN,1));
end
