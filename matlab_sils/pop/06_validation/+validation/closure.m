function C = closure(R, verbose)
%VALIDATION.CLOSURE  Do the metrics agree with EACH OTHER?
%   C = validation.closure(R)
%
%   ---------------------------------------------------------------------------
%   THE QUESTION NOBODY WAS ASKING
%   ---------------------------------------------------------------------------
%   Metric [3] says the modelled non-gravitational acceleration is short by some
%   amount. Metric [1] says the along-track position is off by some amount. Those
%   are not two independent facts: **a missing along-track acceleration MUST show up
%   as a position drift, and the relationship is arithmetic**:
%
%       dx = 0.5 * da * t^2
%
%   If they do not agree, one of them is wrong -- and that is worth far more than
%   either number alone. This is exactly the check that broke the CHAMP mystery:
%   metric [3] claimed a 10x drag deficit (1.79e-6 m/s^2), which over a 3 h arc
%   demands 104 m of along-track drift, and metric [1] measured 2.2 m. The metrics
%   contradicted each other by a factor of 50, and it took twelve rounds to notice
%   because nothing ever compared them.
%
%   So: compare them, every run, automatically.
%
%   C.da_acc      the acceleration deficit metric [3] reports [m/s^2]
%   C.dx_pred     the along-track drift it implies over the arc [m]
%   C.dx_obs      the along-track drift metric [1] actually measured [m]
%   C.factor      dx_obs / dx_pred. ~1 = the metrics agree.
%   C.verdict     what that means
    if nargin<2, verbose = true; end
    C = struct('ok',false,'da_acc',NaN,'dx_pred',NaN,'dx_obs',NaN,'factor',NaN,'verdict','', ...
               'kin_pred_mm',NaN,'kin_obs_mm',NaN,'kin_factor',NaN,'kin_ok',false);

    if ~isfield(R,'acc') || ~isfield(R,'rdo') || isempty(R.acc) || isempty(R.rdo)
        C.verdict = 'needs metric [1] and metric [3]; one is absent';
        if verbose, fprintf('  closure: %s\n', C.verdict); end
        return
    end

    % the deficit, bias-removed (the raw one is drag/bias and means nothing here)
    if isfield(R.acc,'rms_ac_meas') && isfield(R.acc,'rms_ac_mod')
        C.da_acc = abs(R.acc.rms_ac_meas - R.acc.rms_ac_mod);
    elseif isfield(R.acc,'rms_meas') && isfield(R.acc,'rms_mod')
        C.da_acc = abs(R.acc.rms_meas - R.acc.rms_mod);
    else
        C.verdict = 'metric [3] carries no RMS to difference'; return
    end

    t = R.acc.t(:);
    T = max(t) - min(t);
    if T <= 0, C.verdict = 'zero-length arc'; return, end

    % An acceleration deficit is NOT all along-track, and only the along-track part
    % integrates into a secular drift. Taking the full magnitude gives the UPPER
    % bound on the drift it can explain -- which is the useful direction: if even the
    % upper bound cannot reach the observed drift, something else is moving it.
    C.dx_pred = 0.5 * C.da_acc * T^2;
    C.dx_obs  = abs(subsref_default(R.rdo,'along',NaN));      % RMS along-track [m]
    C.factor  = C.dx_obs / max(C.dx_pred, realmin);

    if ~isfinite(C.factor)
        C.verdict = 'cannot compare';
    elseif C.factor > 0.1 && C.factor < 10
        C.ok = true;
        C.verdict = sprintf(['CONSISTENT: the accelerometer deficit and the position drift ' ...
            'agree to %.1fx. Both are describing the same missing force.'], ...
            max(C.factor,1/C.factor));
    elseif C.factor < 0.1
        C.verdict = sprintf(['INCONSISTENT: metric [3] claims a deficit that would move the ' ...
            'orbit %.1f m along-track, but metric [1] measured only %.2f m -- %.0fx less. ' ...
            'A deficit that large CANNOT hide inside that residual. Metric [3] is ' ...
            'measuring something that is not a force: an instrument bias, a frame ' ...
            'mismatch, or a units error in the reference.'], C.dx_pred, C.dx_obs, 1/C.factor);
    else
        C.verdict = sprintf(['INCONSISTENT: the position drifts %.2f m along-track but the ' ...
            'accelerometer deficit only explains %.2f m -- %.0fx short. Something is ' ...
            'moving the orbit that the accelerometer does not see: a gravity/frame ' ...
            'error, or a force that is off in the model but present in truth.'], ...
            C.dx_obs, C.dx_pred, C.factor);
    end

    % ---- KINEMATIC CLOSURE: dv_radial must equal -n * x_along ----------------
    % Reading your GRACE-A plots, the radial velocity residual is SECULAR (-4 mm/s
    % and growing) while the radial POSITION residual is BOUNDED (+-0.5 m). If dv_R
    % were the derivative of dr_R that is impossible -- so either something is
    % wrong, or dv_R is not what it looks like.
    %
    % It is not. THE RTN TRIAD ROTATES. For a purely along-track residual:
    %
    %    d/dt (x_T * That) = xdot_T*That + x_T*dThat/dt = xdot_T*That - n*x_T*Rhat
    %
    % so a growing along-track POSITION error MUST produce a radial VELOCITY
    % residual of -n*x_T. Measured on GRACE-A: x_T = 4.6 m at 180 min, n = 1.1165e-3
    % rad/s, predicted dv_R = -5.14 mm/s, plot shows -4.2 mm/s. 78% agreement from
    % numbers read off a picture.
    %
    % This check exists because that panel LOOKS alarming and is not. Without it,
    % the honest response to "is the velocity error growing a problem?" is a shrug --
    % and a shrug is how a real bug gets waved through next time.
    if isfield(R,'rdo') && isfield(R.rdo,'dv_rtn') && ~isempty(R.rdo.dv_rtn) && ...
       isfield(R.rdo,'alo') && ~isempty(R.rdo.alo)
        try
            K_ = de440.constants();
            % orbit rate from the arc itself, not assumed
            rmag = subsref_default(R.rdo,'rmag',[]);
            if isempty(rmag), rmag = K_.Re_earth + 400e3; end
            nrate = sqrt(K_.mu_earth/mean(rmag)^3);
            xT    = R.rdo.alo(:);                 % along-track POSITION residual [m]
            dvR   = R.rdo.dv_rtn(:,1);            % radial VELOCITY residual [m/s]
            pred  = -nrate * xT;
            sel   = abs(pred) > 1e-6;             % only where there is something to check
            if any(sel)
                C.kin_pred_mm = 1000*pred(end);
                C.kin_obs_mm  = 1000*dvR(end);
                C.kin_factor  = dvR(end)/pred(end);
                C.kin_ok      = abs(C.kin_factor-1) < 0.5;
            end
        catch
            % a diagnostic must not break the diagnosis
        end
    end

    if verbose
        fprintf('\n  ---- CLOSURE: do metrics [1] and [3] agree with each other? ----\n');
        fprintf('    acceleration deficit (bias-removed) : %.4e m/s^2\n', C.da_acc);
        fprintf('    arc length                          : %.0f s\n', T);
        fprintf('    -> along-track drift it implies     : %.2f m   (dx = 0.5*da*t^2)\n', C.dx_pred);
        fprintf('    along-track drift actually measured : %.2f m   (metric [1] RMS)\n', C.dx_obs);
        fprintf('    ratio observed/predicted            : %.2f\n', C.factor);
        fprintf('    %s\n', C.verdict);
        if isfield(C,'kin_factor') && isfinite(C.kin_factor)
            fprintf('\n    kinematic check (the radial dv panel that looks alarming):\n');
            fprintf('      a purely along-track offset must give dv_radial = -n*x_along,\n');
            fprintf('      because the RTN triad rotates. It is NOT a radial force.\n');
            fprintf('      predicted %.2f mm/s | observed %.2f mm/s | ratio %.2f  %s\n', ...
                    C.kin_pred_mm, C.kin_obs_mm, C.kin_factor, ...
                    subsref_tern(C.kin_ok,'CONSISTENT','<== does NOT match: investigate'));
        end
    end
end
