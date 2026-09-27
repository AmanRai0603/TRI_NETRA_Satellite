function tt = time_to(t, x, thr, hold)
%ASILS.METRICS.TIME_TO  First time x drops below thr and stays below for hold seconds.
%   NaN when it never does within the run.
    tt = NaN; below = x < thr; n = numel(t);
    k = 1;
    while k <= n
        if below(k)
            j = k;
            while j < n && below(j+1), j = j + 1; end
            if t(j) - t(k) >= hold || (j == n && t(n) - t(k) >= hold)
                tt = t(k); return
            end
            k = j + 1;
        else
            k = k + 1;
        end
    end
end
