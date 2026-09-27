function [figs, tags] = show_OD(R, sol, ttl, outdir, PLOTS)
%SHOW_OD  How is our propagator doing against the real satellite? Figure by figure.
%
%   figs = show_OD(R, sol, ttl, outdir, PLOTS)
%     R      from validation.od_metrics (via validate_OD / compare_OD)
%     sol    the propagation
%     ttl    title prefix, e.g. 'CHAMP 2007-01-01'
%     outdir '' = draw only; else PNGs written here
%     PLOTS  struct of switches -- EVERY figure has one. Nothing is drawn behind
%            your back; omit it and you get the defaults below.
%
%   ---------------------------------------------------------------------------
%   THE KNOBS  (defaults in brackets)
%   ---------------------------------------------------------------------------
%     .rtn        [1]  position residual in RTN, one panel each, + moving average
%     .overlay_r  [1]  |r| ours vs measured OVERLAID, with the error underneath
%     .overlay_v  [1]  |v| ours vs measured, + residual
%     .growth     [1]  how the error GROWS: running RMS + along-track envelope
%     .hist       [1]  residual distribution per RTN axis (bias vs scatter)
%     .acc        [1]  accelerometer: measured vs modelled per axis + error
%     .density    [1]  density: measured (ITSG and/or TU Delft) vs modelled + ratio
%     .tracks     [1]  INDEPENDENT tracks (TU Delft, TLE): |r| overlaid + the error
%                      underneath. These are NOT the OD reference -- see below.
%     .stats      [1]  the numbers, as text
%     .movavg_s [300]  moving-average window [s]; 0 disables it. 300 s is ~1/18 of
%                      a LEO rev: long enough to kill per-sample noise, short enough
%                      to leave once-per-rev signal intact -- which is the signal
%                      you are actually hunting.
%
%   ---------------------------------------------------------------------------
%   HOW TO READ THESE  (the point of drawing them at all)
%   ---------------------------------------------------------------------------
%   * The REFERENCE FLOOR (R.refSpread, grey band) is the reference's OWN
%     uncertainty. Inside the band = not resolvable. Do not tune there.
%   * A residual that GROWS along-track is a force-model error (usually drag): it
%     integrates. One that OSCILLATES once per rev without growing is usually the
%     seed, or geometry/frame.
%   * BIAS vs SCATTER are different diagnoses. Centred and wide = noise, mostly the
%     reference's. Offset and narrow = systematic, and averaging will never remove
%     it. The histograms separate them; the moving mean shows the bias directly.
%   * Radial residual on a near-circular orbit is dominated by the chord-vs-arc
%     effect of any interpolation -- which is why state sampling is Hermite on
%     (r,v) and never pchip. See validation.interp_state.
    if nargin < 3 || isempty(ttl), ttl = ''; end
    if nargin < 4, outdir = ''; end
    if nargin < 5, PLOTS = struct(); end
    % show_OD() with no arguments returns the DEFAULTS and draws nothing. A caller
    % -- or a test -- can then ask "what figures exist and what are they called?"
    % without a fixture, a screen, or a 20 s propagation. The alternative is parsing
    % this file's source for the answer, which is what the old toggle test did, and
    % it broke the moment a tag came from a variable.
    if nargin == 0, figs = defaults(struct()); return, end
    P = defaults(PLOTS);
    % ---- A FIGURE MUST NEVER KILL THE RUN THAT PRODUCED IT ------------------
    % `P = R.rdo.dv_rtn` clobbered the options struct and `P.growth` died 18 lines
    % later -- taking down the WHOLE validation after a 20 s propagation and every
    % download, and taking the other eight figures with it. The plot was a
    % diagnostic; it destroyed the thing it was diagnosing.
    %
    % So each figure is guarded INDIVIDUALLY. A broken panel prints what broke and
    % the rest still render. This is NOT the swallowing-catch pattern this audit has
    % been removing: it REPORTS (file, line, message), it does not discard. The
    % distinction is whether you find out. And plotting code has no business
    % deciding whether a validation run succeeded.
    plotErrs = {};
    figs = {};   % cell, not []: figure() is an object in MATLAB, a double in Octave
    tags = {};   % the NAME of each figure, recorded as it is created. Files used to be
                 % named by POSITION in the list, so turning any figure off shifted
                 % every later filename -- rendering only .tracks wrote OD_rtn.png.
                 % Silently mislabelled output is worse than none: you would have
                 % believed the wrong picture.

    haveRDO = isfield(R,'rdo');
    haveKIN = isfield(R,'kin');
    floorM  = NaN; if isfield(R,'refSpread'), floorM = R.refSpread; end

    % ======================================================= F1: RTN residual ==

    % ============ F1: RTN vs ONE reference -- overlay AND residual, per component
    % One figure per reference. The old figures mixed them or silently used whichever
    % existed, so a curve labelled "error" could not tell you WHAT it disagreed with
    % -- and the reduced-dynamic and kinematic orbits are DIFFERENT MEASUREMENTS.
    % Their mutual difference is the floor under both: a residual smaller than
    % RDO-vs-KIN is not a model error, it is the references arguing.
    %
    % Left column  : the two orbits in RTN, ours vs that reference.
    % Right column : the residual for that component, with mean, +-1 sigma, and the
    %                bias/scatter verdict.
    % Every series on a checkbox: overlay curves AND residual curves.
    for qq = 1:2
        if qq == 1
            tg='rtn_rdo'; fl='rdo'; refname='REDUCED-DYNAMIC';
            why = 'the ITSG dynamic fit -- smooth, model-informed';
        else
            tg='rtn_kin'; fl='kin'; refname='KINEMATIC';
            why = 'GPS geometry ALONE -- no force model, noisier, INDEPENDENT of our physics';
        end
        if ~P.(tg) || ~isfield(R,fl) || ~isfield(R.(fl),'r_ref'), continue, end
        D = R.(fl);
        tt = D.t/60;

        Po = validation.rtn_project(D.r_our, D.r_our, refVel(D));
        Pm = validation.rtn_project(D.r_ref, D.r_our, refVel(D));
        comp = {'radial','along-track','cross-track'};
        note = {'chord-vs-arc / seed', 'DRAG -- this one integrates', 'frame / inclination'};
        res  = {D.rad, D.alo, D.cro};

        f = figure('Color','w','Name',['RTN vs ' refname],'Position',[60 30 1240 940]);
        gO=[]; gM=[]; gR=[]; gRm=[]; gMu=[]; gSg=[];
        for c = 1:3
            % ---- left: the two orbits in this component ------------------------
            subplot(3,2,2*c-1); hold on; grid on; box on;
            gO(end+1) = plot(tt, Po(:,c)/1000, '-', 'Color',[0.20 0.45 0.80],'LineWidth',1.6);
            gM(end+1) = plot(tt, Pm(:,c)/1000, '--','Color',[0.85 0.35 0.15],'LineWidth',1.1);
            ylabel(sprintf('%s [km]', comp{c}));
            if c==1, title(sprintf('OVERLAY: ours vs %s', refname),'FontWeight','normal','FontSize',9); end
            if c==3, xlabel('time [min]'); end

            % ---- right: the residual for this component ------------------------
            subplot(3,2,2*c); hold on; grid on; box on;
            e  = res{c};
            mu = mean(e); sg = std(e);
            xb = [tt(1) tt(end)];
            gSg(end+1) = fill([xb fliplr(xb)], [mu-sg mu-sg mu+sg mu+sg], ...
                              [0.2 0.45 0.8], 'FaceAlpha',0.10,'EdgeColor','none');
            gR(end+1)  = plot(dec(tt,P.decimate), dec(e,P.decimate), '-', ...
                              'Color',[0.45 0.45 0.45],'LineWidth',0.8);
            m = movavg(D.t, e, P.movavg_s);
            if ~isempty(m), gRm(end+1) = plot(tt, m, 'k-','LineWidth',1.7); end
            gMu(end+1) = plot(xb, [mu mu], '-','Color',[0.10 0.30 0.65],'LineWidth',1.3);
            plot(xb,[0 0],'k:');
            ylabel(sprintf('d%s [m]', comp{c}));
            % RMS^2 = mean^2 + std^2: say WHICH regime, do not make the reader divide
            if abs(mu) > 1.5*sg
                v = sprintf('BIAS (|mean| %.1fx std): a force is missing or mis-scaled', abs(mu)/max(sg,eps));
            elseif sg > 1.5*abs(mu)
                v = sprintf('SCATTER (std %.1fx |mean|): oscillation, not a steady force', sg/max(abs(mu),eps));
            else
                v = 'mixed: bias and scatter comparable';
            end
            title(wrapText(sprintf('%s: RMS %.3f = mean %+.3f +- std %.3f m  <- %s. %s', ...
                  comp{c}, rms_(e), mu, sg, note{c}, v), 78), ...
                  'FontWeight','normal','FontSize',8);
            if c==3, xlabel('time [min]'); end
        end
        G = {gO, gM, gR, gMu, gSg}; L = {'ours', ['ref: ' refname], 'residual','mean','+-1 sigma'};
        if ~isempty(gRm), G{end+1}=gRm; L{end+1}=sprintf('%g s moving mean',P.movavg_s); end
        sgt(sprintf('%s - RTN vs %s (%s)  |  3D RMS %.3f m', ttl, refname, why, ...
                    subsref_default(D,'pos3D',NaN)));
        validation.toggle_panel(f, G, L);
        figs{end+1} = f; tags{end+1} = tg;
    end

    % ==================== F1v: RTN VELOCITY -- overlay AND residual ============
    % Reduced-dynamic only, and the title says why: the kinematic product is
    % MJD,x,y,z. POSITION ONLY. There is no kinematic velocity in existence, so a
    % velocity-vs-kinematic panel cannot be drawn. Saying that beats leaving a gap
    % where a panel should be -- a missing panel looks like an oversight, an absent
    % product is a fact about the data, and only one of those is worth chasing.
    if P.rtn_v && isfield(R,'rdo') && isfield(R.rdo,'v_ref')
        D = R.rdo; tt = D.t/60;
        Vo = validation.rtn_project(D.v_our, D.r_our, D.v_our);
        Vm = validation.rtn_project(D.v_ref, D.r_our, D.v_our);
        dV = validation.rtn_project(D.v_our - D.v_ref, D.r_our, D.v_our);
        comp = {'radial','along-track','cross-track'};
        note = {'dv_r = -n*x_along: TRIAD ROTATION, not a force', ...
                'where a drag error goes', 'frame / inclination'};
        f = figure('Color','w','Name','RTN velocity','Position',[80 30 1240 940]);
        gO=[]; gM=[]; gR=[]; gRm=[]; gMu=[];
        for c = 1:3
            subplot(3,2,2*c-1); hold on; grid on; box on;
            gO(end+1) = plot(tt, Vo(:,c), '-', 'Color',[0.20 0.45 0.80],'LineWidth',1.6);
            gM(end+1) = plot(tt, Vm(:,c), '--','Color',[0.85 0.35 0.15],'LineWidth',1.1);
            ylabel(sprintf('v_{%s} [m/s]', comp{c}));
            if c==1, title('OVERLAY: ours vs REDUCED-DYNAMIC','FontWeight','normal','FontSize',9); end
            if c==3, xlabel('time [min]'); end

            subplot(3,2,2*c); hold on; grid on; box on;
            e  = dV(:,c)*1000;              % mm/s
            mu = mean(e); sg = std(e);
            gR(end+1) = plot(dec(tt,P.decimate), dec(e,P.decimate), '-', ...
                             'Color',[0.45 0.45 0.45],'LineWidth',0.8);
            m = movavg(D.t, e, P.movavg_s);
            if ~isempty(m), gRm(end+1) = plot(tt, m, 'k-','LineWidth',1.7); end
            gMu(end+1) = plot([tt(1) tt(end)],[mu mu],'-','Color',[0.85 0.35 0.15],'LineWidth',1.3);
            plot([tt(1) tt(end)],[0 0],'k:');
            ylabel(sprintf('dv_{%s} [mm/s]', comp{c}));
            title(wrapText(sprintf('%s residual: RMS %.4f = mean %+.4f +- std %.4f mm/s  <- %s', ...
                  comp{c}, rms_(e), mu, sg, note{c}), 78),'FontWeight','normal','FontSize',8);
            if c==3, xlabel('time [min]'); end
        end
        G = {gO, gM, gR, gMu}; L = {'ours','reduced-dynamic','residual','mean'};
        if ~isempty(gRm), G{end+1}=gRm; L{end+1}=sprintf('%g s moving mean',P.movavg_s); end
        sgt(sprintf(['%s - RTN velocity: overlay + residual vs REDUCED-DYNAMIC. ' ...
             'There is NO kinematic velocity: that product is MJD,x,y,z -- position only.'], ttl));
        validation.toggle_panel(f, G, L);
        figs{end+1} = f; tags{end+1} = 'rtn_v';
    end

    % ==================== F1g: per-axis RTN error GROWTH, both refs ============
    % A value cannot distinguish forces; a GROWTH RATE can. Drag integrates twice, so
    % its along-track error goes as t^2. A seed or frame error is flat or linear. The
    % running RMS of each component, per reference, on one figure -- so you can see
    % WHICH component is growing and whether BOTH references agree that it is.
    %
    % If only one reference shows the growth, it is that reference drifting, not our
    % model. That is a distinction no single-reference figure can make.
    if P.rtn_growth && isfield(R,'rdo')
        f = figure('Color','w','Name','RTN error growth','Position',[100 30 1240 880]);
        comp = {'radial','along-track','cross-track'};
        cols = [0.20 0.45 0.80; 0.85 0.35 0.15];
        G = {}; L = {};
        for c = 1:3
            subplot(3,1,c); hold on; grid on; box on;
            hh = [];
            for qq = 1:2
                if qq==1, fl='rdo'; nmq='vs reduced-dynamic'; else, fl='kin'; nmq='vs kinematic'; end
                if ~isfield(R,fl), continue, end
                D = R.(fl);
                if ~isfield(D,'rad'), continue, end
                ee = {D.rad, D.alo, D.cro};
                rr = runningRMS(ee{c});
                hh(end+1) = plot(D.t/60, rr, '-','Color',cols(qq,:),'LineWidth',1.6); %#ok<AGROW>
                if c==1, G{end+1} = hh(end); L{end+1} = nmq; end %#ok<AGROW>
            end
            ylabel(sprintf('running RMS %s [m]', comp{c}));
            % the growth verdict, per component
            D = R.rdo; ee = {D.rad, D.alo, D.cro}; y = abs(ee{c});
            if numel(y) > 8
                h1 = mean(y(1:floor(end/2))); h2 = mean(y(floor(end/2)+1:end));
                if h1 > 0
                    g = h2/h1;
                    if g > 1.6, vv = sprintf('GROWING %.1fx: an integrating force (drag)', g);
                    elseif g < 0.7, vv = sprintf('SHRINKING %.1fx: transient, likely the seed', g);
                    else, vv = sprintf('FLAT (%.1fx): not integrating -- seed or frame, not drag', g); end
                else
                    vv = '';
                end
            else
                vv = '';
            end
            title(sprintf('%s: %s', comp{c}, vv),'FontWeight','normal','FontSize',9);
            if c==3, xlabel('time [min]'); end
        end
        sgt(sprintf(['%s - RTN error GROWTH per component. Drag goes as t^2; a seed or ' ...
             'frame error is flat. If only ONE reference grows, it is that reference ' ...
             'drifting, not our model.'], ttl));
        validation.toggle_panel(f, G, L);
        figs{end+1} = f; tags{end+1} = 'rtn_growth';
    end

    % ==================================== F1b: POSITION vector, BOTH refs =====
    % Left column  : the OVERLAY -- model, reduced-dynamic, kinematic, per axis.
    % Right column : the ERROR against EACH reference, per axis.
    %
    % Both references, on the same axes, because they are INDEPENDENT truths and
    % their disagreement is the floor under everything: if model-vs-RDO and
    % model-vs-KIN differ by more than RDO-vs-KIN, the model is the odd one out; if
    % they track each other, you are looking at one error, not two. Plotting them on
    % separate figures makes that comparison impossible, which is the whole reason
    % they are here together.
    if P.vec_r && haveRDO && isfield(R.rdo,'r_ref')
        f = figure('Color','w','Name','position vector: overlay + error vs both refs', ...
                   'Position',[100 30 1180 940]);
        t   = R.rdo.t/60;
        rr  = R.rdo.r_our;  rm = R.rdo.r_ref;
        haveK = isfield(R,'kin') && isfield(R.kin,'r_ref') && ~isempty(R.kin.r_ref);
        if haveK, tk = R.kin.t/60; rk = R.kin.r_ref; rok = R.kin.r_our; end
        axn = {'x','y','z'};
        gOur = []; gRdo = []; gKin = []; gErdo = []; gEkin = [];
        for c = 1:3
            subplot(3,2,2*c-1); hold on; grid on; box on;
            gOur(end+1) = plot(t, rr(:,c)/1000, '-', 'Color',[0.20 0.45 0.80],'LineWidth',1.6);
            gRdo(end+1) = plot(t, rm(:,c)/1000, '--','Color',[0.85 0.35 0.15],'LineWidth',1.1);
            if haveK
                gKin(end+1) = plot(tk, rk(:,c)/1000, ':','Color',[0.15 0.55 0.25],'LineWidth',1.3);
            end
            ylabel(sprintf('r_%s [km]', axn{c}));
            if c==1, title('OVERLAY: model / reduced-dynamic / kinematic','FontWeight','normal','FontSize',9); end
            if c==3, xlabel('time [min]'); end

            subplot(3,2,2*c); hold on; grid on; box on;
            e1 = rr(:,c) - rm(:,c);
            gErdo(end+1) = plot(t, e1, '-','Color',[0.85 0.35 0.15],'LineWidth',1.0);
            lab = {sprintf('vs RDO: %+.3f +- %.3f m', mean(e1), std(e1))};
            if haveK
                e2 = rok(:,c) - rk(:,c);
                gEkin(end+1) = plot(tk, e2, '-','Color',[0.15 0.55 0.25],'LineWidth',1.0);
                lab{end+1} = sprintf('vs KIN: %+.3f +- %.3f m', mean(e2), std(e2));
            end
            plot([t(1) t(end)],[0 0],'k:');
            ylabel(sprintf('dr_%s [m]', axn{c}));
            title(strjoin(lab, '  |  '),'FontWeight','normal','FontSize',8);
            if c==3, xlabel('time [min]'); end
        end
        G = {gOur, gRdo, gErdo}; L = {'model','reduced-dynamic','error vs RDO'};
        if haveK, G{end+1} = gKin; L{end+1} = 'kinematic'; G{end+1} = gEkin; L{end+1} = 'error vs KIN'; end
        sgt(sprintf('%s - position: overlay + error vs BOTH references', ttl));
        validation.toggle_panel(f, G, L);
        figs{end+1} = f; tags{end+1} = 'vec_r';
    end

    % ==================================== F1c: VELOCITY vector ================
    % Reduced-dynamic ONLY, and the title says why: the kinematic product is
    % MJD,x,y,z -- POSITION ONLY. There is no kinematic velocity in existence to
    % compare against. Stating that beats omitting the curve, because a missing
    % panel looks like an oversight while an absent product is a fact about the
    % data, and only one of those is worth investigating.
    if P.vec_v && haveRDO && isfield(R.rdo,'v_ref')
        f = figure('Color','w','Name','velocity vector: overlay + error', ...
                   'Position',[120 30 1180 940]);
        t  = R.rdo.t/60;
        vv = R.rdo.v_our;  vm = R.rdo.v_ref;
        axn = {'x','y','z'};
        gOur = []; gRdo = []; gErr = []; gMean = [];
        for c = 1:3
            subplot(3,2,2*c-1); hold on; grid on; box on;
            gOur(end+1) = plot(t, vv(:,c)/1000, '-', 'Color',[0.20 0.45 0.80],'LineWidth',1.6);
            gRdo(end+1) = plot(t, vm(:,c)/1000, '--','Color',[0.85 0.35 0.15],'LineWidth',1.1);
            ylabel(sprintf('v_%s [km/s]', axn{c}));
            if c==1, title('OVERLAY: model vs reduced-dynamic','FontWeight','normal','FontSize',9); end
            if c==3, xlabel('time [min]'); end

            subplot(3,2,2*c); hold on; grid on; box on;
            e = (vv(:,c) - vm(:,c))*1000;
            gErr(end+1)  = plot(t, e, '-','Color',[0.35 0.35 0.35],'LineWidth',0.9);
            mu = mean(e); sg = std(e);
            gMean(end+1) = plot([t(1) t(end)],[mu mu],'-','Color',[0.85 0.35 0.15],'LineWidth',1.3);
            m = movavg(R.rdo.t, e, P.movavg_s);
            if ~isempty(m), plot(t, m, 'k-','LineWidth',1.4); end
            plot([t(1) t(end)],[0 0],'k:');
            ylabel(sprintf('dv_%s [mm/s]', axn{c}));
            title(sprintf('vs RDO: mean %+.4f +- std %.4f mm/s', mu, sg), ...
                  'FontWeight','normal','FontSize',9);
            if c==3, xlabel('time [min]'); end
        end
        sgt(sprintf(['%s - velocity: overlay + error vs reduced-dynamic ' ...
                     '(no KINEMATIC velocity exists: that product is position-only)'], ttl));
        validation.toggle_panel(f, {gOur, gRdo, gErr, gMean}, ...
                                  {'model','reduced-dynamic','error','mean'});
        figs{end+1} = f; tags{end+1} = 'vec_v';
    end

    % ============================================ F1d: RTN overlay ============
    % The RTN OVERLAY, as distinct from the RTN RESIDUAL figure: here the two
    % ORBITS are projected into the triad and drawn against each other, so you see
    % what the residual is a difference OF. It is the same argument as the vector
    % overlay -- a residual alone cannot tell you whether both series are wrong.

    % ================================================ F2: |r| overlay + error ==
    % The overlay answers "are we on the same orbit". At metres-on-6800-km it will
    % ALWAYS look perfect -- which is exactly why the error panels below are not
    % optional. Never show the overlay alone.

    % ================================================ F3: |v| overlay + error ==
    if P.overlay_v && haveRDO
        f = figure('Color','w','Name','v overlay','Position',[80 80 1000 500]);
        t  = R.rdo.t/60;
        vn = vecnorm(sol.v,2,2);
        subplot(2,1,1); hold on; grid on; box on;
        plot(t, vn/1000, '-','Color',[0.20 0.45 0.80],'LineWidth',1.4);
        ylabel('|v| [km/s]'); legend({'ours'},'Location','best');
        title('velocity magnitude','FontWeight','normal');
        subplot(2,1,2); hold on; grid on; box on;
        if isfield(R.rdo,'dv_rtn') && ~isempty(R.rdo.dv_rtn)
            % The SERIES in RTN. A flat line at a scalar RMS (what this drew before)
            % tells you nothing: the residual's SHAPE is the diagnostic. A constant
            % offset is a bias; a linear ramp is a drag scale error; a once-per-rev
            % oscillation is a frame or phase error. The RMS collapses all three into
            % one number that cannot tell them apart.
            % Vrtn, NOT P. `P = defaults(PLOTS)` is the options struct for this whole
            % function (line ~47) and I overwrote it with a matrix here, so `P.growth`
            % died 18 lines later with "Dot indexing is not supported". The figure that
            % crashed was not even the one I broke.
            %
            % Reusing a short name inside a long function is the same bug as inventing
            % one: the difference is that MATLAB will not warn about either until the
            % line runs. Long functions make it likelier, single letters make it
            % invisible.
            Vrtn = R.rdo.dv_rtn;
            plot(t, Vrtn(:,1)*1000, '-', 'LineWidth',1.2, 'Color',[0.20 0.45 0.80]); hold on;
            plot(t, Vrtn(:,2)*1000, '-', 'LineWidth',1.4, 'Color',[0.85 0.33 0.10]);
            plot(t, Vrtn(:,3)*1000, '-', 'LineWidth',1.2, 'Color',[0.15 0.55 0.25]);
            legend({'radial','along-track','cross-track'},'Location','best','FontSize',7);
            ylabel('dv [mm/s]');
            title(sprintf(['velocity residual in RTN  (RMS %.4f m/s). Along-track is where a ' ...
                   'drag error goes.'], R.rdo.dv),'FontWeight','normal');
        end
        xlabel('time [min]');
        sgt(sprintf('%s - velocity', ttl));
        figs{end+1} = f; tags{end+1} = 'overlay_v';
    end

    % ==================================================== F4: error growth ====
    % Does the error GROW? The single most diagnostic question here. Drag
    % mismodelling integrates into along-track and grows; a seed error or a
    % geometry problem oscillates and does not.
    if P.growth && haveRDO
        f = figure('Color','w','Name','error growth','Position',[100 100 1000 560]);
        t = R.rdo.t/60;
        d3 = sqrt(R.rdo.rad.^2 + R.rdo.alo.^2 + R.rdo.cro.^2);
        runRMS = sqrt(cumsum(d3.^2)./(1:numel(d3)).');
        subplot(2,1,1); hold on; grid on; box on;
        drawFloor(t, floorM);
        plot(t, d3,     '-', 'Color',[0.75 0.75 0.75], 'LineWidth',0.6);
        plot(t, runRMS, 'k-', 'LineWidth',1.8);
        ylabel('|error| [m]'); legend({'reference floor','|error| per epoch','running RMS'},'Location','best');
        title('running RMS climbing = a force is mismodelled; flattening = noise','FontWeight','normal');
        subplot(2,1,2); hold on; grid on; box on;
        plot(t, abs(R.rdo.alo), '-','Color',[0.85 0.35 0.15],'LineWidth',0.7);
        ma = movavg(R.rdo.t, abs(R.rdo.alo), P.movavg_s);
        if ~isempty(ma), plot(t, ma, 'k-','LineWidth',1.6); end
        ylabel('|along-track| [m]'); xlabel('time [min]');
        legend({'|along-track|','moving mean'},'Location','best');
        title('along-track is where drag error accumulates - watch this one first','FontWeight','normal');
        sgt(sprintf('%s - error growth', ttl));
        figs{end+1} = f; tags{end+1} = 'growth';
    end

    % ================================================= F5: distributions =====
    if P.hist && haveRDO
        f = figure('Color','w','Name','distribution','Position',[120 120 1000 380]);
        comp = {'rad','alo','cro'}; nm = {'radial','along-track','cross-track'};
        for c = 1:3
            subplot(1,3,c); hold on; grid on; box on;
            y = R.rdo.(comp{c});
            % P.hist_bins, not a hard-coded 40. The knob existed and the ONE figure
            % that draws a histogram ignored it -- so setting it did nothing and you
            % would have concluded the bin count does not matter. It does: too few
            % bins hide a bimodal residual (two regimes, e.g. eclipse vs sunlit)
            % behind one fat peak; too many turn the distribution into a comb and you
            % read sample noise as structure.
            nb = P.hist_bins;
            if nb <= 0, nb = max(12, min(40, round(numel(y)/12))); end
            hist(y, nb);
            mu = mean(y); sd = std(y); yl = ylim;
            plot([mu mu], yl, 'r-', 'LineWidth',2);
            plot([0 0],   yl, 'k:', 'LineWidth',1.2);
            xlabel(sprintf('%s [m]', nm{c}));
            if c==1, ylabel('count'); end
            title(sprintf('bias %+.3f m / scatter %.3f m', mu, sd),'FontWeight','normal');
        end
        sgt(sprintf('%s - distribution: red = mean (BIAS), dotted = zero. Offset mean = systematic.', ttl));
        figs{end+1} = f; tags{end+1} = 'hist';
    end

    % ================================================== F6: accelerometer ====



    % ================================ F4b: accelerometer RAW + the bias =======
    % The INSTRUMENT story: what the accelerometer actually returns, before any
    % correction. The dashed line is the measured MEAN -- the bias -- drawn next to
    % the signal it swamps, which is the only way to SEE why an RMS about zero is
    % mostly the offset.
    %
    % Every other accelerometer figure works on bias-REMOVED data. This is the one
    % that shows what was removed and how big it was, and without it the correction
    % is something you have to take on trust.
    if P.acc_raw && isfield(R,'acc') && isfield(R.acc,'meas')
        f = figure('Color','w','Name','accelerometer: RAW + bias','Position',[140 30 1180 880]);
        t = R.acc.t/60;
        hasBias = isfield(R.acc,'bias_meas');
        axn = {'x','y','z'};
        gMe=[]; gMo=[]; gB=[];
        for c = 1:3
            subplot(3,1,c); hold on; grid on; box on;
            gMe(end+1) = plot(t, R.acc.meas(:,c), '-','Color',[0.85 0.15 0.15],'LineWidth',1.0);
            gMo(end+1) = plot(t, R.acc.mod(:,c),  '-','Color',[0.20 0.45 0.80],'LineWidth',1.4);
            if hasBias
                gB(end+1) = plot([t(1) t(end)], R.acc.bias_meas(c)*[1 1], '--', ...
                                 'Color',[0.5 0.1 0.5],'LineWidth',1.5);
            end
            plot([t(1) t(end)],[0 0],'k:');
            ylabel(sprintf('a_%s [m/s^2]  RAW, ECI', axn{c}));
            if hasBias
                title(sprintf('%s:  measured mean (bias) = %+.3e m/s^2', axn{c}, R.acc.bias_meas(c)), ...
                      'FontWeight','normal','FontSize',9);
            else
                title(sprintf('%s: measured vs modelled, RAW', axn{c}),'FontWeight','normal','FontSize',9);
            end
            if c==3, xlabel('time [min]'); end
        end
        G = {gMe, gMo}; L = {'MEASURED (raw)','modelled'};
        if ~isempty(gB), G{end+1}=gB; L{end+1}='measured MEAN = bias'; end
        validation.toggle_panel(f, G, L);
        if hasBias
            sgt(sprintf(['%s - accelerometer RAW, ECI axes: |bias| = %.2e = %.1fx the ' ...
                'modelled signal.  MEASURED IN THE SATELLITE BODY FRAME, rotated to ECI by ' ...
                'the fetched attitude quaternions -- so the PER-AXIS split inherits the ' ...
                'attitude product''s error; the MAGNITUDE does not.'], ...
                ttl, norm(R.acc.bias_meas), norm(R.acc.bias_meas)/max(subsref_default(R.acc,'rms_ac_mod',NaN),eps)));
        else
            sgt(sprintf('%s - accelerometer RAW, ECI axes (rotated from the body frame)', ttl));
        end
        figs{end+1} = f; tags{end+1} = 'acc_raw';
    end

    % ================================ F4c: WHICH FORCE dominates ==============
    % Magnitudes, LOG axis, with the modelled SUM and the MEASUREMENT overlaid.
    %
    % Log because at 480 km drag and SRP are comparable while ERP is an order down,
    % and in eclipse SRP falls to zero -- three orders of magnitude on one plot. A
    % linear axis shows the biggest curve and a flat line for everything else; the
    % per-axis figures are linear (they must be: the components change sign) and so
    % cannot answer "which force is doing the work" at all.
    if P.acc_forces && isfield(R,'acc') && isfield(R.acc,'cmp')
        f = figure('Color','w','Name','accelerometer: which force?','Position',[180 30 1180 620]);
        t = R.acc.t/60;
        hasAC = isfield(R.acc,'ac_meas');
        if hasAC, Am = R.acc.ac_meas; Ao = R.acc.ac_mod; else, Am = R.acc.meas; Ao = R.acc.mod; end
        hold on; grid on; box on;
        cols = struct('drag',[0.15 0.55 0.25],'srp',[0.85 0.60 0.10],'erp',[0.55 0.25 0.65]);
        G = {}; L = {};
        for nm = {'drag','srp','erp'}
            if isfield(R.acc.cmp, nm{1}) && any(R.acc.cmp.(nm{1})(:) ~= 0)
                h = plot(t, max(vecnorm(R.acc.cmp.(nm{1}),2,2),1e-14), '-', ...
                         'Color',cols.(nm{1}),'LineWidth',1.3);
                G{end+1} = h; L{end+1} = nm{1};
            end
        end
        hS = plot(t, max(vecnorm(Ao,2,2),1e-14), '-','Color',[0.20 0.45 0.80],'LineWidth',2.0);
        hM = plot(t, max(vecnorm(Am,2,2),1e-14), '--','Color',[0.85 0.15 0.15],'LineWidth',1.6);
        G = [G, {hS, hM}]; L = [L, {'modelled SUM','MEASURED'}];
        set(gca,'YScale','log');
        ylabel('|a| [m/s^2]'); xlabel('time [min]');
        legend(L,'Location','best','FontSize',7);
        title(wrapText(['each force, the modelled SUM, and the MEASUREMENT' ...
               subsref_tern(hasAC,' (bias-removed)','') ...
               '.  "the non-grav is 10% low" is not actionable -- these are three ' ...
               'physics with three fixes.'], 95),'FontWeight','normal','FontSize',9);
        sgt(sprintf('%s - which force is doing the work?', ttl));
        validation.toggle_panel(f, G, L);
        figs{end+1} = f; tags{end+1} = 'acc_forces';
    end

    % ============================ F4d: accelerometer PER AXIS, FULL BREAKDOWN ==
    % Per axis, three things on one plot:
    %   1. every modelled force separately  (drag, SRP, ERP)
    %   2. their SUM, and the MEASURED accelerometer, overlaid
    %   3. delta_a = modelled_sum - measured  =  THE UNMODELLED ACCELERATION
    %
    % Per AXIS and not magnitude, because the forces do not share a direction: drag
    % is along-track, SRP points away from the Sun, ERP comes off the Earth. A
    % magnitude ratio of 0.53 could be a uniform 47% shortfall in everything, or
    % drag correct and SRP missing entirely -- OPPOSITE fixes. Only the split can
    % tell them apart, and delta_a per axis says WHICH DIRECTION the missing force
    % points, which is the strongest clue available about what it is.
    %
    % Linear axis, not log: delta_a CHANGES SIGN, and a log axis cannot show that.
    % The sign is the whole point -- it says whether the model is over or under, and
    % on which axis.
    if P.acc_axes && isfield(R,'acc') && isfield(R.acc,'meas')
        f = figure('Color','w','Name','accelerometer: per-axis breakdown + unmodelled', ...
                   'Position',[160 30 1200 940]);
        t  = R.acc.t/60;
        hasAC = isfield(R.acc,'ac_meas') && isfield(R.acc,'ac_mod');
        if hasAC, Am = R.acc.ac_meas; Ao = R.acc.ac_mod; else, Am = R.acc.meas; Ao = R.acc.mod; end
        haveC = isfield(R.acc,'cmp');
        axn = {'x','y','z'};
        cols = struct('drag',[0.15 0.55 0.25],'srp',[0.85 0.60 0.10],'erp',[0.55 0.25 0.65]);
        gD=[]; gS=[]; gE=[]; gSum=[]; gMeas=[]; gDel=[]; gDelM=[];
        for c = 1:3
            % ---- left: the forces, the sum, and the measurement ----------------
            subplot(3,2,2*c-1); hold on; grid on; box on;
            if haveC
                if isfield(R.acc.cmp,'drag'), gD(end+1) = plot(t, R.acc.cmp.drag(:,c), '-','Color',cols.drag,'LineWidth',1.0); end
                if isfield(R.acc.cmp,'srp'),  gS(end+1) = plot(t, R.acc.cmp.srp(:,c),  '-','Color',cols.srp, 'LineWidth',1.0); end
                if isfield(R.acc.cmp,'erp'),  gE(end+1) = plot(t, R.acc.cmp.erp(:,c),  '-','Color',cols.erp, 'LineWidth',1.0); end
            end
            gSum(end+1)  = plot(t, Ao(:,c), '-', 'Color',[0.20 0.45 0.80],'LineWidth',1.8);
            gMeas(end+1) = plot(t, Am(:,c), '--','Color',[0.85 0.15 0.15],'LineWidth',1.4);
            plot([t(1) t(end)],[0 0],'k:');
            ylabel(sprintf('a_%s [m/s^2]', axn{c}));
            if c==1
                title(['each FORCE, their SUM, and the MEASUREMENT' ...
                       subsref_tern(hasAC,' (bias-removed)','')],'FontWeight','normal','FontSize',9);
            end
            if c==3, xlabel('time [min]'); end

            % ---- right: delta_a = the unmodelled acceleration ------------------
            subplot(3,2,2*c); hold on; grid on; box on;
            da = Ao(:,c) - Am(:,c);
            gDel(end+1) = plot(dec(t,P.decimate), dec(da,P.decimate), '-','Color',[0.35 0.35 0.35],'LineWidth',0.9);
            m = movavg(R.acc.t, da, P.movavg_s);
            if ~isempty(m), gDelM(end+1) = plot(t, m, 'k-','LineWidth',1.7); end
            mu = mean(da); sg = std(da);
            plot([t(1) t(end)],[mu mu],'-','Color',[0.85 0.35 0.15],'LineWidth',1.3);
            plot([t(1) t(end)],[0 0],'k:');
            ylabel(sprintf('\\Deltaa_%s [m/s^2]', axn{c}));
            % price the mean in metres: an acceleration nobody can weigh against a
            % 2 m residual is not an actionable number.
            T_ = max(R.acc.t) - min(R.acc.t);
            title(sprintf('UNMODELLED  \\Deltaa = model - measured:  mean %+.2e +- %.2e  (=%.2f m over %.0f s)', ...
                  mu, sg, 0.5*abs(mu)*T_^2, T_),'FontWeight','normal','FontSize',8);
            if c==3, xlabel('time [min]'); end
        end
        G = {}; L = {};
        if ~isempty(gD), G{end+1}=gD; L{end+1}='drag'; end
        if ~isempty(gS), G{end+1}=gS; L{end+1}='SRP'; end
        if ~isempty(gE), G{end+1}=gE; L{end+1}='ERP'; end
        G = [G, {gSum, gMeas, gDel}];  L = [L, {'modelled SUM','MEASURED','\Deltaa unmodelled'}];
        if ~isempty(gDelM), G{end+1}=gDelM; L{end+1}='\Deltaa moving mean'; end
        sgt(sprintf(['%s - accelerometer per ECI axis: forces, sum vs measured, and the ' ...
             'unmodelled remainder.  ITSG nonConservativeForces is measured in the SATELLITE ' ...
             'BODY frame and rotated here by the measured attitude, so a bad attitude moves ' ...
             'signal BETWEEN these panels without changing the total.'], ttl));
        validation.toggle_panel(f, G, L);
        figs{end+1} = f; tags{end+1} = 'acc_axes';
    end

    % ============================ F4e: THE SCALE FACTOR =======================
    % The question this figure answers:
    %
    %     rho * Cd * A/m is ONE number. Cd is not measured for any satellite in this
    %     catalog -- it is a literature value somebody typed -- and Aref is one
    %     number standing in for a whole shape. So a constant multiplicative error
    %     is EXPECTED and is not interesting: it tells you one of four factors is
    %     off, which you already knew.
    %
    %     The interesting question is what SURVIVES the best possible scale factor,
    %     because that part cannot be fixed by any Cd, any area, any density
    %     scaling. It is structure -- a missing force, a wrong direction, a phase
    %     error -- and it is the only part of the residual that is evidence about
    %     the MODEL rather than about the bookkeeping.
    %
    % Per axis: k is fitted per axis, because the forces do not share a direction
    % and a single k across all three would silently average a drag error on one
    % axis against an SRP error on another.
    if P.acc_scale && isfield(R,'acc') && isfield(R.acc,'meas')
        f = figure('Color','w','Name','accelerometer: scale factor', ...
                   'Position',[200 30 1200 940]);
        t = R.acc.t/60;
        hasAC = isfield(R.acc,'ac_meas') && isfield(R.acc,'ac_mod');
        if hasAC, Am = R.acc.ac_meas; Ao = R.acc.ac_mod; else, Am = R.acc.meas; Ao = R.acc.mod; end
        axn = {'x','y','z'};
        gM=[]; gK=[]; gU=[]; gD=[]; gDm=[];
        T_ = max(R.acc.t) - min(R.acc.t);
        for c = 1:3
            SF = validation.scale_factor(Am(:,c), Ao(:,c), ...
                     struct('name',sprintf('a_%s',axn{c}),'unit','m/s^2'));

            % ---- left: measured vs k*model (and the unscaled model, for contrast)
            subplot(3,2,2*c-1); hold on; grid on; box on;
            gM(end+1) = plot(t, Am(:,c), '-', 'Color',[0.85 0.15 0.15],'LineWidth',1.6);
            gU(end+1) = plot(t, Ao(:,c), ':', 'Color',[0.55 0.55 0.55],'LineWidth',1.1);
            gK(end+1) = plot(t, SF.factor*Ao(:,c), '-','Color',[0.20 0.45 0.80],'LineWidth',1.4);
            plot([t(1) t(end)],[0 0],'k:');
            ylabel(sprintf('a_%s [m/s^2]', axn{c}));
            title(sprintf('%s:  k = %.3f   r2 = %.3f   (scale absorbs %.0f%% of the residual)', ...
                  axn{c}, SF.factor, SF.r2, 100*max(SF.explained,0)), ...
                  'FontWeight','normal','FontSize',9);
            if c==3, xlabel('time [min]'); end

            % ---- right: what the scale factor CANNOT explain --------------------
            subplot(3,2,2*c); hold on; grid on; box on;
            da = SF.resid;                       % = measured - k*model
            gD(end+1) = plot(dec(t,P.decimate), dec(da,P.decimate), '-','Color',[0.35 0.35 0.35],'LineWidth',0.9);
            m = movavg(R.acc.t, da, P.movavg_s);
            if ~isempty(m), gDm(end+1) = plot(t, m, 'k-','LineWidth',1.7); end
            mu = mean(da); sg = std(da);
            plot([t(1) t(end)],[mu mu],'-','Color',[0.85 0.35 0.15],'LineWidth',1.3);
            plot([t(1) t(end)],[0 0],'k:');
            ylabel(sprintf('\\Deltaa_%s [m/s^2]', axn{c}));
            % the mean priced in metres: an acceleration is not a quantity anyone can
            % weigh against a 2 m position residual in their head
            title(sprintf('UNMODELLED: measured - k*model.  mean %+.2e +- %.2e  (=%.2f m/arc)', ...
                  mu, sg, 0.5*abs(mu)*T_^2), 'FontWeight','normal','FontSize',8);
            if c==3, xlabel('time [min]'); end

            if c==1
                % the interpretation, once, where it cannot be missed
                text(0.02, 0.02, SF.note, 'Units','normalized', 'FontSize',7, ...
                     'Color',[0.35 0.15 0.15], 'VerticalAlignment','bottom');
            end
        end
        sgt(sprintf(['%s - SCALE FACTOR: what survives the best k?  ' ...
             '(rho*Cd*A/m is ONE number -- a constant error is expected and dull; ' ...
             'the remainder is not)'], ttl));
        validation.toggle_panel(f, {gM, gK, gU, gD}, ...
            {'MEASURED','k x model','model (unscaled)','\Deltaa = meas - k*model'});
        figs{end+1} = f; tags{end+1} = 'acc_scale';
    end

    % ============================== F5: density -- ONE FIGURE PER SOURCE ======
    % Two INDEPENDENT truths, so two figures -- not two rows of one figure.
    %
    %   ITSG neutralDensity_1.0 : same server, same processing, same folder as the
    %                             ORBIT you are validating against. Exists for 7 of
    %                             the 24 satellites here.
    %   TU Delft                : a different group, a different pipeline, its own
    %                             coverage. Exists for 8.
    %   BOTH                    : only 5.
    %
    % They are not two views of one measurement, they are two measurements. Sharing a
    % figure invites reading one as a correction to the other. Separate figures, with
    % IDENTICAL panels and IDENTICAL arithmetic, so the only thing that differs
    % between them is the data -- which is the whole point of having two.
    dsrc = {};
    if isfield(R,'den'), dsrc{end+1} = {'den','density_itsg', ...
        'ITSG neutralDensity_1.0 -- same server/processing as the orbit'}; end
    if isfield(R,'tud'), dsrc{end+1} = {'tud','density_tud', ...
        'TU Delft -- independent group, independent pipeline'}; end

    for q = 1:numel(dsrc)
        fld = dsrc{q}{1}; tg = dsrc{q}{2}; nm = dsrc{q}{3};
        if ~P.(tg), continue, end
        D  = R.(fld);
        td = D.t/60;
        me = D.meas(:); mo = D.mod(:);

        % ONE calculation, used by both figures. k = measured/model: "what must I
        % multiply MY MODEL by to match the truth" -- the number you would apply.
        SF   = validation.scale_factor(me, mo, struct('name',nm,'unit','kg/m^3'));
        rat  = me ./ max(mo, realmin);
        lim  = P.ratio_lim;
        bad  = ~isfinite(rat) | rat < lim(1) | rat > lim(2);
        % BIAS in the log domain: density spans decades, so a difference in kg/m^3 is
        % meaningless (1e-13 vs 1e-12 is the same "size of wrong" as 1e-14 vs 1e-13).
        % mean(log(measured/model)) is the bias that respects that.
        lg   = log(max(rat(~bad), realmin));
        biasFac = exp(mean(lg));            % multiplicative bias
        sprd    = exp(std(lg));             % multiplicative spread

        f = figure('Color','w','Name',['density: ' tg],'Position',[220 30 1180 880]);

        % ---- 1: the two curves, plus the scaled model ------------------------
        subplot(3,1,1); hold on; grid on; box on;
        gMe = plot(td, me, '-', 'Color',[0.85 0.15 0.15],'LineWidth',1.3);
        gMo = plot(td, mo, '-', 'Color',[0.20 0.45 0.80],'LineWidth',1.3);
        gK  = plot(td, SF.factor*mo, '--','Color',[0.10 0.60 0.30],'LineWidth',1.3);
        set(gca,'YScale','log'); ylabel('rho [kg/m^3]');
        title(sprintf('measured vs model.  SCALE FACTOR k = %.3f  (r2 %.2f)', SF.factor, SF.r2), ...
              'FontWeight','normal','FontSize',9);

        % ---- 2: measured / model through the arc ------------------------------
        subplot(3,1,2); hold on; grid on; box on;
        gR = plot(dec(td,P.decimate), dec(rat,P.decimate), '-','Color',[0.10 0.60 0.30],'LineWidth',0.9);
        mr = movavg(D.t(~bad), rat(~bad), P.movavg_s);
        gRm = [];
        if ~isempty(mr), gRm = plot(td(~bad), mr, 'k-','LineWidth',1.6); end
        gOne = plot([td(1) td(end)],[1 1],'k--','LineWidth',1.2);
        gBia = plot([td(1) td(end)],[biasFac biasFac],'-','Color',[0.85 0.35 0.15],'LineWidth',1.4);
        gDrp = [];
        if any(bad)
            gDrp = plot(td(bad), min(max(rat(bad),lim(1)),lim(2)), 'v', ...
                        'MarkerFaceColor',[0.8 0.2 0.2],'MarkerEdgeColor','none','MarkerSize',5);
        end
        set(gca,'YScale','log'); ylim(lim); ylabel('measured / model');
        t2 = sprintf('BIAS = x%.3f (log-mean), spread x%.3f (log-std), median %.3f', ...
                     biasFac, sprd, median(rat(~bad)));
        if any(bad)
            t2 = sprintf('%s  |  %d REFERENCE dropout(s) clipped, NOT deleted', t2, sum(bad));
        end
        title(t2,'FontWeight','normal','FontSize',9);

        % ---- 3: what the scale factor cannot explain --------------------------
        subplot(3,1,3); hold on; grid on; box on;
        res = me - SF.factor*mo;
        gRes = plot(dec(td,P.decimate), dec(res,P.decimate), '-','Color',[0.35 0.35 0.35],'LineWidth',0.9);
        mres = movavg(D.t, res, P.movavg_s);
        gResm = [];
        if ~isempty(mres), gResm = plot(td, mres, 'k-','LineWidth',1.6); end
        plot([td(1) td(end)],[0 0],'k:');
        ylabel('measured - k*model [kg/m^3]'); xlabel('time [min]');
        title(sprintf(['what the scale factor CANNOT explain: rms %.2e (was %.2e before ' ...
              'scaling, so k absorbed %.0f%%)'], SF.rms_after, SF.rms_before, ...
              100*max(SF.explained,0)),'FontWeight','normal','FontSize',9);

        G = {gMe, gMo, gK, gR, gBia, gRes}; L = {'measured','model','k x model', ...
             'measured/model','bias (log-mean)','residual after k'};
        if ~isempty(gRm),   G{end+1}=[gRm gResm]; L{end+1}=sprintf('%g s moving mean',P.movavg_s); end
        if ~isempty(gDrp),  G{end+1}=gDrp;  L{end+1}='dropouts'; end
        if ~isempty(gOne),  G{end+1}=gOne;  L{end+1}='perfect = 1'; end
        sgt(sprintf('%s - %s', ttl, nm));
        validation.toggle_panel(f, G, L);
        figs{end+1} = f; tags{end+1} = tg;
    end



    % ============================================ F8: INDEPENDENT TRACKS =====
    % TU Delft and TLE are NOT the OD reference and must not be read as one.
    %
    %   TU Delft : a geodetic track that ships WITH the density retrieval. Same
    %              satellite, same epoch, its own processing chain. Useful as a
    %              second opinion at the 10-100 m level -- and it is only a
    %              comparison at all if the satellite and epoch MATCH. GRACE-B
    %              silently receiving GRACE-A's file showed up here as 247 km.
    %   TLE      : SGP4 mean elements. Kilometre-class BY DESIGN. A TLE overlay
    %              that looks bad is a TLE being a TLE, not a propagator being
    %              wrong. It is here to catch gross errors (wrong satellite, wrong
    %              epoch, wrong frame) -- nothing finer.
    %
    % So: overlay for "are we on the same orbit at all", error panel for the size,
    % and the reference floor is deliberately NOT drawn -- these live far above it.
    if P.tracks && (isfield(R,'tud_track') || isfield(R,'tle_track'))
        f = figure('Color','w','Name','independent tracks','Position',[200 200 1000 640]);
        src = {}; if isfield(R,'tud_track'), src{end+1}='tud_track'; end
        if isfield(R,'tle_track'), src{end+1}='tle_track'; end
        np = numel(src);
        for i = 1:np
            S = R.(src{i});
            nm = subsref_tern(strcmp(src{i},'tud_track'), 'TU Delft track', 'TLE (SGP4)');
            subplot(np,2,2*i-1); hold on; grid on; box on;
            plot(S.t/60, S.r_ours/1000,  '-','Color',[0.20 0.45 0.80],'LineWidth',1.4);
            plot(S.t/60, S.r_track/1000, '--','Color',[0.85 0.35 0.15],'LineWidth',1.0);
            ylabel('|r| [km]'); legend({'ours',nm},'Location','best');
            title(sprintf('%s - overlaid', nm),'FontWeight','normal');
            if i==np, xlabel('time [min]'); end
            subplot(np,2,2*i); hold on; grid on; box on;
            plot(S.t/60, S.err, '-','Color',[0.35 0.35 0.35],'LineWidth',0.8);
            m = movavg(S.t, S.err, P.movavg_s);
            if ~isempty(m), plot(S.t/60, m, 'k-','LineWidth',1.6); end
            ylabel('|error| [m]');
            title(sprintf('%s - error   RMS %.1f m   %s', nm, rms_(S.err), ...
                  subsref_tern(strcmp(src{i},'tle_track'), '(km-class is NORMAL for a TLE)', '')), ...
                  'FontWeight','normal');
            if i==np, xlabel('time [min]'); end
        end
        sgt(sprintf('%s - independent tracks (NOT the OD reference)', ttl));
        figs{end+1} = f; tags{end+1} = 'tracks';
    end

    % ======================================================== F9: the numbers =
    if P.stats
        f = figure('Color','w','Name','statistics','Position',[180 180 900 620]); axis off;
        L = {sprintf('%s   -   OD VALIDATION', ttl), ''};
        if haveRDO
            L{end+1} = '[1] vs reducedDynamicOrbit  (TU Graz force model + empirical accels)';
            L{end+1} = sprintf('      radial %8.3f   along %8.3f   cross %8.3f   |3D| %8.3f  m RMS', ...
                        R.rdo.radial, R.rdo.along, R.rdo.cross, R.rdo.pos3D);
            L{end+1} = sprintf('      bias:  radial %+8.3f   along %+8.3f   cross %+8.3f  m', ...
                        mean(R.rdo.rad), mean(R.rdo.alo), mean(R.rdo.cro));
            if isfield(R.rdo,'dv'), L{end+1} = sprintf('      velocity %.5f m/s RMS', R.rdo.dv); end
        end
        if haveKIN
            L{end+1} = '';
            L{end+1} = '[2] vs kinematicOrbit  (GPS geometry only - INDEPENDENT of any force model)';
            L{end+1} = sprintf('      radial %8.3f   along %8.3f   cross %8.3f   |3D| %8.3f  m RMS', ...
                        R.kin.radial, R.kin.along, R.kin.cross, R.kin.pos3D);
        end
        if isfinite(floorM)
            L{end+1} = '';
            L{end+1} = sprintf('    REFERENCE FLOOR = %.3f m RMS   (kinematic - reducedDynamic)', floorM);
            L{end+1} = '    A residual below this is NOT resolvable. Do not tune there.';
        end
        if isfield(R,'acc')
            % gf_, not a bare R.acc.rms_diff. This panel assumed the field and died
            % on a fixture that did not have it -- the same "never assume a struct's
            % shape" rule show_provenance already follows. R.acc's contents depend on
            % which branch of od_metrics ran, and a stats panel must not be the thing
            % that decides whether a run succeeded.
            gf_ = @(f) subsref_default(R.acc, f, NaN);
            L{end+1} = '';
            L{end+1} = '[3] non-conservative force vs accelerometer   (Cd*A/m and density, DIRECTLY)';
            L{end+1} = sprintf('      measured %.4e | modelled %.4e | diff %.4e m/s^2 RMS', ...
                        gf_('rms_meas'), gf_('rms_mod'), gf_('rms_diff'));
            [ar_, ok_] = validation.acc_ratio(R);
            L{end+1} = sprintf('      ratio modelled/measured = %.4f  %s', ar_, ...
                subsref_tern(ok_,'(BIAS-REMOVED -- the physics number)', ...
                                 '(RAW: includes the instrument bias, ~drag/bias)'));
            if isfield(R.acc,'bias_meas')
                L{end+1} = sprintf('      accelerometer bias |b| = %.3e m/s^2 (~10x the drag at 345 km)', ...
                                   norm(R.acc.bias_meas));
            end
        end
        if isfield(R,'den')
            L{end+1} = '';
            L{end+1} = sprintf('[4] density vs ITSG (%s):  ratio %.4f | RMS log-error %.4f', ...
                        subsref_default(R.den,'model','?'), subsref_default(R.den,'ratio',NaN), ...
                        subsref_default(R.den,'rms_logerr',NaN));
        end
        if isfield(R,'tud')
            L{end+1} = sprintf('[5] density vs TU Delft (%s):  ratio %.4f | RMS log-error %.4f   <- independent', ...
                        subsref_default(R.tud,'model','?'), subsref_default(R.tud,'ratio',NaN), ...
                        subsref_default(R.tud,'rms_logerr',NaN));
        end
        if isfield(R,'closure') && ~isempty(R.closure) && isfinite(R.closure.factor)
            Cl = R.closure;
            L{end+1} = '';
            L{end+1} = '-- CLOSURE: do metrics [1] and [3] agree with EACH OTHER? --';
            L{end+1} = sprintf('   accel deficit %.3e m/s^2 -> implies %.2f m of along-track drift', ...
                               Cl.da_acc, Cl.dx_pred);
            L{end+1} = sprintf('   metric [1] actually measured %.2f m   (ratio %.2f)', Cl.dx_obs, Cl.factor);
            L{end+1} = sprintf('   %s', subsref_tern(Cl.ok, 'CONSISTENT', '*** INCONSISTENT ***'));
            % wrap the verdict so it does not run off the panel
            w = Cl.verdict; while numel(w) > 72
                k = find(w(1:72)==' ', 1, 'last'); if isempty(k), k = 72; end
                L{end+1} = sprintf('   %s', w(1:k-1)); w = w(k+1:end);
            end
            if ~isempty(w), L{end+1} = sprintf('   %s', w); end
        end
        if isfield(R,'densitySpread')
            L{end+1} = sprintf('    TRUTH vs TRUTH: ITSG and TU Delft differ by %.1f%% in median density.', ...
                        100*(exp(R.densitySpread)-1));
            L{end+1} = '    Neither is absolute - both are retrievals. That gap is the density floor.';
        end
        L{end+1} = '';
        L{end+1} = 'Metric (1) ranks agreement with TU Graz''s force model, not with physics.';
        L{end+1} = 'Metrics (3),(4),(5) are measurements. Prefer them when they disagree.';
        text(0.02, 0.98, L, 'Units','normalized','VerticalAlignment','top', ...
             'FontName','Courier New','FontSize',9,'Interpreter','none');
        figs{end+1} = f; tags{end+1} = 'stats';
    end

    if ~isempty(outdir)
        % mkdir FIRST. print() does not create the directory and errors if it is
        % missing -- so a run that produced every figure correctly would still throw
        % at the very last step, after the propagation and every download, for the
        % most trivial reason there is. validate_OD happens to make outdir earlier,
        % which is exactly why this went unnoticed: the bug only appears when
        % show_OD is called on its own, which is what anyone debugging a plot does.
        if ~exist(outdir,'dir'), mkdir(outdir); end
        % Name each file after the figure it IS, not its position in the list. This
        % used to index a fixed name array by loop counter, so turning any figure OFF
        % shifted every later filename: rendering only .tracks wrote OD_rtn.png.
        % Silently mislabelled output is worse than none -- you would have believed
        % the wrong picture.
        for i = 1:numel(figs)
            print(figs{i}, '-dpng', '-r150', fullfile(outdir, sprintf('OD_%s.png', tags{i})));
        end
        fprintf('  figures -> %s\n', outdir);
    end
end

% =============================================================== helpers ======
function y = dec(x, n)
%DEC  Decimate a series FOR DRAWING ONLY.
%   Applied at the plot call, never before a mean/std/RMS. If it were applied once
%   at the top of show_OD, every statistic in every title would silently be computed
%   on a subsample -- and a std computed on every 10th point is a different number
%   that looks exactly like the right one. The knob says "draw fewer points"; it must
%   not become "measure fewer points".
    if n <= 1 || isempty(x), y = x; return, end
    y = x(1:n:end, :);
end

function names = figureToggles()
%FIGURETOGGLES  THE list of figure switches. One source of truth.
%
%   Toggles and sampling KNOBS used to live in one struct with no way to tell them
%   apart: `hist_bins = 0` and `decimate = 1` look exactly like figure switches, and
%   a test trying to check "every toggle suppresses its figure" tried to switch off
%   the histogram BIN COUNT. That is not the test being clumsy -- it is the struct
%   conflating two different kinds of thing and daring anyone to guess which is
%   which. Naming the figures explicitly ends the guessing for every reader, human or
%   otherwise.
    % ---- ONE REFERENCE PER FIGURE ------------------------------------------
    % rtn / overlay_r / rtn_overlay are GONE. Between them the radial residual was
    % drawn three times and the along-track three times, and every one of them either
    % mixed the two references on one axes or silently used whichever existed. You
    % cannot tell, from a curve labelled "error", whether you are looking at a
    % disagreement with the reduced-dynamic orbit or with the kinematic one -- and
    % those are DIFFERENT MEASUREMENTS whose difference is the floor under both.
    %
    % Replaced by one figure per reference, each carrying its own overlay AND its own
    % residual, so the label on the figure IS the answer to "compared to what".
    names = { 'rtn_rdo', ...       % [1] RTN vs REDUCED-DYNAMIC: overlay + residual
              'rtn_kin', ...       % [2] RTN vs KINEMATIC: overlay + residual
              'rtn_v', ...         % [1] RTN velocity: overlay + residual (RDO only)
              'rtn_growth', ...    % [1][2] per-axis RTN error growth, both refs
              'overlay_v', 'vec_r', 'vec_v', ...
              'growth', 'hist', 'acc_raw', 'acc_forces', 'acc_axes', 'acc_scale', ...
              'density_itsg', 'density_tud', 'tracks', 'stats' };
end

function P = defaults(P)
    d = struct();
    % ---- FIGURES: one switch each, all on --------------------------------------
    fn_ = figureToggles();
    for i_ = 1:numel(fn_), d.(fn_{i_}) = 1; end

    % ---- SAMPLING AND TUNING KNOBS ---------------------------------------------
    % These were magic numbers buried in the plotting code. They CHANGE WHAT YOU SEE,
    % so they are decisions, and a decision nobody can find is a decision nobody can
    % question.
    %
    % movavg_s   THE big one. Too short smooths nothing; too long flattens a real
    %            secular trend into the mean and it DISAPPEARS. ~1/18 of a 90 min
    %            orbit by default. Sweep it: a feature that dies at 600 s and lives at
    %            300 s is per-rev structure, not a trend -- that IS the diagnosis.
    % hist_bins  0 = auto (N/12, clamped 12..40). Too few hides bimodality; too many
    %            turn a distribution into a comb and you read sample noise as shape.
    % ratio_lim  the dropout test AND the axis. Widen it to see the raw spikes.
    % decimate   plot every Nth sample. DRAWING ONLY -- see dec().
    d.movavg_s  = 300;
    d.hist_bins = 0;
    d.ratio_lim = [0.2 5];
    d.decimate  = 1;

    fn = fieldnames(d);
    for i = 1:numel(fn)
        if ~isfield(P, fn{i}) || isempty(P.(fn{i})), P.(fn{i}) = d.(fn{i}); end
    end
end

function m = movavg(t, y, win_s)
%MOVAVG  Centred moving mean over a TIME window, not a sample count.
%   Epochs are irregular (kinematic especially), so a fixed sample count would be a
%   different physical window at different places in the arc. Seconds keeps it honest.
    m = [];
    if isempty(win_s) || win_s <= 0 || numel(y) < 3, return, end
    t = t(:); y = y(:); m = nan(size(y)); h = win_s/2;
    for k = 1:numel(y)
        s = t >= t(k)-h & t <= t(k)+h;
        m(k) = mean(y(s));
    end
end

function drawFloor(t, floorM)
%DRAWFLOOR  Shade +/- the reference's own uncertainty. Inside this band nothing is
%   resolvable, and a residual drawn without it invites you to chase noise.
    if ~isfinite(floorM) || floorM <= 0, return, end
    x = [t(1) t(end) t(end) t(1)];
    y = [-floorM -floorM floorM floorM];
    fill(x, y, [0.90 0.90 0.90], 'EdgeColor','none');
end

function y = rms_(x), y = sqrt(mean(x(:).^2)); end

function sgt(s)
%SGT  Super-title, wrapped, with room ACTUALLY made for it.
%
%   Round 40 shrank each axes so its BOX cleared the super-title, and the titles
%   still collided -- because `title()` does not draw inside the box. It draws ABOVE
%   it. Clearing the box and leaving the title where it was is fixing the wrong
%   rectangle, and the pixel check I ran happened to be on the one figure with no
%   subplot title, so it passed. A test that cannot fail is not a test.
%
%   This version reserves a band at the top and RE-LAYS every axes underneath it,
%   scaling the whole stack rather than nudging individual boxes. Each axes keeps its
%   share of the remaining height, so a 3-panel figure stays evenly spaced instead of
%   having its top panel squashed.
    s  = wrapText(s, 100);
    nl = 1 + sum(s == char(10));

    % Reserve: the super-title itself, plus a gap, plus room for the top subplot's
    % own title. That last term is what round 40 missed.
    band = 0.030*nl + 0.030;
    top  = 1 - band;

    % Only axes whose PARENT is the figure. findobj walks the whole tree, so once a
    % uipanel exists it returns that panel's children too -- and repositioning an
    % axes that lives inside a panel, using FIGURE-normalized coordinates, is how you
    % get "Could not find node in peer tree during reparentChildren": we hand MATLAB
    % a layout referring to a parent the object does not have.
    %
    % sgt is also now called BEFORE toggle_panel everywhere, so this is belt and
    % braces -- but a layout helper that silently corrupts the scene when called in
    % the wrong order is a trap, and the next figure added will fall into it.
    ax = findobj(gcf, 'Type','axes', 'Parent', gcf);
    if ~isempty(ax)
        keep = true(size(ax));
        for q_ = 1:numel(ax), keep(q_) = ~strcmp(get(ax(q_),'Tag'), 'legend'); end
        ax = ax(keep);
    end
    if ~isempty(ax)
        % get(ax,'Position') returns a PLAIN VECTOR for a single handle and a CELL
        % for several. cell2mat on the vector throws. Loop instead of special-casing:
        % the one-axes figure is exactly the case round 40 "verified", so it is worth
        % not writing code that treats it differently.
        hi = 0;
        for q = 1:numel(ax)
            p = get(ax(q), 'Position');
            hi = max(hi, p(2) + p(4));
        end
        if hi > top && hi > 0
            k = top / hi;                       % scale the WHOLE stack
            for q = 1:numel(ax)
                p = get(ax(q),'Position');
                set(ax(q), 'Position', [p(1), p(2)*k, p(3), p(4)*k]);
            end
        end
    end
    try
        sgtitle(s, 'FontWeight','bold', 'Interpreter','none', 'FontSize',10);
    catch
        try
            annotation('textbox',[0.02 top 0.96 band],'String',s,'EdgeColor','none', ...
                       'HorizontalAlignment','center','FontWeight','bold', ...
                       'Interpreter','none','FitBoxToText','off','FontSize',9);
        catch
            % LEGITIMATE swallow: sgtitle is R2018b+, annotation is unreliable on
            % Octave's gnuplot backend. A missing super-title must not kill a figure
            % that is otherwise correct.
        end
    end
end

function out = wrapText(s, w)
%WRAPTEXT  Break a string onto lines of at most w characters, at word boundaries.
%   Titles in this suite carry an interpretation, not just a label -- "k = 1.887,
%   r2 = 1.00, the model has the right shape and the wrong size" is the finding. A
%   caption that runs off the axes or overprints its neighbour delivers none of it,
%   so wrap rather than truncate: truncating loses the end of the sentence, which is
%   usually where the conclusion lives.
    out = '';
    if isempty(s), return, end
    words = strsplit(s, ' ');
    line = '';
    for i = 1:numel(words)
        cand = strtrim([line ' ' words{i}]);
        if numel(cand) > w && ~isempty(line)
            out = [out line char(10)];
            line = words{i};
        else
            line = cand;
        end
    end
    out = [out line];
end

function d = rtnToEci(rad, alo, cro, r, v)
%RTNTOECI  Rebuild an ECI residual vector from its RTN components. [Nx1 x3] -> [Nx3]
%   The exact inverse of validation.rtn_project, using the SAME triad construction,
%   written here next to its caller rather than as a second package function so the
%   pair cannot drift -- one edited and the other not is the "two definitions of
%   along-track" bug this audit has removed several times.
%
%   (This function was deleted by accident in round 41: the sgt() rewrite cut from
%   `function sgt` to `function wrapText` and rtnToEci was sitting between them.
%   Caught immediately by rendering -- which is why the render fixture exists.)
    n = size(r,1);
    d = zeros(n,3);
    for k = 1:n
        rk = r(k,:).'; vk = v(k,:).';
        R_ = rk/norm(rk);
        C_ = cross(rk,vk); C_ = C_/norm(C_);
        A_ = cross(C_, R_);
        d(k,:) = (rad(k)*R_ + alo(k)*A_ + cro(k)*C_).';
    end
end

function v = refVel(D)
%REFVEL  The velocity to build the RTN triad from, for whichever reference this is.
%   The KINEMATIC product has no velocity -- it is MJD,x,y,z. But an RTN triad needs
%   one, so use OUR velocity sampled onto the kinematic epochs (od_metrics stores it
%   as v_our). That is not a fudge: the triad only sets the DIRECTIONS to resolve the
%   difference along, and both series are being resolved in the SAME triad, which is
%   what makes the comparison meaningful. Using two different triads would silently
%   compare two different "along-tracks" -- the exact bug this audit has removed
%   twice.
    if isfield(D,'v_our') && ~isempty(D.v_our)
        v = D.v_our;
    elseif isfield(D,'v_ref') && ~isempty(D.v_ref)
        v = D.v_ref;
    else
        error('show_OD:noVel','no velocity to build an RTN triad from');
    end
end

function y = runningRMS(x)
%RUNNINGRMS  RMS of everything up to each epoch. Not a moving window.
%   The question is "is the error accumulating", and a moving window deliberately
%   forgets the past, which is the thing being asked about. A running RMS that
%   climbs means the error is still growing; one that flattens means it has stopped,
%   whatever the instantaneous curve is doing.
    x = x(:);
    y = sqrt(cumsum(x.^2) ./ (1:numel(x)).');
end
