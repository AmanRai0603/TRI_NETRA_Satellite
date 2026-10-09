function J = jobs(caseId, seeds)
%ASILS.SOLUTION.JOBS  Every (mode, option, seed) run of a case's solution
%   matrix, in a fixed order (workers index into it).
    if nargin < 2, seeds = [1 2]; end
    J = struct('mode', {}, 'option', {}, 'seed', {}, 'cost', {});
    R = asils.util.root();
    C = asils.case.read(fullfile(R, 'cases', [caseId '.csv']));
    T = asils.models.caseorbit.dispersed_period(C.v.orbit_alt*1e3);   % env_case_orbit's period (generated)
    for m = asils.solution.mode()
        M = asils.solution.mode(m{1});
        for i = 1:numel(M.options)
            o = M.options{i};
            cost = asils.util.getf(o, 'duration_orbits', M.test.duration_orbits)*T/o.dt_s;
            for s = seeds(:)'
                J(end+1) = struct('mode', m{1}, 'option', o.id, 'seed', s, 'cost', cost); %#ok<AGROW>
            end
        end
    end
end
