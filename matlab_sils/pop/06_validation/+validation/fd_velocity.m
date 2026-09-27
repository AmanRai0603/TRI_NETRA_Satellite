function v = fd_velocity(r, ts, opts)
%VALIDATION.FD_VELOCITY  Velocity from a measured position series (seed-grade).
%   v = validation.fd_velocity(r, ts[, opts])
%     r  (Nx3) positions [m] at times ts (Nx1) [s]
%     v  (Nx3) velocities [m/s]
%   opts.method : 'auto' (default) | 'poly' | 'fd'
%   opts.deg    : polynomial degree for 'poly'      (default 7 -- see autoWindow:
%                 at the ~1000 s span it targets, degree 5 is too curved. The old
%                 comment here said 5 and the code said 7.)
%   opts.window : window length (odd) for 'poly'    (default 21)
%
%   WHY THIS EXISTS
%   Only some products carry velocity:
%     SP3            -> V records if the header says #dV (then the FILE's v is
%                       used and this is not called)
%     TLE            -> r,v analytically from mean elements (not called)
%     TU Delft track -> POSITION ONLY, so v MUST be derived here.
%   The seed of a validation run is the FIRST sample, so its velocity error sets
%   the whole residual: dv maps to a semi-major-axis error da = 2*a*dv/v, seen as
%   a radial oscillation of amplitude da plus secular along-track growth.
%
%   TWO ERROR SOURCES, AND WHY 'poly' IS THE DEFAULT
%   (1) Truncation: a one-sided FIRST-order difference at the seed, (r2-r1)/h, is
%       off by (h/2)*|r''| ~ 44 m/s at 10 s (130 m/s at 30 s) for LEO.
%   (2) NOISE AMPLIFICATION: differentiating quantised/noisy positions multiplies
%       the position error by ~1/h, and a bare high-order stencil makes this WORSE
%       (larger coefficients). A track quantised to ~1 m at 10 s injects ~0.1-1 m/s
%       of velocity noise; at LEO 1 m/s of seed error is ~1.7 km of semi-major
%       axis -> kilometres of residual.
%   A sliding least-squares polynomial fit (Savitzky-Golay) fixes both: exact for
%   motion up to degree `deg` (kills truncation) and averages `window` samples
%   (suppresses noise by ~sqrt(window)), including AT THE ENDS, where the window
%   is shifted and the derivative evaluated at the point's offset.
%
%   Non-uniform sampling or a series too short for a window falls back to a
%   4th-order stencil, then to gradient, each with a warning.

    if nargin<3, opts=struct(); end
    method = getf(opts,'method','auto');
    deg    = getf(opts,'deg',7);
    win    = getf(opts,'window',[]);      % [] -> chosen from the SAMPLING RATE below

    n = size(r,1); ts = ts(:);
    if n ~= numel(ts)
        error('validation:fd_velocity:size','r (%dx3) and ts (%d) must agree', n, numel(ts));
    end

    uniform = false; h = NaN;
    if n >= 2
        h = ts(2)-ts(1);
        uniform = all(abs(diff(ts)-h) < 1e-6*max(1,abs(h)));
    end

    if strcmp(method,'auto')
        wNeed = win; if isempty(wNeed) && uniform, wNeed = autoWindow(h, deg); end
        if uniform && ~isempty(wNeed) && n >= wNeed, method='poly'; else, method='fd'; end
    end

    switch method
        case 'poly'
            if ~uniform, error('validation:fd_velocity:poly','poly needs uniform sampling'); end
            if isempty(win), win = autoWindow(h, deg); end
            if mod(win,2)==0, win=win+1; end
            if win > n, win = n - (1-mod(n,2)); end        % keep odd and <= n
            if win < deg+2
                warning('validation:fd_velocity:window', ...
                    'window %d too small for degree %d -> using 4th-order FD', win, deg);
                v = fdStencil(r, ts, h, n); return
            end
            v = sgVelocity(r, h, deg, win);

        case 'fd'
            if uniform && n >= 5
                v = fdStencil(r, ts, h, n);
            else
                if ~uniform
                    warning('validation:fd_velocity:nonuniform', ...
                        ['non-uniform sampling -> 2nd-order gradient; the seed velocity ' ...
                         'carries more error than usual.']);
                else
                    warning('validation:fd_velocity:short', ...
                        ['only %d samples -> 2nd-order gradient; the seed velocity ' ...
                         'carries more error than usual.'], n);
                end
                v = zeros(n,3);
                for j=1:3, v(:,j) = gradient(r(:,j), ts); end
            end
        otherwise
            error('validation:fd_velocity:method','unknown method "%s"', method);
    end
end

% ---------------------------------------------------------------------------
function v = sgVelocity(r, h, p, w)
% Sliding least-squares polynomial derivative (Savitzky-Golay), valid at the ends.
%
%   CONDITIONING -- this used to build the Vandermonde in RAW sample offsets:
%       s = (-m:m).';   A(:,j+1) = s.^j;   C = (A.'*A) \ A.';
%   With the defaults (window 21, degree 5) s reaches +/-10 and s^5 reaches 1e5, so
%   A'A spans ~1e10; on a wider window it is far worse. MATLAB reported
%       RCOND = 4.46e-24
%   on the TU Delft track -- i.e. the normal equations were numerically singular and
%   the fitted derivative was whatever the round-off decided. Forming A'A SQUARES the
%   condition number, which is why the classic advice is never to solve a Vandermonde
%   least-squares that way.
%
%   Two fixes, both standard:
%     1. NORMALISE the abscissa to u = s/m in [-1,1]. Then u^j stays O(1) and the
%        condition number drops by ~m^(2p). The chain rule gives du/ds = 1/m, so the
%        derivative picks up a 1/m: d/dt = (1/(m*h)) * d/du.
%     2. Solve by QR (A\I) instead of the normal equations (A'A)\A', which halves
%        the condition number of the solve.
    n = size(r,1); m = (w-1)/2;
    u = (-m:m).'/m;                          % normalised to [-1,1]  <-- the fix
    A = zeros(w, p+1);
    for j=0:p, A(:,j+1) = u.^j; end
    C = A \ eye(w);                          % (p+1) x w, via QR -- not (A'A)\A'

    hu = m*h;                                % d/dt = (1/(m*h)) d/du
    v = zeros(n,3);
    % interior: derivative at the window centre (u0 = 0) -> only the linear term
    wInt = C(2,:) / hu;
    for k = m+1 : n-m
        v(k,:) = wInt * r(k-m:k+m, :);
    end
    % ends: shift the window, evaluate the fitted derivative at the point's offset
    for k = [1:m, n-m+1:n]
        if k <= m, i0 = 1;      u0 = (k - (m+1))/m;
        else,      i0 = n-w+1;  u0 = (k - (n-m))/m;
        end
        d = zeros(1,w);
        for j = 1:p
            d = d + j * (u0^(j-1)) * C(j+1,:);
        end
        v(k,:) = (d / hu) * r(i0:i0+w-1, :);
    end
end

% ---------------------------------------------------------------------------
function v = fdStencil(r, ts, h, n)
% 4th-order central + 4th-order one-sided at the ends (no noise suppression).
    v = zeros(n,3);
    if n < 5
        for j=1:3, v(:,j) = gradient(r(:,j), ts); end, return
    end
    v(3:n-2,:) = (-r(5:n,:) + 8*r(4:n-1,:) - 8*r(2:n-3,:) + r(1:n-4,:)) / (12*h);
    v(1,:)   = (-25*r(1,:)   + 48*r(2,:)   - 36*r(3,:)   + 16*r(4,:)   -  3*r(5,:)  ) / (12*h);
    v(2,:)   = ( -3*r(1,:)   - 10*r(2,:)   + 18*r(3,:)   -  6*r(4,:)   +    r(5,:)  ) / (12*h);
    v(n-1,:) = (  3*r(n,:)   + 10*r(n-1,:) - 18*r(n-2,:) +  6*r(n-3,:) -    r(n-4,:)) / (12*h);
    v(n,:)   = ( 25*r(n,:)   - 48*r(n-1,:) + 36*r(n-2,:) - 16*r(n-3,:) +  3*r(n-4,:)) / (12*h);
end

function w = autoWindow(h, deg)
% Choose the window from the SAMPLING RATE, not a fixed point count.
% Truncation of the fit grows with the window's TIME SPAN as a fraction of the
% orbit (a degree-5 fit is excellent over ~4% of an orbit, poor over ~12%), while
% noise suppression wants more points. Targeting a fixed ~210 s span keeps the
% truncation small at any sampling rate and still averages many samples when the
% data are dense (e.g. 21 points at 10 s, 9 points at 30 s).
    % Two competing errors:
    %   truncation ~ how much ORBIT the window spans (fix with a higher degree)
    %   noise      ~ how FEW samples it averages (fix with a longer window)
    % A real GPS-derived track carries metre-level rounding, and differentiating it
    % amplifies that by ~1/h, so the window must be long. Measured on a 5 m-quantised
    % 10 s track at 265 km (seed error):
    %   span  3.9% (w=21)  deg5 0.339   <- the old default: NOT enough averaging
    %   span 18.7% (w=101) deg5 1.007   <- too curved for degree 5
    %   span 18.7% (w=101) deg7 0.033   <- 10x better  <== chosen
    %   span 29.9% (w=161) deg9 0.021
    % So: target ~1000 s (~18% of a LEO orbit) and let degree 7 carry the curvature.
    TARGET_SPAN_S = 1000;
    w = round(TARGET_SPAN_S / max(h, eps));
    w = max(w, deg+4);                 % enough points to fit at all
    w = min(w, 121);                   % never span too much of the orbit
    if mod(w,2)==0, w = w+1; end
end

function val=getf(s,f,d), if isfield(s,f)&&~isempty(s.(f)), val=s.(f); else, val=d; end, end
