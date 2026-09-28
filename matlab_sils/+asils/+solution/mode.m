function M = mode(modeId)
%ASILS.SOLUTION.MODE  A mission-mode definition (data/modes/<id>.json, from
%   catalogue/modes/<id>.toml), with its options and metrics as cell arrays.
%   With no argument: the ids of every mode, in mission order.
    R = asils.util.root();
    if nargin < 1
        d = dir(fullfile(R, 'data', 'modes', '*.json'));
        ids = cellfun(@(n) n(1:end-5), {d.name}, 'UniformOutput', false);
        o = zeros(1, numel(ids));
        for i = 1:numel(ids), Mi = asils.solution.mode(ids{i}); o(i) = Mi.order; end
        [~, k] = sort(o); M = ids(k); return
    end
    M = asils.util.readjson(fullfile(R, 'data', 'modes', [modeId '.json']));
    if isstruct(M.options), M.options = num2cell(M.options); end
    if isstruct(M.metrics), M.metrics = num2cell(M.metrics); end
    for i = 1:numel(M.options)
        o = M.options{i};
        u = {'mtq', o.actuator}; if isfield(o, 'dump'), u{end+1} = o.dump; end
        M.options{i}.uses = unique(u);                      % actuators the option needs
    end
end
