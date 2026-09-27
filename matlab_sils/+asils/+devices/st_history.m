function D = st_history(D, t, q, span)
%ASILS.DEVICES.ST_HISTORY  Keep the true attitude of the last `span` seconds
%   (every plant tick) so the star tracker can output its latency-old solution.
    D.hist_t(end+1) = t; D.hist_q(:, end+1) = q;
    if D.hist_t(1) < t - span
        keep = D.hist_t >= t - span;
        D.hist_t = D.hist_t(keep); D.hist_q = D.hist_q(:, keep);
    end
end
