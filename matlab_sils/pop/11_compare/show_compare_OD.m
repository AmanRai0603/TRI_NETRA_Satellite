function show_compare_OD(RUNS, SWEEP, ttl, outdir)
%SHOW_COMPARE_OD  Figures for a compare_OD sweep. One figure per question.
%
%   show_compare_OD(RUNS, SWEEP, ttl, outdir)
%
%   F1  the ranking bar chart, WITH the reference floor drawn across it. The floor
%       is the point: bars below it are not distinguishable from each other and the
%       ordering between them is noise.
%   F2  RTN residual vs time, all configs overlaid -- shows WHERE a config loses
%       (a growing along-track error is drag; a once-per-rev radial signature is
%       gravity or the seed; a constant cross-track offset is a frame problem).
%   F3  accelerometer gap per config -- the measurement, not the fit.
%   F4  density ratio per config, with 1.0 marked.
%   F5  a text summary table.
%
%   Configs that FAILED are drawn as gaps and named in F5, not silently dropped.
    if nargin < 3, ttl = ''; end
    if nargin < 4, outdir = ''; end
    figs = {};   % cell, not []: figure() is an object in MATLAB, a double in Octave

    ok  = cellfun(@(s) s.ok, RUNS);
    lbl = cellfun(@(s) s.label, RUNS, 'UniformOutput', false);
    n   = numel(RUNS);
    if SWEEP.on, knob = SWEEP.knob; else, knob = 'single run'; end

    floorM = NaN;
    for i = 1:n
        if RUNS{i}.ok && isfield(RUNS{i}.R,'refSpread'), floorM = RUNS{i}.R.refSpread; break, end
    end

    % ---- F1: ranking, against the floor ---------------------------------------
    p3 = nan(1,n); k3 = nan(1,n);
    for i = 1:n
        if ~ok(i), continue, end
        p3(i) = getm(RUNS{i}.R,'rdo','pos3D');
        k3(i) = getm(RUNS{i}.R,'kin','pos3D');
    end
    if any(isfinite(p3))
        f = figure('Color','w','Name','ranking','Position',[60 60 900 520]);
        hold on; grid on; box on;
        bar([p3(:) k3(:)]);
        if isfinite(floorM)
            xl = [0.5 n+0.5];
            plot(xl, [floorM floorM], 'k--', 'LineWidth', 1.4);
            text(n+0.4, floorM, sprintf('  reference floor %.3f m', floorM), ...
                 'VerticalAlignment','bottom','HorizontalAlignment','right','FontSize',9);
        end
        set(gca,'XTick',1:n,'XTickLabel',lbl,'YScale','log');
        ylabel('position residual |3D| [m RMS]');
        legend('vs reduced-dynamic','vs kinematic','reference floor','Location','best');
        title(sprintf('%s — %s  (bars below the floor are NOT distinguishable)', ttl, knob), ...
              'Interpreter','none');
        figs{end+1} = f;
    end

    % ---- F2: RTN vs time, all configs -----------------------------------------
    if any(ok)
        f = figure('Color','w','Name','RTN vs time','Position',[80 80 1000 640]);
        comp = {'rad','alo','cro'}; nm = {'radial','along','cross'};
        for c = 1:3
            subplot(3,1,c); hold on; grid on; box on;
            for i = 1:n
                if ~ok(i) || ~isfield(RUNS{i}.R,'rdo'), continue, end
                Rr = RUNS{i}.R.rdo;
                plot(Rr.t/60, Rr.(comp{c}), 'LineWidth', 1.0, 'DisplayName', lbl{i});
            end
            ylabel(sprintf('%s [m]', nm{c}));
            if c == 1
                title(sprintf('%s — residual vs reduced-dynamic, per %s', ttl, knob), 'Interpreter','none');
                legend('Location','best','Interpreter','none');
            end
        end
        xlabel('time [min]');
        figs{end+1} = f;
    end

    % ---- F3: the accelerometer gap (a measurement, not a fit) ------------------
    ag = nan(1,n); ar = nan(1,n);
    for i = 1:n
        if ~ok(i) || ~isfield(RUNS{i}.R,'acc'), continue, end
        ag(i) = RUNS{i}.R.acc.rms_diff;
        ar(i) = validation.acc_ratio(RUNS{i}.R);   % bias-removed; see validation.acc_ratio
    end
    if any(isfinite(ag))
        f = figure('Color','w','Name','accelerometer gap','Position',[100 100 900 560]);
        subplot(2,1,1); bar(ag); grid on; box on;
        set(gca,'XTick',1:n,'XTickLabel',lbl); ylabel('|model − meas| [m/s^2 RMS]');
        title(sprintf('%s — non-conservative force gap per %s', ttl, knob), 'Interpreter','none');
        subplot(2,1,2); hold on; bar(ar); grid on; box on;
        plot([0.5 n+0.5], [1 1], 'k--', 'LineWidth', 1.2);
        set(gca,'XTick',1:n,'XTickLabel',lbl); ylabel('model / measured');
        title('1.0 = Cd*A/m and density are both right (or wrong by cancelling amounts)');
        figs{end+1} = f;
    end

    % ---- F4: density ratio -----------------------------------------------------
    dr = nan(1,n); dl = nan(1,n);
    for i = 1:n
        if ~ok(i) || ~isfield(RUNS{i}.R,'den'), continue, end
        dr(i) = RUNS{i}.R.den.ratio;
        dl(i) = RUNS{i}.R.den.rms_logerr;
    end
    if any(isfinite(dr))
        f = figure('Color','w','Name','density','Position',[120 120 900 560]);
        subplot(2,1,1); hold on; bar(dr); grid on; box on;
        plot([0.5 n+0.5], [1 1], 'k--', 'LineWidth', 1.2);
        set(gca,'XTick',1:n,'XTickLabel',lbl); ylabel('median model / measured');
        title(sprintf('%s — density vs measured, per %s', ttl, knob), 'Interpreter','none');
        subplot(2,1,2); bar(dl); grid on; box on;
        set(gca,'XTick',1:n,'XTickLabel',lbl); ylabel('RMS log-error');
        title('log-error penalises the SHAPE of the error, not just its median bias');
        figs{end+1} = f;
    end

    % ---- F5: the table ---------------------------------------------------------
    f = figure('Color','w','Name','summary','Position',[140 140 900 500]);
    axis off;
    L = {sprintf('%s   —   compare_OD   [%s]', ttl, knob), ''};
    L{end+1} = sprintf('%-14s %10s %10s %11s %8s %8s', 'value', 'rdo|3D|', 'kin|3D|', 'acc gap', 'acc r', 'rho r');
    L{end+1} = repmat('-', 1, 66);
    for i = 1:n
        if ~ok(i)
            L{end+1} = sprintf('%-14s FAILED: %s', lbl{i}, RUNS{i}.err);
        else
            L{end+1} = sprintf('%-14s %10.3f %10.3f %11.3e %8.3f %8.3f', ...
                        lbl{i}, p3(i), k3(i), ag(i), ar(i), dr(i));
        end
    end
    L{end+1} = '';
    if isfinite(floorM)
        L{end+1} = sprintf('REFERENCE FLOOR = %.3f m RMS  (kinematic − reducedDynamic)', floorM);
        L{end+1} = '  Rows within this of each other are NOT distinguishable.';
        L{end+1} = '';
    end
    L{end+1} = 'rdo|3D| measures agreement with TU Graz''s force model, not with physics:';
    L{end+1} = 'their reduced-dynamic solution absorbs mismodelling into empirical accelerations.';
    L{end+1} = 'The acc gap and rho ratio are measurements. Prefer them when they disagree.';
    text(0.02, 0.98, L, 'Units','normalized','VerticalAlignment','top', ...
         'FontName','Courier New','FontSize',9, 'Interpreter','none');
    figs{end+1} = f;

    if ~isempty(outdir)
        nmz = {'ranking','rtn_vs_time','acc_gap','density','summary'};
        for i = 1:numel(figs)
            tag = sprintf('fig%d', i);
            if i <= numel(nmz), tag = nmz{i}; end
            print(figs{i}, '-dpng', '-r150', fullfile(outdir, sprintf('compareOD_%s.png', tag)));
        end
    end
end
