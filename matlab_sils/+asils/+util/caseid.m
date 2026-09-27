function S = caseid(S)
%ASILS.UTIL.CASEID  Expose the JSON key "case" as S.case_id ('case' is a MATLAB
%   keyword, so jsondecode renames it -- xCase in MATLAB and Octave).
    f = fieldnames(S);
    k = find(strcmp(f, 'xCase') | strcmp(f, 'x_case') | strcmp(f, 'case_'), 1);
    if ~isempty(k), S.case_id = S.(f{k}); end
end
