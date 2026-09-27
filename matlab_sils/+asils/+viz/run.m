function files = run(rec, outdir, visible)
%ASILS.VIZ.RUN  Standard figure set for one run, saved as PNG.
%   files = asils.viz.run(rec)                 % store/results/<scenario>/fig_*.png
%   files = asils.viz.run(rec, dir, true)      % also leave the figures open
%   1 attitude errors (APE/AKE, 3-axis and payload LOS) against the case requirement
%   2 body rates and mode timeline      3 disturbance torques by source
%   4 actuators: dipole, wheel momentum, power
%   5 environment from the precision orbit: density, shadow, |B|, altitude
%   6 ground track and 3-D orbit with Sun direction
    if nargin < 2 || isempty(outdir), outdir = fullfile(asils.util.root(), 'store', 'results', rec.P.id); end
    if nargin < 3, visible = false; end
    if ~exist(outdir, 'dir'), mkdir(outdir); end
    vis = 'off'; if visible, vis = 'on'; end
    t = rec.t/60; files = {};
    req = rec.P.case.v;
    ttl = sprintf('%s | %s | %s', rec.P.id, rec.P.case.id, rec.P.dev.id);

    f = figure('visible', vis, 'position', [50 50 1100 750]);
    subplot(2,1,1);
    semilogy(t, rec.ape_los, t, rec.ape_3ax); hold on; grid on;
    if isfinite(req.req_ape), plot(t([1 end]), req.req_ape*[1 1], 'r--', 'linewidth', 2); end
    ylabel('APE [deg]'); legend('payload LOS', '3-axis', 'requirement', 'location', 'northeast'); title([ttl ' - pointing'], 'interpreter', 'none');
    subplot(2,1,2);
    semilogy(t, rec.ake_los, t, rec.ake_3ax); hold on; grid on;
    if isfinite(req.req_ake), plot(t([1 end]), req.req_ake*[1 1], 'r--', 'linewidth', 2); end
    ylabel('AKE [deg]'); xlabel('time [min]'); legend('payload LOS', '3-axis', 'requirement'); title('knowledge', 'interpreter', 'none');
    files{end+1} = save_(f, outdir, 'fig1_attitude_error');

    f = figure('visible', vis, 'position', [50 50 1100 750]);
    subplot(3,1,1); plot(t, rec.w'*180/pi); grid on; ylabel('\omega [deg/s]'); legend('x','y','z'); title([ttl ' - body rate'], 'interpreter', 'none');
    subplot(3,1,2); semilogy(t, rec.rate); grid on; ylabel('|\omega| [deg/s]');
    subplot(3,1,3); stairs(t, rec.mode); grid on; ylim([0.5 5.5]);
    set(gca, 'ytick', 1:5, 'yticklabel', rec.modes); xlabel('time [min]'); title('mode', 'interpreter', 'none');
    files{end+1} = save_(f, outdir, 'fig2_rates_modes');

    f = figure('visible', vis, 'position', [50 50 1100 750]);
    nm = {'gravity gradient', 'aerodynamic', 'solar radiation', 'residual dipole'};
    for i = 1:4
        subplot(4,1,i); T = rec.tau_dist(3*(i-1)+(1:3), :);
        plot(t, T'); grid on; ylabel('[N m]'); title(sprintf('%s  (rms |tau| %.2e N m)', nm{i}, sqrt(mean(sum(T.^2,1)))), 'interpreter', 'none');
    end
    xlabel('time [min]'); files{end+1} = save_(f, outdir, 'fig3_disturbance_torques');

    f = figure('visible', vis, 'position', [50 50 1100 750]);
    subplot(3,1,1); plot(t, rec.m'); grid on; ylabel('dipole [A m^2]'); title([ttl ' - actuators'], 'interpreter', 'none'); legend('x','y','z');
    subplot(3,1,2);
    if any(isfinite(rec.h_w(:))), plot(t, rec.h_w'*1e3); ylabel('wheel h [mN m s]'); else, plot(t, sqrt(sum(rec.tau_mtq.^2,1))); ylabel('|tau_{mtq}| [N m]'); end
    grid on;
    subplot(3,1,3); plot(t, rec.P_mtq + rec.P_rw); grid on; ylabel('ADCS power [W]'); xlabel('time [min]');
    files{end+1} = save_(f, outdir, 'fig4_actuators');

    f = figure('visible', vis, 'position', [50 50 1100 750]);
    alt = (sqrt(sum(rec.r.^2,1)) - 6378137)/1e3;
    subplot(4,1,1); semilogy(t, rec.rho); grid on; ylabel('\rho [kg/m^3]'); title([ttl ' - environment from the in-loop precision orbit (POP)'], 'interpreter', 'none');
    subplot(4,1,2); plot(t, rec.nu); grid on; ylabel('sunlit fraction'); ylim([-0.05 1.05]);
    subplot(4,1,3); plot(t, sqrt(sum(rec.B.^2,1))*1e9); grid on; ylabel('|B| [nT]');
    subplot(4,1,4); plot(t, alt); grid on; ylabel('radius - R_e [km]'); xlabel('time [min]');
    files{end+1} = save_(f, outdir, 'fig5_environment');

    f = figure('visible', vis, 'position', [50 50 1100 600]);
    r = rec.r; lat = asind(r(3,:)./sqrt(sum(r.^2,1)));
    gm = mod(280.46061837 + 360.98564736629*(asils.util.jd(rec.P.epoch_utc) + rec.t/86400 - 2451545), 360);
    lon = mod(atan2d(r(2,:), r(1,:)) - gm + 180, 360) - 180;
    subplot(1,2,1); plot(lon, lat, '.', 'markersize', 2); grid on; axis([-180 180 -90 90]);
    xlabel('longitude [deg]'); ylabel('latitude [deg]'); title('ground track', 'interpreter', 'none');
    subplot(1,2,2); plot3(r(1,:)/1e3, r(2,:)/1e3, r(3,:)/1e3); hold on; grid on; axis equal;
    s = rec.sun_eci(:,1)*8000; plot3([0 s(1)], [0 s(2)], [0 s(3)], 'r-', 'linewidth', 2);
    [X, Y, Z] = sphere(24); surf(6378*X, 6378*Y, 6378*Z, 'facealpha', 0.3, 'edgecolor', 'none');
    xlabel('x [km]'); ylabel('y [km]'); zlabel('z [km]'); title('orbit (ECI) and Sun direction', 'interpreter', 'none');
    files{end+1} = save_(f, outdir, 'fig6_orbit');
end
function f = save_(h, d, name)
    f = fullfile(d, [name '.png']);
    print(h, f, '-dpng', '-r110');
    if strcmp(get(h, 'visible'), 'off'), close(h); end
end
