function f = show_model_io(sol, W, ttl, outdir, opts)
%SHOW_MODEL_IO  ONE figure: what goes INTO each model, what it DERIVES, what comes OUT.
%   f = show_model_io(sol, W, ttl, outdir[, opts])
%
%   ---------------------------------------------------------------------------
%   WHY ONE FIGURE
%   ---------------------------------------------------------------------------
%   The chain is: drivers -> atmosphere -> (rho, composition, T) + attitude ->
%   (Cd(t), A(t)) -> acceleration. Every one of those handoffs has been wrong at
%   some point in this codebase, and every one was invisible in the final number:
%   density in cm^-3 (panel drag 1e6 low), R_bi frozen at eye(3) (drag exactly
%   zero), Cd read from the wrong struct (CHAMP at 2.2 not 3.0). A plot of the
%   final acceleration alone would have shown none of them.
%
%   So this shows the INTERMEDIATE quantities, side by side, on one time axis. If
%   rho looks right and Cd looks right and A(t) is flat when the satellite is
%   yawing, you have found something -- and you have found it without reading code.
%
%   Subplots, not a figure each: the point is to see them TOGETHER.
%
%   opts.every_s  sample spacing for the trace (default 30 s -- these quantities
%                 need op.accel per sample, which is not free)
    if nargin<5 || isempty(opts), opts = struct(); end
    every_s = getf(opts,'every_s',30);
    if nargin<3 || isempty(ttl), ttl = 'model I/O'; end
    if nargin<4, outdir = ''; end

    t = (sol.t(1) : every_s : sol.t(end)).';
    n = numel(t);
    [rq, vq] = validation.interp_state(sol.t, sol.r, sol.v, t);

    % ---- walk the chain, sample by sample, recording every intermediate --------
    Z = struct('rho',nan(n,1), 'T',nan(n,1), 'nO',nan(n,1), 'Cd',nan(n,1), ...
               'A',nan(n,1), 'yaw',nan(n,1), 'nu',nan(n,1), 'alt',nan(n,1), ...
               'drag',nan(n,1), 'srp',nan(n,1), 'erp',nan(n,1), 'grav',nan(n,1));
    d   = subsref_default(W.cfg.forces,'drag',struct());
    mdl = lower(getf(d,'atmos','exponential'));
    for k = 1:n
        [~, parts, ctx, info] = op.accel(t(k), rq(k,:).', vq(k,:).', W);
        Z.grav(k) = norm(parts.gravity);
        if isfield(parts,'drag'), Z.drag(k) = norm(parts.drag); end
        if isfield(parts,'srp'),  Z.srp(k)  = norm(parts.srp);  end
        if isfield(parts,'erp'),  Z.erp(k)  = norm(parts.erp);  end
        if isfield(info,'drag') && ~isempty(info.drag)
            D = info.drag;
            if ~isempty(D.atm)
                Z.rho(k) = D.atm.rho;
                if isfield(D.atm,'T'), Z.T(k) = D.atm.T; end
                if isfield(D.atm,'n') && isfield(D.atm.n,'O'), Z.nO(k) = D.atm.n.O; end
            end
            if ~isempty(D.geo), Z.alt(k) = D.geo.alt_km; end
            if ~isempty(D.out), Z.Cd(k) = D.out.Cd; end   % [] for cannonball: it has no Cd(t)
        end
        R = getf(ctx.sc,'R_bi',eye(3));
        Z.yaw(k) = atan2d(R(2,1), R(1,1));
        if ~isempty(getf(ctx.sc,'facets',[]))
            Z.A(k) = validation.projected_area(ctx.sc.facets, R, ctx.v_rel_eci);
        end
        if isfield(ctx,'E') && ~isempty(ctx.E) && isfield(ctx.E,'sun_eci')
            Z.nu(k) = srp.eclipse(rq(k,:).', ctx.E.sun_eci);
        end
    end
    tm = t/60;

    f = figure('Color','w','Name','model I/O','Position',[80 80 1280 860]);
    % ONE colour vocabulary, shared with show_provenance and show_OD. A colour that
    % means ASSUMED here and DERIVED there is worse than no colour -- you would read
    % it confidently and be wrong. See validation.prov_colors.
    PC = validation.prov_colors();
    C = struct('in', PC.fetched, 'der', PC.derived, 'out', [0.15 0.55 0.25], ...
               'asm', PC.assumed, 'meas', PC.measured);

    % ---------------- row 1: INPUTS (what the models are fed) -----------------
    subplot(3,3,1); semilogy(tm, Z.rho, '-','Color',C.in,'LineWidth',1.3); grid on; box on;
    ylabel('\rho [kg/m^3]');
    title(sprintf('\\rho -- MODEL %s, on FETCHED indices', mdl),'FontWeight','normal');

    subplot(3,3,2); plot(tm, Z.T, '-','Color',C.in,'LineWidth',1.3); grid on; box on;
    ylabel('T [K]'); title('IN: temperature','FontWeight','normal');

    subplot(3,3,3);
    if any(isfinite(Z.nO))
        semilogy(tm, Z.nO, '-','Color',C.in,'LineWidth',1.3);
        ylabel('n_O [m^{-3}]'); title('IN: atomic O (drives SESAM a_T)','FontWeight','normal');
    else
        text(0.5,0.5,{'no composition','from this model'},'HorizontalAlignment','center');
        axis off; title('IN: atomic O','FontWeight','normal');
    end
    grid on; box on;

    % ---------------- row 2: DERIVED (what the toggles produce) ---------------
    % attitude: MEASURED if quaternions were supplied, ASSUMED if we modelled ram
    attIsMeas = isstruct(getf(W.sc,'attitude',[]));
    subplot(3,3,4);
    plot(tm, Z.yaw, '-','Color',subsref_tern(attIsMeas,C.meas,C.asm),'LineWidth',1.3);
    grid on; box on; ylabel('yaw [deg]');
    title(sprintf('attitude (R_{bi}) -- %s', subsref_tern(attIsMeas, ...
          'MEASURED','ASSUMED: modelled ram')),'FontWeight','normal');

    subplot(3,3,5);
    if any(isfinite(Z.Cd))
        plot(tm, Z.Cd, '-','Color',C.der,'LineWidth',1.5); ylabel('C_d(t) [-]');
        title(sprintf('C_d(t) -- DERIVED by %s from GSI', getf(d,'model','cannonball')), ...
              'FontWeight','normal');
    else
        Cd0 = getf(W.sc,'Cd',NaN);
        plot(tm, Cd0*ones(n,1), '--','Color',C.asm,'LineWidth',2);
        ylabel('C_d [-]'); ylim([Cd0-1 Cd0+1]);
        title('C_d -- ASSUMED (cannonball: a constant you typed)','FontWeight','normal');
    end
    grid on; box on;

    subplot(3,3,6);
    if any(isfinite(Z.A))
        plot(tm, Z.A, '-','Color',C.der,'LineWidth',1.5); ylabel('A(t) [m^2]');
        title('A(t) -- DERIVED (geometry x attitude)','FontWeight','normal');
    else
        plot(tm, getf(W.sc,'Aref',NaN)*ones(n,1), '--','Color',C.fetched,'LineWidth',2);
        ylabel('A [m^2]');
        title('A_{ref} -- FETCHED (catalog), held CONSTANT','FontWeight','normal');
    end
    grid on; box on;

    % ---------------- row 3: OUTPUTS ------------------------------------------
    subplot(3,3,7); semilogy(tm, Z.drag,'-','Color',C.out,'LineWidth',1.4); grid on; box on;
    xlabel('time [min]'); ylabel('|a| [m/s^2]'); title('OUT: drag','FontWeight','normal');

    subplot(3,3,8); hold on; grid on; box on;
    semilogy(tm, Z.srp,'-','Color',C.out,'LineWidth',1.4);
    if any(isfinite(Z.nu)) && any(Z.nu<1)
        yl=ylim; ecl=Z.nu<1;
        area(tm, ecl*yl(2)+~ecl*yl(1), 'BaseValue',yl(1), 'FaceColor',[0.85 0.85 0.9], ...
             'EdgeColor','none','FaceAlpha',0.5);
        set(gca,'Children',flipud(get(gca,'Children'))); ylim(yl);
    end
    set(gca,'YScale','log'); xlabel('time [min]'); ylabel('|a| [m/s^2]');
    title('OUT: SRP (shaded = eclipse)','FontWeight','normal');

    subplot(3,3,9); hold on; grid on; box on;
    semilogy(tm, Z.erp,'-','Color',C.out,'LineWidth',1.4);
    xlabel('time [min]'); ylabel('|a| [m/s^2]');
    title(sprintf('OUT: ERP (%s)', getf(subsref_default(W.cfg.forces,'erp',struct()),'model','knocke')), ...
          'FontWeight','normal');

    % the colour legend belongs ON the figure: a reader should not have to know the
    % convention to read the picture
    try
        sgtitle(sprintf(['%s\n' ...
            'blue = FETCHED (open source)   pale green = DERIVED (physics)   ' ...
            'orange = ASSUMED (a knob)   green = MEASURED'], ttl), 'FontSize',9);
    catch
        % LEGITIMATE swallow: see above -- a missing super-title is cosmetic.
    end

    if ~isempty(outdir)
        if ~exist(outdir,'dir'), mkdir(outdir); end
        print(f, '-dpng','-r150', fullfile(outdir,'OD_model_io.png'));
        fprintf('  model I/O -> %s\n', fullfile(outdir,'OD_model_io.png'));
    end
end

function v = getf(s,fn,dv)
    if isstruct(s) && isfield(s,fn) && ~isempty(s.(fn)), v = s.(fn); else, v = dv; end
end
