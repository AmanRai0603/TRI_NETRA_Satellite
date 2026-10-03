function show_16U(RUNS, SWEEP, WANT, outdir)
%SHOW_16U  Figures for EXAMPLE_16U. Handles a single run or a whole sweep.
%
%   show_16U(RUNS, SWEEP, WANT, outdir)
%     RUNS  : cell array of struct('cfg','W','sol','val','B') -- one per sweep value
%     SWEEP : struct with .on .knob .values .label
%     WANT  : struct of flags .alt .elem .forces .energy .ground .three_d
%     outdir: '' or a folder to save PNGs into
%
%   DESIGN: one figure per QUESTION, and when a sweep is on every figure overlays
%   the runs on the same axes -- because the whole point of a sweep is to see the
%   curves separate. Nothing is stacked into a single crowded canvas.
    if nargin<4, outdir=''; end
    K = de440.constants(); Re = K.Re_earth; mu = K.mu_earth;   % not a retyped 6378137
    n  = numel(RUNS);
    sweeping = SWEEP.on && n > 1;
    co = lines(max(n,3));
    if sweeping
        leg = arrayfun(@(i) sprintf('%s = %s', tag(SWEEP.knob), valStr(RUNS{i}.val)), ...
                       1:n, 'UniformOutput', false);
    else
        leg = {sprintf('B = %.2e m^2/kg', RUNS{1}.B)};
    end
    ttl = sprintf('16U  %.0f km', (norm(RUNS{1}.sol.r(1,:))-Re)/1000);
    figs = [];

    % ---- altitude + decay -----------------------------------------------------
    if WANT.alt
        f = figure('Color','w','Name','altitude','Position',[60 60 980 560]); figs(end+1)=f;
        subplot(2,1,1); hold on; grid on; box on;
        for i=1:n
            S=RUNS{i}; plot(S.sol.t/3600, (vecnorm(S.sol.r,2,2)-Re)/1000, ...
                            'Color',co(i,:),'LineWidth',1.3);
        end
        ylabel('altitude [km]'); title([ttl ' — altitude']); legend(leg,'Location','best');
        subplot(2,1,2); hold on; grid on; box on;
        for i=1:n
            S=RUNS{i}; a=zeros(numel(S.sol.t),1);
            for k=1:numel(S.sol.t), a(k)=op.rv2coe(S.sol.r(k,:).',S.sol.v(k,:).',mu); end
            plot(S.sol.t/3600, a-a(1), 'Color',co(i,:),'LineWidth',1.3);
        end
        xlabel('time [h]'); ylabel('\Delta sma [m]');
        title('semi-major axis change — the SECULAR decay (drag), free of the short-period wobble');
    end

    % ---- elements -------------------------------------------------------------
    if WANT.elem
        f = figure('Color','w','Name','elements','Position',[80 80 1000 700]); figs(end+1)=f;
        nm = {'sma [m]','ecc [-]','inc [deg]','RAAN [deg]'};
        for p=1:4
            subplot(4,1,p); hold on; grid on; box on;
            for i=1:n
                S=RUNS{i}; m=numel(S.sol.t); y=zeros(m,1);
                for k=1:m
                    [a,e,inc,RA] = op.rv2coe(S.sol.r(k,:).',S.sol.v(k,:).',mu);
                    switch p
                        case 1, y(k)=a; case 2, y(k)=e;
                        case 3, y(k)=rad2deg(inc); case 4, y(k)=rad2deg(RA);
                    end
                end
                if p==1, y = y - y(1); end        % sma: show the change, not 6.7e6
                plot(S.sol.t/3600, y, 'Color',co(i,:),'LineWidth',1.2);
            end
            ylabel(nm{p});
            if p==1, title([ttl ' — orbital elements (sma shown as \Delta)']); legend(leg,'Location','best'); end
            if p==4, xlabel('time [h]'); end
        end
    end

    % ---- force ladder ---------------------------------------------------------
    % The most instructive plot in the file: it shows WHICH force is worth arguing
    % about at this altitude. At low LEO drag climbs to within a few orders of J2 and
    % everything else is decoration.
    if WANT.forces
        f = figure('Color','w','Name','force ladder','Position',[100 100 1000 560]); figs(end+1)=f;
        hold on; grid on; box on;
        S = RUNS{1};                              % the ladder is per-configuration
        m = numel(S.sol.t); names = {}; mag = [];
        for k=1:m
            [~, parts] = op.accel(S.sol.t(k), S.sol.r(k,:).', S.sol.v(k,:).', S.W);
            fn = fieldnames(parts);
            if k==1, names = fn; mag = zeros(m, numel(fn)); end
            for j=1:numel(names)
                if isfield(parts, names{j}), mag(k,j) = norm(parts.(names{j})); end
            end
        end
        for j=1:numel(names)
            if any(mag(:,j) > 0)
                semilogy(S.sol.t/3600, max(mag(:,j),1e-16), 'LineWidth',1.3, ...
                         'DisplayName', names{j});
            end
        end
        set(gca,'YScale','log'); xlabel('time [h]'); ylabel('|a| [m/s^2]');
        title([ttl ' — force magnitudes (which forces actually matter here)']);
        legend('Location','eastoutside');
    end

    % ---- energy drift = the integrator's own error ----------------------------
    if WANT.energy
        f = figure('Color','w','Name','energy','Position',[120 120 960 460]); figs(end+1)=f;
        hold on; grid on; box on;
        for i=1:n
            S=RUNS{i};
            E = 0.5*sum(S.sol.v.^2,2) - mu./vecnorm(S.sol.r,2,2);
            semilogy(S.sol.t/3600, max(abs((E-E(1))/E(1)),1e-18), ...
                     'Color',co(i,:),'LineWidth',1.2);
        end
        set(gca,'YScale','log'); xlabel('time [h]'); ylabel('|\DeltaE/E_0|');
        title([ttl ' — two-body energy drift (with drag ON this is PHYSICS, not error;']);
        subtitle_('turn drag/SRP off to read it as pure integrator error');
        legend(leg,'Location','best');
    end

    % ---- ground track ---------------------------------------------------------
    if WANT.ground
        f = figure('Color','w','Name','ground track','Position',[140 140 1000 520]); figs(end+1)=f;
        hold on; grid on; box on;
        for i=1:n
            S=RUNS{i}; m=numel(S.sol.t); la=zeros(m,1); lo=zeros(m,1);
            for k=1:m
                u  = utcAt(S.cfg.epoch, S.sol.t(k));
                T  = timeconv.convertUTC(u(1),u(2),u(3),u(4),u(5),u(6),0);
                C  = frames.eci2ecef(u, S.cfg.frame.build, struct('gmst_rad',T.gmst_rad));
                [la(k), lo(k)] = op.geodetic(C*S.sol.r(k,:).', Re, K.f_earth);
            end
            lo = mod(lo+180,360)-180;
            d = [0; abs(diff(lo))]; lo(d>180) = NaN;      % break the wrap
            plot(lo, la, '.', 'Color',co(i,:), 'MarkerSize',4);
        end
        xlabel('longitude [deg]'); ylabel('latitude [deg]');
        xlim([-180 180]); ylim([-90 90]); title([ttl ' — ground track']);
    end

    % ---- 3D -------------------------------------------------------------------
    if WANT.three_d
        f = figure('Color','w','Name','trajectory','Position',[160 160 760 700]); figs(end+1)=f;
        hold on; grid on; box on; axis equal;
        [xs,ys,zs] = sphere(40);
        surf(xs*Re/1e3, ys*Re/1e3, zs*Re/1e3, 'FaceColor',[0.85 0.9 0.95], ...
             'EdgeColor','none','FaceAlpha',0.5);
        for i=1:n
            S=RUNS{i};
            plot3(S.sol.r(:,1)/1e3, S.sol.r(:,2)/1e3, S.sol.r(:,3)/1e3, ...
                  'Color',co(i,:),'LineWidth',1.1);
        end
        xlabel('x [km]'); ylabel('y [km]'); zlabel('z [km]');
        title([ttl ' — ECI trajectory']); view(45,20);
    end

    if ~isempty(outdir)
        for i=1:numel(figs)
            print(figs(i),'-dpng','-r150', fullfile(outdir, sprintf('16U_fig%d.png', i)));
        end
    end
end

% ===================================================================== local ==
function u = utcAt(epoch, dt)
    dn = datenum(epoch) + dt/86400; v = datevec(dn); u = v(1:6);
end
function s = valStr(v)
    if ischar(v), s=v; elseif isempty(v), s='-'; else, s=sprintf('%g',v); end
end
function t = tag(knob)
    p = strsplit(knob,'.'); t = p{end};
end
function subtitle_(s)
    % subtitle() is R2020b+; keep this working on older MATLAB and on Octave.
    try, subtitle(s); catch, xl=get(gca,'XLabel'); set(xl,'String',[get(xl,'String') '   (' s ')']); end
end
