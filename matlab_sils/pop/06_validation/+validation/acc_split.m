function S = acc_split(R, verbose)
%VALIDATION.ACC_SPLIT  Separate the drag and SRP deficits using ECLIPSE.
%   S = validation.acc_split(R)
%
%   ---------------------------------------------------------------------------
%   THE DEGENERACY THIS BREAKS
%   ---------------------------------------------------------------------------
%   Metric [3] says the modelled non-gravitational acceleration is short by ~1.4e-8
%   on GRACE-A. Two candidates, both plausible, both the right size:
%
%     (a) Cd 2.5 -> 3.31       (dria derives it)      : +6.2e-09
%     (b) SRP area 1x -> ~2.5x (attitude, real shape)  : +1.0e-08
%
%   An RMS over the arc CANNOT separate them. Both are just "more non-gravitational
%   acceleration", and at 480 km in solar minimum SRP is 0.81x the drag, so neither
%   dominates enough to ignore the other. Sweeping Cd would "fit" the residual and
%   silently absorb an SRP error into a drag parameter -- which is how a model gets
%   the right answer for the wrong reason and then fails on the next arc.
%
%   BUT SRP GOES TO EXACTLY ZERO IN ECLIPSE. DRAG DOES NOT.
%
%   The orbit runs this experiment twice per revolution, for free:
%       IN ECLIPSE : measured = drag + ERP        (SRP is OFF)
%       IN SUNLIGHT: measured = drag + ERP + SRP
%
%   So the eclipse samples test the DRAG model with SRP structurally unable to
%   contaminate them, and the difference tests SRP. This is the only place in the
%   toolbox where two non-gravitational forces can be separated by DATA rather than
%   by assumption -- everywhere else they are degenerate and we can only say so.
%
%   S.n_ecl / S.n_sun     sample counts
%   S.ratio_ecl           modelled/measured IN ECLIPSE  -> the DRAG test
%   S.ratio_sun           modelled/measured IN SUNLIGHT
%   S.verdict             which force the deficit lives in
    if nargin<2, verbose = true; end
    S = struct('ok',false,'n_ecl',0,'n_sun',0,'ratio_ecl',NaN,'ratio_sun',NaN, ...
               'ratio_drag',NaN,'ratio_srp',NaN,'verdict','');

    if ~isfield(R,'acc') || ~isfield(R.acc,'cmp') || ~isfield(R.acc.cmp,'srp')
        S.verdict = 'needs R.acc.cmp.srp (the per-force breakdown) to find eclipse';
        if verbose, fprintf('  acc_split: %s\n', S.verdict); end
        return
    end

    % Eclipse = the epochs where the MODEL put SRP at zero. Using the model's own
    % shadow function is the right call: we are testing the model's SRP against the
    % measurement, so the model must be the one that says when it thinks SRP is off.
    srpMag = sqrt(sum(R.acc.cmp.srp.^2, 2));
    ecl    = srpMag < 1e-12;
    S.n_ecl = sum(ecl); S.n_sun = sum(~ecl);
    if S.n_ecl < 10 || S.n_sun < 10
        S.verdict = sprintf(['not enough of both: %d eclipse / %d sunlit samples. ' ...
            'A sun-synchronous or high-beta orbit may never eclipse over a short arc.'], ...
            S.n_ecl, S.n_sun);
        if verbose, fprintf('  acc_split: %s\n', S.verdict); end
        return
    end

    % bias-removed, per the metric-[3] fix: the raw series is bias + signal
    am = R.acc.meas; ao = R.acc.mod;
    if isfield(R.acc,'ac_meas'), am = R.acc.ac_meas; ao = R.acc.ac_mod;
    else, am = am - mean(am,1); ao = ao - mean(ao,1); end

    rms_ = @(x) sqrt(mean(sum(x.^2,2)));
    S.ratio_ecl = rms_(ao(ecl,:)) / max(rms_(am(ecl,:)), eps);
    S.ratio_sun = rms_(ao(~ecl,:)) / max(rms_(am(~ecl,:)), eps);

    % ---- COMPARING THE TWO RATIOS IS NOT THE SRP TEST -----------------------
    % My first version read "sunlit ratio /= eclipse ratio" as evidence of an SRP
    % error. It is not. A test on a KNOWN case killed it: drag 60% low with PERFECT
    % SRP gave eclipse 0.514 and sunlit 0.716, and the code called that "SRP adds 20%
    % of error". It does not -- correct SRP DILUTES a drag error, because the sunlit
    % ratio is (r_d*D + S)/(D + S), which moves toward 1 as S grows. The ratios
    % differ for a reason that has nothing to do with SRP being wrong.
    %
    % The eclipse samples do not give a ratio to compare against; they give the DRAG
    % SCALE, which then lets us SOLVE for SRP:
    %
    %   in eclipse : meas = drag_true (+ERP)      ->  r_d = |drag_mod| / |meas_ecl|
    %   in sunlight: meas = drag_true + srp_true
    %                drag_true ~= drag_mod / r_d          (using the eclipse scale)
    %                srp_true  =  meas_sun - drag_true    (VECTOR subtraction)
    %                r_s       =  |srp_mod| / |srp_true|
    %
    % That is a real, separate number for each force, not a difference of ratios.
    S.ratio_drag = S.ratio_ecl;                    % clean: SRP structurally absent
    S.ratio_srp  = NaN;
    try
        dm = R.acc.cmp.drag; sm = R.acc.cmp.srp;
        if S.ratio_drag > 1e-6
            drag_true_sun = dm(~ecl,:) / S.ratio_drag;          % scale up by the eclipse-measured error
            srp_true      = am(~ecl,:) - (drag_true_sun - mean(drag_true_sun,1));
            S.ratio_srp   = rms_(sm(~ecl,:) - mean(sm(~ecl,:),1)) / max(rms_(srp_true), eps);
        end
    catch
        % leave NaN: better an absent number than an invented one
    end
    S.ok = true;

    near = @(x) isfinite(x) && abs(x-1) < 0.15;
    dOK = near(S.ratio_drag); sOK = near(S.ratio_srp);
    if dOK && sOK
        S.verdict = sprintf(['BOTH forces match: drag %.3f (from eclipse), SRP %.3f ' ...
            '(solved). The non-grav model is doing its job in both regimes.'], ...
            S.ratio_drag, S.ratio_srp);
    elseif dOK && isfinite(S.ratio_srp)
        S.verdict = sprintf(['the deficit is SRP. Drag is %.3f in eclipse, where SRP is ' ...
            'structurally absent -- so the DRAG MODEL IS FINE. SRP solves to %.3f. Look ' ...
            'at Cr (a catalog default), the SRP area, or the attitude -- not at Cd.'], ...
            S.ratio_drag, S.ratio_srp);
    elseif ~dOK && sOK
        S.verdict = sprintf(['the deficit is DRAG: %.3f in eclipse, with SRP switched off ' ...
            'by nature. SRP solves to %.3f, which is fine. Try DRAG_MODEL = dria -- it ' ...
            'DERIVES Cd (~3.3 at 480 km) instead of using the catalog 2.5.'], ...
            S.ratio_drag, S.ratio_srp);
    elseif ~dOK && isfinite(S.ratio_srp)
        S.verdict = sprintf(['BOTH are off, and now SEPARATELY SIZED: drag %.3f (clean, ' ...
            'from eclipse), SRP %.3f (solved). Fix the drag first -- its number is the ' ...
            'one no assumption about the Sun can contaminate.'], S.ratio_drag, S.ratio_srp);
    else
        S.verdict = sprintf(['drag is %.3f from eclipse (this number is CLEAN). SRP could ' ...
            'not be solved -- treat the sunlit ratio %.3f as drag+SRP combined.'], ...
            S.ratio_drag, S.ratio_sun);
    end

    if verbose
        fprintf('\n  ---- [3] SPLIT BY ECLIPSE: which force is short? ----\n');
        fprintf('    SRP is exactly zero in eclipse and drag is not, so the orbit runs\n');
        fprintf('    this experiment for free, twice per revolution.\n');
        fprintf('      IN ECLIPSE  (%4d samples): ratio %.4f   <- DRAG alone. Clean.\n', S.n_ecl, S.ratio_ecl);
        fprintf('      IN SUNLIGHT (%4d samples): ratio %.4f   <- drag + SRP mixed\n', S.n_sun, S.ratio_sun);
        fprintf('    solving with the eclipse scale:\n');
        fprintf('      DRAG model / truth : %.4f\n', S.ratio_drag);
        fprintf('      SRP  model / truth : %.4f\n', S.ratio_srp);
        fprintf('    %s\n', S.verdict);
    end
end
