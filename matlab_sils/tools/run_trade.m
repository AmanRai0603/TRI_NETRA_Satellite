function T = run_trade(ids)
%RUN_TRADE  Run trade studies and rank their candidates.
%   run_trade                              % every trade in data/trades
%   run_trade({'trade_mtq_pointing_ais'})  % a subset
%   Results: store/trades/<id>/trade.json, trade.png; docs/SELECTION.md is
%   written from them by tools/report.py.
%   Copyright (c) 2026 Agastya. All rights reserved.
    R = asils.util.root();
    if nargin < 1 || isempty(ids)
        d = asils.util.listinputs(fullfile(R, 'data', 'trades'), '*.json');
        ids = cellfun(@(n) n(1:end-5), {d.name}, 'UniformOutput', false);
    end
    if ischar(ids), ids = {ids}; end
    if exist('OCTAVE_VERSION', 'builtin'), graphics_toolkit('gnuplot'); end
    for i = 1:numel(ids)
        T = asils.trade.run(ids{i});
    end
end
