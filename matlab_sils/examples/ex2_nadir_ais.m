% EX2  Magnetic-only nadir hold of the 3U AIS satellite against its 10 deg APE.
% Copyright (c) 2026 Agastya. All rights reserved.
startup_asils
rec = asils.run('nadir_hold_ais', 'cases/ais_3u.csv');
asils.result.save(rec);                                  % channels, manifest, rec.mat (figures: adcs figures)
