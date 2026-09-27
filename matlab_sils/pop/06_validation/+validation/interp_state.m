function [rq, vq] = interp_state(t, r, v, tq)
%VALIDATION.INTERP_STATE  Sample an orbit state at arbitrary epochs, correctly.
%   [rq, vq] = validation.interp_state(t, r, v, tq)
%
%   INPUT   t  [N x 1] epochs [s]      r [N x 3] position [m]
%           v  [N x 3] velocity [m/s]  tq [M x 1] query epochs [s]
%   OUTPUT  rq [M x 3] position, vq [M x 3] velocity at tq
%
%   ---------------------------------------------------------------------------
%   WHY THIS EXISTS -- do not replace it with interp1(...,'pchip').
%   ---------------------------------------------------------------------------
%   validate_OD used to sample the propagated solution with interp1 'pchip'. On a
%   30 s output grid at 450 km that produced a 72.6 m RMS position error --
%   against a reference whose own uncertainty is 3 cm. The kinematic comparison
%   was therefore reporting 72 m of INTERPOLANT as if it were a physics residual,
%   sitting 2000x above the very floor the script computes to stop you doing that.
%
%   The cause is not the step size, it is pchip itself. pchip is SHAPE-PRESERVING:
%   it deliberately flattens the slope wherever the samples turn over, to guarantee
%   no overshoot. An orbit turns over twice per revolution in every component, so
%   the limiter fires near every extremum and clips the true derivative. Measured
%   on the same 30 s grid:
%       pchip                     72.6440 m RMS   (radial 72.3, and BIASED +1.2 m)
%       spline                     0.0319 m RMS
%       Hermite with exact v       0.0276 m RMS   <- this function
%   The error is radial-dominated because it is a chord-vs-arc sagitta: the
%   interpolant cuts the corner of the orbit. That is a systematic bias, not noise,
%   so it does not average away.
%
%   Hermite is the right answer here for a reason that is specific to orbits: we
%   are not interpolating an arbitrary curve, we KNOW the exact derivative of
%   position -- it is the velocity, sitting in the same solution struct. A cubic
%   Hermite with exact end slopes is O(h^4) and needs no neighbour stencil, so it
%   also does not care that the query epochs are irregular (kinematic epochs are).
%
%   For position-only references (kinematicOrbit, TU Delft) there is no v to use.
%   Use interp1(...,'spline') there -- NOT pchip.
%
%   Outside [t(1), t(end)] the end interval is extrapolated. Guard the range in
%   the caller; validate_OD does, via its t >= 0 & t <= TSPAN_S masks.

    t = t(:); tq = tq(:);
    n = numel(t);
    if n < 2, error('validation:interp_state','need at least 2 epochs'); end
    if size(r,1) ~= n || size(v,1) ~= n
        error('validation:interp_state','r and v must have %d rows', n);
    end

    % bracket each query epoch: i such that t(i) <= tq < t(i+1)
    i = zeros(numel(tq),1);
    for k = 1:numel(tq)
        j = find(t <= tq(k), 1, 'last');
        if isempty(j), j = 1; end
        i(k) = min(max(j,1), n-1);
    end

    h = t(i+1) - t(i);
    s = (tq - t(i)) ./ h;

    % cubic Hermite basis and its derivative
    h00 =  2*s.^3 - 3*s.^2 + 1;
    h10 =    s.^3 - 2*s.^2 + s;
    h01 = -2*s.^3 + 3*s.^2;
    h11 =    s.^3 -   s.^2;
    d00 =  6*s.^2 - 6*s;
    d10 =  3*s.^2 - 4*s + 1;
    d01 = -6*s.^2 + 6*s;
    d11 =  3*s.^2 - 2*s;

    rq = h00.*r(i,:) + (h10.*h).*v(i,:) + h01.*r(i+1,:) + (h11.*h).*v(i+1,:);
    vq = (d00./h).*r(i,:) + d10.*v(i,:) + (d01./h).*r(i+1,:) + d11.*v(i+1,:);
end
