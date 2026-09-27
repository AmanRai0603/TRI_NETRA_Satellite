function s = val_str(v)
%VALIDATION.VAL_STR  A sweep value as a short label, whatever its type.
    if ischar(v),        s = v;
    elseif islogical(v), if v, s = 'on'; else, s = 'off'; end
    elseif isempty(v),   s = 'baseline';
    else,                s = sprintf('%g', v);
    end
end
