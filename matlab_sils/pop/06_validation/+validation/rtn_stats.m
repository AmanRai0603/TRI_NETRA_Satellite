function S = rtn_stats(d, r, v)
%VALIDATION.RTN_STATS  Resolve a position-difference series into the RTN frame.
%   S = validation.rtn_stats(d, r, v)
%
%   INPUT
%     d  [N x 3]  difference vector series, inertial frame [m]  (ours - reference)
%     r  [N x 3]  position defining the RTN frame at each epoch [m]
%     v  [N x 3]  velocity defining the RTN frame at each epoch [m/s]
%
%   PROCESS
%     At every epoch builds the local orbital triad
%        R = r/|r|                 radial
%        C = (r x v)/|r x v|       cross-track (orbit normal)
%        A = C x R                 along-track (completes the right-handed set)
%     and projects d onto it. Note A is the TRANSVERSE direction, not v/|v|; for a
%     near-circular orbit the two agree to O(e), which is why "along-track" is the
%     usual name.
%
%   OUTPUT  S with
%     .rad .alo .cro  [N x 1]  the per-epoch components [m]
%     .radial .along .cross    their RMS [m]
%     .pos3D                   RMS of |d| [m]
%     .t                       [] -- the caller owns the time base and fills it in
%
%   Lifted out of validate_OD (was a script-local function) so that it resolves
%   under Octave, which -- unlike MATLAB -- only defines a script's local functions
%   once execution reaches them, i.e. never, when they sit at the end of the file.
    n = size(d,1);
    rad = zeros(n,1); alo = zeros(n,1); cro = zeros(n,1);
    for k = 1:n
        R_ = r(k,:).'/norm(r(k,:));
        C_ = cross(r(k,:).', v(k,:).'); C_ = C_/norm(C_);
        A_ = cross(C_, R_);
        rad(k) = d(k,:)*R_;  alo(k) = d(k,:)*A_;  cro(k) = d(k,:)*C_;
    end
    % ---- RMS IS NOT ENOUGH: MEAN AND STD SAY DIFFERENT THINGS ----------------
    % RMS^2 = mean^2 + std^2, so one RMS number hides which of two very different
    % errors you have:
    %   mean >> std  -> a BIAS. Constant offset. A force is missing or mis-scaled.
    %   std  >> mean -> SCATTER about zero. Noise, or a periodic mismodelling.
    % Your GRACE-A run: along-track RMS 2.091 with mean +1.668, so std ~1.26 --
    % that is a BIAS with structure on top, and the bias is the drag deficit. The
    % radial is RMS 0.371 / mean -0.170, so std ~0.33: mostly oscillation, which is
    % the chord-vs-arc signature, not a force. The same RMS with a different split
    % would mean something completely different, and the RMS alone cannot tell you.
    S = struct('radial', rms_(rad), 'along', rms_(alo), 'cross', rms_(cro), ...
               'pos3D', sqrt(mean(sum(d.^2,2))), 't', [], ...
               'rad', rad, 'alo', alo, 'cro', cro, ...
               'mean_rad', mean(rad), 'mean_alo', mean(alo), 'mean_cro', mean(cro), ...
               'std_rad',  std(rad),  'std_alo',  std(alo),  'std_cro',  std(cro), ...
               'rmag', sqrt(sum(r.^2,2)));   % so callers can get the ORBIT RATE from
                                             % the arc instead of assuming an altitude
end

function y = rms_(x)
    y = sqrt(mean(x.^2));
end
