% EX8  Fault injection: a reaction wheel fails; FDIR isolates it and the coils fly the lost axis.
% Copyright (c) 2026 Agastya. All rights reserved.
startup_asils
r = asils.run('fault_wheel_img', 'cases/ais_img_3u.csv', 'set', struct('sim__duration_s', 1500, 'faults__t_s', 400));
for i = 1:numel(r.mode_log), fprintf('%7.1f s  %s\n', r.mode_log(i).t, r.mode_log(i).mode); end
figure; semilogy(r.t, r.ape_los); grid on; xlabel('time [s]'); ylabel('camera APE [deg]'); title('wheel 1 fails at 400 s');
