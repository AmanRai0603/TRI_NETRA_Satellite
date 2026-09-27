function [q, nflip] = quat_continuous(q)
%VALIDATION.QUAT_CONTINUOUS  Remove sign flips from a quaternion time series.
%   [q, nflip] = validation.quat_continuous(q)     q [N x 4], scalar-first
%
%   WHY THIS IS NOT OPTIONAL BEFORE INTERPOLATING.
%   q and -q are the SAME rotation, so a star-camera product is free to emit
%   either, and real ones do -- the sign flips whenever the solver's branch
%   changes. Nothing downstream notices: quat2dcm(q) == quat2dcm(-q) exactly.
%
%   But interpolation notices. Component-wise interp across a flip pulls the
%   quaternion through the origin: halfway between q and -q is 0, which is not a
%   rotation at all. Renormalising a near-zero quaternion then amplifies whatever
%   noise survived into a confidently WRONG attitude for the epochs either side of
%   the flip. In validate_OD that lands as isolated spikes in the accelerometer
%   comparison -- it barely moves the RMS, so it reads as sensor noise rather than
%   as a bug. That is what makes it worth fixing rather than tolerating.
%
%   The fix is trivial: walk the series and negate any sample sitting in the
%   opposite hemisphere from its predecessor (dot product < 0). This changes no
%   rotation, it only picks a consistent branch so the path is continuous.
%
%   Do this BEFORE interp1/slerp, and prefer 'spline' over 'pchip' afterwards --
%   pchip's shape-preserving limiter clips slopes at turning points (see
%   validation.interp_state for what that cost us in position).
%
%   nflip = how many samples were negated. If that is a large fraction of N, the
%   series is not merely branch-flipped and something else is wrong -- check it.
    if isempty(q), nflip = 0; return, end
    nflip = 0;
    for k = 2:size(q,1)
        if dot(q(k,:), q(k-1,:)) < 0
            q(k,:) = -q(k,:);
            nflip  = nflip + 1;
        end
    end
end
