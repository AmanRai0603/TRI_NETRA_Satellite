% EX1  B-dot detumble of the 3U AIS satellite (coils only), 550 km dawn-dusk SSO.
% Copyright (c) 2026 Agastya. All rights reserved.
startup_asils
rec = asils.run('detumble_ais', 'cases/ais_3u.csv');   % three orbits, prints the metrics
asils.viz.run(rec, '', true);                            % figures on screen (+ PNG in store/results)
