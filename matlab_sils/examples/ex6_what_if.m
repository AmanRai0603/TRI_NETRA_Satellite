% EX6  What-if: change any parameter without editing code, e.g. solar maximum and a
%      dirtier magnetic spacecraft, and compare the AIS nadir hold with the nominal.
% Copyright (c) 2026 Agastya. All rights reserved.
startup_asils
nominal = asils.run('nadir_hold_ais', 'cases/ais_3u.csv', 'set', struct('sim__duration_s', 11480));
stormy  = asils.run('nadir_hold_ais', 'cases/ais_3u.csv', 'set', struct('sim__duration_s', 11480, ...
             'env__F107', 250, 'env__F107a', 250, 'env__Kp', 6, 'sc__m_res', [0.02; 0.02; 0.02]));
figure; semilogy(nominal.t/60, nominal.ape_los, stormy.t/60, stormy.ape_los); grid on
legend('nominal', 'solar max + 3.5x dipole'); xlabel('time [min]'); ylabel('AIS antenna APE [deg]');
