function v = getm(R, sect, fld)
%GETM  R.(sect).(fld) if it exists, else NaN.
%   Sweep tables are ragged by nature -- a config with no accelerometer product,
%   or one that failed, simply has no R.acc. Printing NaN in that cell is the
%   honest answer; erroring out would lose the rows that DID work.
    if isstruct(R) && isfield(R,sect) && isfield(R.(sect),fld) && ~isempty(R.(sect).(fld))
        v = R.(sect).(fld);
    else
        v = NaN;
    end
end
