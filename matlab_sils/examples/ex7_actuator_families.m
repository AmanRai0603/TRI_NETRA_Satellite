% EX7  The same 30 deg slew with every actuator family.
% Copyright (c) 2026 Agastya. All rights reserved.
startup_asils
fam = {'img', 'fmr', 'rw_rcs', 'cmg', 'vscmg'};
figure; hold on
for i = 1:numel(fam)
    r = asils.run(['slew_' fam{i}], 'cases/ais_img_3u.csv', 'quiet', true);
    semilogy(r.t/60, r.ape_los); fprintf('%-7s settle %.1f s, on target %.4f deg\n', fam{i}, r.metrics(1).value, r.metrics(2).value);
end
set(gca, 'yscale', 'log'); grid on; legend(fam); xlabel('time [min]'); ylabel('camera APE [deg]');
