function G = gps_history(G, t, r, v, latency)
%ASILS.DEVICES.GPS_HISTORY  Keep the truth state of every tick back to the newest
%   one at or before t - latency (what a fix latency_s old needs).
%   Engine: adcs-sim-core sensors.rs Gps::history.
    G.t(end+1) = t; G.r(:, end+1) = r; G.v(:, end+1) = v;
    tl = t - latency;
    k = 1;
    while k + 1 <= numel(G.t) && G.t(k+1) <= tl + 1e-9, k = k + 1; end
    if k > 1, G.t = G.t(k:end); G.r = G.r(:, k:end); G.v = G.v(:, k:end); end
end
