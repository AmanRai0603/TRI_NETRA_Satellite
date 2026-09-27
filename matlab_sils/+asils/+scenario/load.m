function S = load(id)
%ASILS.SCENARIO.LOAD  Read data/scenarios/<id>.json (the exported scenario).
    R = asils.util.root();
    f = fullfile(R, 'data', 'scenarios', [id '.json']);
    assert(exist(f, 'file') == 2, 'asils:scenario:missing', 'No scenario %s (%s)', id, f);
    S = asils.util.readjson(f);
    S = asils.util.caseid(S);        % 'case' is a MATLAB keyword: jsondecode renames it
    if isfield(S, 'metrics') && isstruct(S.metrics), S.metrics = num2cell(S.metrics); end
end
