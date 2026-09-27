% EX3  Fine nadir hold (0.01 deg) of the 3U imaging satellite, 550 km SSO LTAN 10:00.
% Copyright (c) 2026 Agastya. All rights reserved.
startup_asils
rec = asils.run('fine_hold_img', 'cases/ais_img_3u.csv');
asils.viz.run(rec, '', true);
