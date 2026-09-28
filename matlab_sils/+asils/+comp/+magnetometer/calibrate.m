function B = calibrate(B_raw, cal)
%ASILS.COMP.MAGNETOMETER.CALIBRATE  STUB: raw field -> calibrated field with a
%   stored bias and scale/misalignment matrix (ground calibration). The
%   in-orbit calibration (attitude-independent, e.g. TWOSTEP) is to come; until
%   then the SILS flies the magnetometer model's output directly.
    if nargin < 2 || isempty(cal), B = B_raw; return, end
    B = cal.M*(B_raw - cal.b);
end
