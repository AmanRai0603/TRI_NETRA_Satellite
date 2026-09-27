function D = spec(tradeId)
%ASILS.TRADE.SPEC  Read data/trades/<id>.json (exported from trades/<id>.toml).
    R = asils.util.root();
    f = fullfile(R, 'data', 'trades', [tradeId '.json']);
    assert(exist(f, 'file') == 2, 'asils:trade:missing', 'No trade %s (%s)', tradeId, f);
    D = asils.util.readjson(f);
    if isstruct(D.candidates), D.candidates = num2cell(D.candidates); end
    if ~isfield(D, 'seeds'), D.seeds = 1; end
end
