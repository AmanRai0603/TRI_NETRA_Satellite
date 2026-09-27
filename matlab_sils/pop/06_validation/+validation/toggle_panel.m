function panel = toggle_panel(fig, groups, labels)
%VALIDATION.TOGGLE_PANEL  Real checkboxes that show/hide curves across a whole figure.
%   panel = validation.toggle_panel(fig, groups, labels)
%     groups : cell array; groups{k} is a vector of graphics handles to toggle
%     labels : cell array of names, one per group
%
%   ---------------------------------------------------------------------------
%   WHY NOT ItemHitFcn
%   ---------------------------------------------------------------------------
%   The first attempt used a clickable legend (`ItemHitFcn`). It did nothing, and
%   worse, it did nothing SILENTLY: the property assignment succeeds, the callback
%   just never fires if the legend's line mapping is not what you assumed, and there
%   is no error to notice. A control that quietly does nothing is the worst kind --
%   you click, the plot does not change, and you cannot tell whether the toggle is
%   broken or the curves genuinely overlap.
%
%   A uicontrol checkbox cannot fail that way. It is a real widget with a real
%   callback and its state is visible on screen. It also works on Octave, where
%   ItemHitFcn does not exist at all.
%
%   ---------------------------------------------------------------------------
%   WHAT IT TOGGLES
%   ---------------------------------------------------------------------------
%   A GROUP, not a line. In these figures the same series is drawn in three panels
%   (x, y, z), and toggling one panel's copy while the other two stay is not a view
%   anybody wants -- you would be comparing an axis you can see against two you
%   cannot. One checkbox hides the series EVERYWHERE it appears.
%
%   Returns the uipanel so a caller can reposition it. The panel steals a strip on
%   the right; the axes are shrunk to fit rather than drawn under it, because a
%   control sitting on top of the data is the overlap problem again in a new hat.
    panel = [];
    if isempty(groups), return, end

    % ---- CAN THIS BACKEND EVEN DRAW A uipanel? -----------------------------
    % Octave's gnuplot toolkit CREATES the panel happily and then dies at print()
    % with "unknown object class, uipanel" -- so the figure builds, the toggles
    % appear to exist, and the failure lands at the very last step, taking the PNG
    % with it. Check first: a control the backend cannot render is worse than no
    % control, because it destroys the figure it was meant to improve.
    %
    % This is a real limitation, not a bug to route around. MATLAB (any version
    % with uicontrol) and Octave's qt toolkit are fine; gnuplot is not.
    persistent warned
    if exist('OCTAVE_VERSION','builtin') && strcmp(graphics_toolkit(), 'gnuplot')
        if isempty(warned)
            warned = true;
            fprintf(['  [toggle panels need a backend that can draw uicontrols.', ...
                     ' This is Octave/gnuplot, which cannot -- the figures are', ...
                     ' drawn WITHOUT toggles. On MATLAB, or Octave with', ...
                     ' graphics_toolkit(''qt''), every series gets a checkbox.', ...
                     ' Use the PLOTS struct to switch whole figures off', ...
                     ' meanwhile.]', char(10)]);
        end
        return
    end
    n = numel(groups);

    % Let the scene settle before touching it. Repositioning axes while MATLAB is
    % mid-render produces "Could not find node in peer tree during reparentChildren"
    % -- we edit the layout out from under the renderer and it loses the object it
    % was drawing. One drawnow costs nothing here and removes the whole class.
    drawnow;

    % shrink every axes to make room -- do NOT draw the panel over them.
    % 'Parent',fig: only axes owned by the FIGURE. findobj walks the entire tree, so
    % on a second call it would also return the axes inside the panel we made last
    % time and reposition them in the wrong coordinate system.
    W  = 0.13;                                  % strip width, normalized
    ax = findobj(fig, 'Type','axes', 'Parent', fig);
    for k = 1:numel(ax)
        if strcmp(get(ax(k),'Tag'),'legend'), continue, end
        p = get(ax(k), 'Position');
        set(ax(k), 'Position', [p(1), p(2), p(3)*(1-W-0.02), p(4)]);
    end
    drawnow;

    panel = uipanel('Parent',fig, 'Units','normalized', ...
                    'Position',[1-W-0.005, 0.02, W, 0.96], ...
                    'Title','show / hide', 'FontSize',8, 'BackgroundColor','w');

    h = min(0.06, 0.9/max(n,1));
    for k = 1:n
        y = 0.95 - k*h;
        uicontrol('Parent',panel, 'Style','checkbox', 'Units','normalized', ...
                  'Position',[0.06, y, 0.9, h*0.9], ...
                  'String',labels{k}, 'Value',1, 'FontSize',7, ...
                  'BackgroundColor','w', 'HorizontalAlignment','left', ...
                  'Callback',{@onToggle, groups{k}});
    end

    % ALL / NONE -- with 15 series, clicking 15 boxes to isolate one is not a
    % feature, it is a chore that stops people using the toggles at all.
    uicontrol('Parent',panel, 'Style','pushbutton', 'Units','normalized', ...
              'Position',[0.06, 0.02, 0.42, 0.05], 'String','all', 'FontSize',7, ...
              'Callback',{@onAll, groups, panel, 1});
    uicontrol('Parent',panel, 'Style','pushbutton', 'Units','normalized', ...
              'Position',[0.52, 0.02, 0.42, 0.05], 'String','none', 'FontSize',7, ...
              'Callback',{@onAll, groups, panel, 0});
end

function onToggle(src, ~, hs)
    setVis(hs, get(src,'Value'));
    rescaleAll(hs);
end

function onAll(~, ~, groups, panel, val)
    for k = 1:numel(groups), setVis(groups{k}, val); end
    cb = findobj(panel, 'Style','checkbox');
    set(cb, 'Value', val);
    if ~isempty(groups), rescaleAll(groups{1}); end
end

function setVis(hs, on)
    v = 'off'; if on, v = 'on'; end
    for i = 1:numel(hs)
        if ishandle(hs(i)), set(hs(i), 'Visible', v); end
    end
end

function rescaleAll(hs)
%RESCALEALL  Rescale every axes the toggled handles live in, to the VISIBLE curves.
%   Without this, hiding the big series leaves the axis stretched to fit a curve
%   that is no longer drawn -- so the small one you wanted to see stays a flat line
%   at the bottom and the click looks like it did nothing. That is the whole reason
%   the toggle exists, so skipping the rescale would waste the feature.
    axs = [];
    for i = 1:numel(hs)
        if ishandle(hs(i)), axs(end+1) = get(hs(i),'Parent'); end %#ok<AGROW>
    end
    for a = unique(axs)
        try
            ch = get(a,'Children'); lo = []; hi = [];
            for k = 1:numel(ch)
                if ~strcmp(get(ch(k),'Visible'),'on'), continue, end
                if ~isprop(ch(k),'YData'), continue, end
                y = get(ch(k),'YData'); y = y(isfinite(y));
                if strcmp(get(a,'YScale'),'log'), y = y(y > 0); end
                if isempty(y), continue, end
                lo = min([lo, min(y)]); hi = max([hi, max(y)]);
            end
            if ~isempty(lo) && hi > lo
                pad = 0.05*(hi-lo); ylim(a, [lo-pad, hi+pad]);
            end
        catch
            % rescaling is a nicety; a failure must not eat the toggle itself,
            % which has already happened and is the thing that was asked for.
        end
    end
end
