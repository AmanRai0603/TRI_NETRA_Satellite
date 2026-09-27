function f = show_provenance(P, R, ttl, outdir)
%SHOW_PROVENANCE  ONE figure: where every input came from, and what it bought.
%   f = show_provenance(P, R, ttl, outdir)
%     P = validation.provenance(cfg, W, REF, SAT)
%     R = validation.od_metrics(...) output (may be [])
%
%   ---------------------------------------------------------------------------
%   THE POINT
%   ---------------------------------------------------------------------------
%   Four kinds of input go into one number and they are NOT interchangeable:
%
%     MEASURED  a real instrument, this satellite, this epoch
%     FETCHED   somebody else's measurement from an open source
%     ASSUMED   a number we chose because nobody published one
%     MISSING   needed, absent, blocking a model
%
%   Once they are floats in a struct they look identical. A residual explained by
%   an ASSUMED Cr is not a finding. A model refused for a MISSING input is not a
%   failure. CHAMP flew at Cd=2.2 while the report printed 3.0 precisely because
%   nothing showed which class a number was in.
%
%   Left column: every input, coloured by class, with its source spelled out.
%   Right column: the metrics -- and each metric's own floor, because a residual
%   below the reference spread is not resolvable and must not be tuned against.
%
%   ONE figure on purpose. The classes only mean something side by side.
    if nargin<3||isempty(ttl), ttl='provenance'; end
    if nargin<4, outdir=''; end
    if nargin<2, R=[]; end

    % the shared vocabulary -- see validation.prov_colors for why this is a
    % function and not five literals typed into three figures
    PC = validation.prov_colors();
    CL = struct('measured',PC.measured, 'measured_ish',PC.derived, ...
                'fetched',PC.fetched, 'assumed',PC.assumed, 'missing',PC.missing);
    f = figure('Color','w','Name','provenance','Position',[60 40 1340 900]);

    % ---------------- LEFT: the inputs, by class ------------------------------
    % LAYOUT. The legend used to be drawn in the SAME axes as the item list, in data
    % coordinates, starting at y=0.2 -- which lands ON TOP of the last few items as
    % soon as the list is long. It collided on the very first real run. Give the
    % legend its own axes and size the list axes to the item count instead of
    % assuming it fits.
    ax = axes('Position',[0.04 0.16 0.44 0.76]); cla(ax); hold(ax,'on');
    items = P.items; n = numel(items);
    ord = {'measured','measured-ish','fetched','assumed','missing'};
    idx = [];
    for k=1:numel(ord), idx = [idx find(strcmp({items.class}, ord{k}))]; end
    items = items(idx);

    % Font and truncation scale with the ITEM COUNT. A fixed 8.5 pt overprints the
    % moment the list passes ~22 rows, and the list length is data-dependent (it grows
    % with every model that has parameters), so it WILL pass it.
    fs   = max(5.5, min(8.5, 190/max(n,1)));
    fs_s = max(5,   fs-1.5);
    ncut = round(max(30, min(58, 1200/max(n,1))));
    y = n;
    for i=1:n
        c = colOf_(items(i).class, CL);
        rectangle('Position',[0 y-0.38 0.28 0.76],'FaceColor',c,'EdgeColor','none');
        text(0.34, y, items(i).name, 'FontSize',fs,'Interpreter','none','VerticalAlignment','middle');
        text(4.3, y, shorten_(items(i).source,ncut), 'FontSize',fs_s,'Color',[0.35 0.35 0.35], ...
             'Interpreter','none','VerticalAlignment','middle');
        y = y - 1;
    end
    xlim([0 11]); ylim([0.3 n+0.7]); axis off;
    title(sprintf('INPUTS by origin:  measured %d | fetched %d | ASSUMED %d | MISSING %d', ...
        P.n.measured, P.n.fetched, P.n.assumed, P.n.missing), ...
        'FontWeight','normal','FontSize',10);
    % legend in its OWN axes, below the list -- it cannot collide with anything
    axl = axes('Position',[0.04 0.02 0.44 0.11]); hold(axl,'on');
    xlim(axl,[0 5]); ylim(axl,[0 1]); axis(axl,'off');
    lg = {PC.label.measured, CL.measured; ...
          PC.label.derived,  CL.measured_ish; ...
          PC.label.fetched,  CL.fetched; ...
          PC.label.assumed,  CL.assumed; ...
          PC.label.missing,  CL.missing};
    for i=1:size(lg,1)
        yy = 1 - 0.19*i;
        rectangle(axl,'Position',[0.05 yy-0.06 0.10 0.12],'FaceColor',lg{i,2},'EdgeColor','none');
        text(0.22, yy, lg{i,1}, 'FontSize',7,'VerticalAlignment','middle','Parent',axl);
    end

    % ---------------- RIGHT: what the inputs bought ---------------------------
    if ~isempty(R)
        % ---- NEVER ASSUME R'S SHAPE ------------------------------------------
        % R.rdo.rms3d does not exist -- validation.rtn_stats returns .pos3D. And
        % R.acc has no .ratio: od_metrics builds it WITHOUT one and computes the
        % ratio inline. Both were names I invented, both crash at the line, and
        % both are the DO_PLOTS bug wearing a dot.
        %
        % A checker cannot save this reliably (a struct's shape depends on which
        % branch ran, and R's contents legitimately vary with which products the
        % satellite has). So the defence belongs HERE, on the reader side: check
        % before every access, and degrade to a missing panel rather than a crash.
        % A figure is a diagnostic -- it must never be the thing that kills the run
        % that produced the diagnosis, especially after 20 s of propagation and
        % every download.
        gf3 = @(S,f) subsref_default(subsref_default(S,f,struct()),'pos3D',NaN);
        axes('Position',[0.57 0.78 0.39 0.15]); hold on; grid on; box on;
        nm = {}; val = []; flr = [];
        if isfield(R,'rdo'), nm{end+1}='[1] vs RDO'; val(end+1)=gf3(R,'rdo'); flr(end+1)=NaN; end
        if isfield(R,'kin'), nm{end+1}='[2] vs KIN'; val(end+1)=gf3(R,'kin');
            flr(end+1)=subsref_default(R,'refSpread',NaN); end
        keep = isfinite(val); nm = nm(keep); val = val(keep); flr = flr(keep);
        if ~isempty(val)
            b = bar(val,'FaceColor',[0.20 0.45 0.80],'EdgeColor','none'); hold on;
            for i=1:numel(flr)
                if isfinite(flr(i))
                    plot([i-0.4 i+0.4],[flr(i) flr(i)],'r-','LineWidth',2);
                end
            end
            set(gca,'XTick',1:numel(nm),'XTickLabel',nm,'YScale','log');
            ylabel('RMS 3D [m]');
            title('position metrics vs the REFERENCE FLOOR (red)','FontWeight','normal','FontSize',9);
        end

        % (b) ratios: model/measured. 1.0 is the only interesting value.
        axes('Position',[0.57 0.55 0.39 0.15]); hold on; grid on; box on;
        rn = {}; rv = [];
        % [3] has NO .ratio field: od_metrics prints rms_mod/rms_meas but does not
        % store it. Recompute from what IS there rather than reading a field I wish
        % existed.
        if isfield(R,'acc')
            % validation.acc_ratio, NOT an inline divide. I wrote the inline version
            % here one round ago and it used the RAW ratio -- so this figure would
            % have shown 0.104 (drag/bias) next to a console printing the corrected
            % number. Five copies of one formula, and only one of them learned about
            % the bias fix. That is the whole argument for the function.
            [ar_, ok_] = validation.acc_ratio(R);
            if isfinite(ar_)
                rn{end+1} = subsref_tern(ok_,'[3] non-grav (bias-removed)','[3] non-grav (RAW)');
                rv(end+1) = ar_;
            end
        end
        if isfield(R,'den') && isfield(R.den,'ratio'), rn{end+1}='[4] density (ITSG)'; rv(end+1)=R.den.ratio; end
        if isfield(R,'tud') && isfield(R.tud,'ratio'), rn{end+1}='[5] density (TUD)';  rv(end+1)=R.tud.ratio; end
        if ~isempty(rv)
            barh(rv,'FaceColor',[0.13 0.55 0.24],'EdgeColor','none');
            plot([1 1],[0.4 numel(rv)+0.6],'k--','LineWidth',1.5);
            set(gca,'YTick',1:numel(rn),'YTickLabel',rn); xlabel('model / measured');
            title('ratios -- 1.0 is the only value that means anything','FontWeight','normal','FontSize',9);
        end

        % (c) what an ASSUMED input could be worth
        axes('Position',[0.55 0.28 0.42 0.20]); axis off; xlim([0 1]); ylim([0 1]);
        as = items(strcmp({items.class},'assumed'));
        % ONE text() call per line: Octave's gnuplot backend silently mangles a
        % cellstr passed to text(), and a figure that renders on MATLAB and not on
        % Octave is a figure nobody checks.
        L = {'ASSUMED inputs that can move the answer:'};
        for i=1:min(6,numel(as)), L{end+1} = sprintf('   - %s', as(i).name); end
        L{end+1} = '';
        L{end+1} = 'A residual explained by any of these is NOT a finding.';
        L{end+1} = 'Sweep it (compare_OD) before believing it.';
        for i=1:numel(L)
            text(0.02, 0.96-0.105*(i-1), L{i}, 'FontSize',8, 'Interpreter','none', ...
                 'VerticalAlignment','top', 'FontWeight', subsref_tern(i==1,'bold','normal'), ...
                 'Color', subsref_tern(i==1, [0.90 0.60 0.10]*0.7, [0 0 0]));
        end

        % (d) what is MISSING and what it costs
        axes('Position',[0.55 0.03 0.42 0.20]); axis off; xlim([0 1]); ylim([0 1]);
        ms = items(strcmp({items.class},'missing'));
        L2 = {'MISSING -- what these block:'};
        if isempty(ms)
            L2{end+1} = '   (nothing missing for this satellite/epoch)';
        else
            for i=1:min(5,numel(ms)), L2{end+1} = sprintf('   - %s', ms(i).name); end
        end
        L2{end+1} = '';
        L2{end+1} = 'A model refused for a missing input is a DATA limit,';
        L2{end+1} = 'not a code limit. validation.capability(SAT) lists them.';
        for i=1:numel(L2)
            text(0.02, 0.96-0.115*(i-1), L2{i}, 'FontSize',8, 'Interpreter','none', ...
                 'VerticalAlignment','top', 'FontWeight', subsref_tern(i==1,'bold','normal'), ...
                 'Color', subsref_tern(i==1, [0.80 0.20 0.20], [0 0 0]));
        end
    end

    try
        sgtitle(sprintf('%s -- provenance', ttl));
    catch
        % LEGITIMATE: sgtitle is R2018b+ and unreliable on Octave's gnuplot backend.
        % Cosmetic only.
    end
    if ~isempty(outdir)
        if ~exist(outdir,'dir'), mkdir(outdir); end
        print(f,'-dpng','-r150',fullfile(outdir,'OD_provenance.png'));
        fprintf('  provenance -> %s\n', fullfile(outdir,'OD_provenance.png'));
    end
end

function c = colOf_(cls, CL)
    switch cls
        case 'measured',     c = CL.measured;
        case 'measured-ish', c = CL.measured_ish;
        case 'fetched',      c = CL.fetched;
        case 'assumed',      c = CL.assumed;
        otherwise,           c = CL.missing;
    end
end
function s = shorten_(s, n)
    if ~ischar(s), s = ''; return, end
    if numel(s) > n, s = [s(1:n-3) '...']; end
end
