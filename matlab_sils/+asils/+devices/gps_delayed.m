function [te, r, v] = gps_delayed(G, t, latency)
%ASILS.DEVICES.GPS_DELAYED  The truth state at t - latency (linear between ticks),
%   and its epoch: what a GNSS fix delivered latency_s after its epoch holds.
%   Before the history reaches back that far (the first latency_s of a run): the
%   oldest state and its epoch. Engine: adcs-sim-core sensors.rs Gps::delayed.
    tl = t - latency;
    n = numel(G.t);
    k = 1;
    while k + 1 <= n && G.t(k+1) <= tl + 1e-9, k = k + 1; end
    if k == n || tl <= G.t(k)
        te = max(G.t(k), tl); r = G.r(:, k); v = G.v(:, k); return
    end
    s = (tl - G.t(k))/(G.t(k+1) - G.t(k));
    te = tl;
    r = G.r(:, k) + s*(G.r(:, k+1) - G.r(:, k));
    v = G.v(:, k) + s*(G.v(:, k+1) - G.v(:, k));
end
