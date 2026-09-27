function [r, isCorrected] = acc_ratio(R)
%VALIDATION.ACC_RATIO  The metric-[3] ratio, bias-removed. ONE definition.
%   [r, isCorrected] = validation.acc_ratio(R)
%
%   ---------------------------------------------------------------------------
%   WHY THIS IS A FUNCTION
%   ---------------------------------------------------------------------------
%   Five places computed this ratio, each with its own inline
%   `R.acc.rms_mod/max(R.acc.rms_meas,eps)`. When od_metrics was corrected to remove
%   the accelerometer bias, ONE of the five learned about it. The other four kept
%   dividing raw -- so the figure would have printed 0.104 while the console printed
%   the corrected number, and compare_OD would have SWEPT against the raw one.
%
%   That last part is the serious one. The raw ratio is ~drag/bias, and the bias is
%   a CONSTANT ~10x larger than the signal. So every sweep against metric [3] would
%   come back nearly FLAT -- which is exactly the symptom that hid this bug for
%   twelve rounds. A comparator pointed at a contaminated metric does not just give
%   a wrong answer; it gives a CONFIDENT wrong answer, because a flat table reads as
%   "this knob does not matter".
%
%   One quantity, one definition, one place -- the same rule this audit has applied
%   to .area/.Aref, drag.force vs panelCoeffs, and the duplicate model knobs.
%
%   CHAMP's STAR accelerometer carries ~1e-6 m/s^2 of along-track bias, about ten
%   times the drag it measures at 345 km. An RMS taken about zero is therefore mostly
%   the instrument offset. Removing the per-axis mean from BOTH series leaves the
%   varying part, which is what the drag models actually predict.
    isCorrected = false;
    r = NaN;
    if ~isstruct(R) || ~isfield(R,'acc') || isempty(R.acc), return, end
    A = R.acc;
    if isfield(A,'ratio') && ~isempty(A.ratio) && isfinite(A.ratio)
        r = A.ratio; isCorrected = true; return          % od_metrics did the work
    end
    % Older R (or a hand-built one): correct it here rather than falling back to the
    % raw number silently. Falling back quietly is how the raw one survived.
    if isfield(A,'meas') && isfield(A,'mod') && ~isempty(A.meas)
        ac_m = A.meas - mean(A.meas,1);
        ac_o = A.mod  - mean(A.mod, 1);
        den  = sqrt(mean(sum(ac_m.^2,2)));
        if den > 0
            r = sqrt(mean(sum(ac_o.^2,2))) / den; isCorrected = true; return
        end
    end
    if isfield(A,'rms_mod') && isfield(A,'rms_meas') && A.rms_meas > 0
        r = A.rms_mod / A.rms_meas;                      % RAW -- flagged, not hidden
        isCorrected = false;
    end
end
