% EX9  The flight-software boundary. 'loopback' packs every sensor and actuator frame
%      to bytes and back each tick (the OILS/HILS link layout, docs/OILS_HILS.md).
%      For OILS: hal__backend = 'udp', hal__realtime = 1, flight OBC on the link.
% Copyright (c) 2026 Agastya. All rights reserved.
startup_asils
r = asils.run('fine_hold_img', 'cases/ais_img_3u.csv', 'set', struct('sim__duration_s', 300, 'hal__backend', 'loopback'));
S = asils.hal.stimulus(r);          % Helmholtz-cage field, Sun-simulator direction, air-bearing rate
fprintf('stimulus: %d samples, |B| %.0f..%.0f nT\n', numel(S.t), min(sqrt(sum(S.B_body_T.^2)))*1e9, max(sqrt(sum(S.B_body_T.^2)))*1e9);
