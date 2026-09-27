function F30 = f30_from_f107(F107)
%F30_FROM_F107  Approximate NATIVE F30 from F10.7 (offline, no download).
%
% Input  : F107 vector [sfu]
% Process: inverts the CLS linear F30<->F10.7 regression (approximation; not a measurement)
% Output : pseudo-F30 vector on the native scale
%
%
% *** DIAGNOSTIC / FALLBACK ONLY ***
% Intended as an unblocking fallback when LISIRD/CLS is unreachable, or to quickly
% test whether the F30 *scale* was the source of error. For a real GOCE validation, use
% the measured F30 from get_f30_cls (CLS/LISIRD). This is an approximation.
%
% Inverts the published F30<->F10.7 relation (Dudok de Wit & Bruinsma 2017;
% Yaya et al. 2017):   F30* = 1.554*F30 - 1.6   (F30* = F30 scaled to F10.7)
% so:                  F30  = (F107 + 1.6) / 1.554
%
% This maps F10.7 down to the NATIVE F30 scale the research model expects.
% Example (Nov 2009): F107 ~ 73  ->  F30 ~ 48 sfu  (NOT 73).
%
% USAGE
%   % F10.7 is already available (the operational model uses it). Convert + feed:
%   F30     = f30_from_f107(F107_daily);     % native-scale daily F30
%   F30_bar = f30_from_f107(F107_81day_mean);% native-scale 81-day mean
%   o = dtm2020_density(270,lat,lon,lst,doy, F30, F30_bar, ap60);
%
% If the research-model error drops from ~46% toward the operational ~3-15%
% after this, the F30 SCALE was indeed the bug -> then switch to real F30.

F30 = (F107 + 1.6) ./ 1.554;
end
