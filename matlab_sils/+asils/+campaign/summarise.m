function s = summarise(rec)
%ASILS.CAMPAIGN.SUMMARISE  What a campaign keeps of one run: metrics, and
%   channels decimated to ~10 s for envelope plots.
    s.metrics = rec.metrics;
    n = numel(rec.t); step = max(1, round(10/max(rec.P.sim.record_dt, 1e-9)));
    k = 1:step:n;
    s.t = rec.t(k); s.ape_los = rec.ape_los(k); s.ape_3ax = rec.ape_3ax(k);
    s.ake_los = rec.ake_los(k); s.ake_3ax = rec.ake_3ax(k); s.rate = rec.rate(k);
    s.h_w = rec.h_w(:, k); s.mode = rec.mode(k);
end
