function F30s = f30_to_f107scale(F30, decimalYear)
%F30_TO_F107SCALE  Rescale native F30 to the F10.7 scale that DTM2020_Res expects.
%
% Input  : native F30 vector, decimal year
% Process: applies the drift-corrected linear regression of the DTM2020 paper (eq. 2): F30* =
%          -1.5998 + 1.553755*F30 + (0.22446*year - 447.13328)
% Output : F30 on the F10.7 scale (the dtm5 driver)
%
%
%   DTM2020's research model (dtm5) is driven by F30 ONLY AFTER rescaling it to
%   F10.7 via the drift-corrected linear regression (authoritative SWAMI/MCM docs,
%   DTM2020 paper eq. 2):
%
%       F30* = -1.5998 + 1.553755*F30 + (0.22446*year - 447.13328)
%
%   with 'year' the decimal calendar year (e.g. 2009.836). Native F30 (~48 sfu at
%   the 2009 solar minimum) rescales to ~77, i.e. comparable to F10.7. Feed F30*
%   (and the rescaled 81-day mean) to dtm2020_density -- NEVER native F30, or the
%   model sees far too little solar input and under-predicts density badly.
%
%   Per SWAMI: "If you are simply running simulations, you can use F10.7 too."
%
%   INPUTS  F30          native F30 [sfu] (scalar or array)
%           decimalYear  decimal year (scalar or array, same size as F30)
%   OUTPUT  F30s         F30 rescaled to the F10.7 scale [sfu]

F30s = -1.5998 + 1.553755.*F30 + (0.22446.*decimalYear - 447.13328);
end
