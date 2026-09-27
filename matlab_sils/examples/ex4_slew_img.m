% EX4  30 deg target slew in 60 s and settle (imaging 3U).
% Copyright (c) 2026 Agastya. All rights reserved.
startup_asils
rec = asils.run('slew_img', 'cases/ais_img_3u.csv');
asils.viz.run(rec, '', true);
