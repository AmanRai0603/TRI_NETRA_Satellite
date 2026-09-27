function S = scale_factor(meas, mod, opts)
%VALIDATION.SCALE_FACTOR  The single number that best rescales a model onto a truth.
%   S = validation.scale_factor(meas, mod)
%   S = validation.scale_factor(meas, mod, struct('name','drag','unit','m/s^2'))
%
%   meas, mod : [N x 1] or [N x 3]. Same epochs, same frame.
%
%   ---------------------------------------------------------------------------
%   WHY A SCALE FACTOR IS THE RIGHT QUESTION
%   ---------------------------------------------------------------------------
%   Drag is  a = 0.5 * rho * Cd * (A/m) * v^2.  On the orbit, `rho*Cd*A/m` is ONE
%   number: no amount of tracking data can tell you which factor was wrong. Cd is
%   not measured for any satellite in this catalog -- it is a literature value
%   somebody typed -- and A/m carries an Aref that is one number standing in for a
%   whole shape.
%
%   So a constant multiplicative error is EXPECTED and is not interesting. It says
%   "one of the four factors is off", which we already knew. The interesting
%   question is the one underneath:
%
%       AFTER absorbing the best possible scale factor, what is LEFT?
%
%   That remainder cannot be fixed by any Cd, any area, any density scaling. It is
%   structure -- a force pointing the wrong way, a missing term, a phase error --
%   and it is the only part of the residual that is evidence about the MODEL rather
%   than about the bookkeeping.
%
%   S.factor      least-squares scale: argmin |meas - k*mod|^2 = <meas,mod>/<mod,mod>
%   S.resid       meas - factor*mod        (what a scale factor CANNOT explain)
%   S.rms_before  rms(meas - mod)
%   S.rms_after   rms(S.resid)
%   S.explained   1 - rms_after/rms_before  (fraction of the residual the scale ate)
%   S.r2          how much of the measured VARIANCE the scaled model reproduces
%
%   ---------------------------------------------------------------------------
%   THE TWO NUMBERS THAT KEEP YOU HONEST
%   ---------------------------------------------------------------------------
%   `explained` and `r2` are here because a scale factor ALWAYS exists -- the
%   formula returns something for any two series, including two unrelated ones. A
%   factor of 1.87 next to `explained = 0.03` means the rescaling achieved nothing
%   and the model does not have the right SHAPE; quoting the 1.87 as "the Cd is 1.87x
%   too low" would then be pure fitting. A factor of 1.87 with `explained = 0.95`
%   means the model has the right shape and the wrong size, which is a real, useful,
%   and completely different finding.
%
%   Least squares, not the ratio of RMS values and not the mean of the pointwise
%   ratio. rms(meas)/rms(mod) ignores SIGN -- it would return ~1 for a model that is
%   perfectly anti-correlated with the truth. mean(meas./mod) blows up wherever mod
%   crosses zero, which for SRP is every eclipse. The projection is the only one of
%   the three that answers "what single k makes k*mod closest to meas".
    if nargin < 3, opts = struct(); end
    S = struct('factor',NaN, 'resid',[], 'rms_before',NaN, 'rms_after',NaN, ...
               'explained',NaN, 'r2',NaN, 'n',0, 'name',getf(opts,'name','signal'), ...
               'unit',getf(opts,'unit',''), 'note','');

    if isempty(meas) || isempty(mod), S.note = 'empty input'; return, end
    if ~isequal(size(meas), size(mod))
        S.note = sprintf('size mismatch: meas %s vs mod %s', mat2str(size(meas)), mat2str(size(mod)));
        return
    end

    m = meas(:); o = mod(:);
    good = isfinite(m) & isfinite(o);
    m = m(good); o = o(good);
    S.n = numel(m);
    if S.n < 3, S.note = 'fewer than 3 finite samples'; return, end

    den = dot(o, o);
    if den <= 0, S.note = 'model is identically zero -- no scale factor exists'; return, end

    S.factor     = dot(m, o) / den;
    r            = m - S.factor * o;
    S.resid      = reshape(meas - S.factor*mod, size(meas));
    S.rms_before = sqrt(mean((m - o).^2));
    S.rms_after  = sqrt(mean(r.^2));
    if S.rms_before > 0
        S.explained = 1 - S.rms_after/S.rms_before;
    end
    vm = var(m);
    if vm > 0
        S.r2 = 1 - var(r)/vm;
    end

    % Say what the numbers mean, once, here -- rather than leaving every caller to
    % re-derive the interpretation and one of them to get it wrong.
    if S.factor < 0
        % A NEGATIVE scale factor with a high r2 is anti-correlation, and my first
        % version called that "right shape, wrong size" -- it is not. The model
        % tracks the measurement and points the OTHER WAY: a sign error, a flipped
        % frame, a reversed normal. That is a serious finding and it must not be
        % reported as a Cd being 1.9x too small.
        S.note = sprintf(['NEGATIVE scale (x%.3f): the model is ANTI-correlated with the ' ...
                  'measurement. It tracks the shape and points the WRONG WAY -- a sign ' ...
                  'error, a flipped frame, or a reversed facet normal. This is not a ' ...
                  'Cd problem.'], S.factor);
    elseif S.rms_before == 0
        S.note = 'model already equals the measurement exactly; nothing to scale';
    elseif ~isfinite(S.explained)
        S.note = 'no baseline to compare against';
    elseif S.r2 > 0.8
        S.note = sprintf(['the model has the RIGHT SHAPE (r2=%.2f) and the wrong SIZE: ' ...
                  'x%.3f. That is a Cd/area/density statement.'], S.r2, S.factor);
    elseif S.r2 > 0.3
        S.note = sprintf(['the model gets part of the shape (r2=%.2f). A scale of x%.3f ' ...
                  'helps but %.0f%% of the residual survives it.'], S.r2, S.factor, ...
                  100*(1-max(S.explained,0)));
    else
        S.note = sprintf(['WRONG SHAPE (r2=%.2f). The scale factor x%.3f is arithmetic, ' ...
                  'not physics -- rescaling cannot fix a model that does not track ' ...
                  'the measurement. Do not quote it as a Cd correction.'], S.r2, S.factor);
    end
end

function v = getf(s, f, d)
    if isstruct(s) && isfield(s,f) && ~isempty(s.(f)), v = s.(f); else, v = d; end
end
