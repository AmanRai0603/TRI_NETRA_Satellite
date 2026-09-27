function v = utcvec(utc)
%FRAMES.UTCVEC  Normalise a UTC input to a [Y Mo D H Mi S] row vector.
    if isa(utc,'datetime')
        v=[utc.Year utc.Month utc.Day utc.Hour utc.Minute utc.Second];
    elseif isnumeric(utc) && numel(utc)>=6
        v=utc(1:6); v=v(:).';
    else
        error('frames:utcvec','UTC must be datetime or [Y Mo D H Mi S]');
    end
end
