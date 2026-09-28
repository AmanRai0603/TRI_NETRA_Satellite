function print(Sol)
%ASILS.SOLUTION.PRINT  Console view of a case's solution.
    fprintf('\nSolution for %s (%s class)\n', Sol.case, Sol.class);
    for m = Sol.mode_ids
        M = Sol.modes.(m{1});
        fprintf('  %s — objective %s (worst over seeds)\n', M.label, M.objective);
        for o = M.options
            tag = 'FAIL'; if o.feasible, tag = 'pass'; end
            fprintf('    %-11s %-4s  %10.4g   power %6.2f W   %s\n', o.id, tag, o.obj, o.power, strjoin(o.failed, ','));
        end
    end
    fprintf('  %-15s %-9s %-6s %8s %8s %8s  methods\n', 'family', 'role', 'modes', 'mass kg', 'P nom W', 'vol L');
    f = fieldnames(Sol.families);
    for i = 1:numel(f)
        E = Sol.families.(f{i});
        mm = cellfun(@(m) E.methods.(m).option, Sol.mode_ids, 'UniformOutput', false);
        fprintf('  %-15s %-9s %d/%d    %8.3f %8.2f %8.3f  %s\n', E.id, E.role, E.passes, numel(Sol.mode_ids), ...
            E.mass_kg, E.power_nominal_W, E.volume_L, strjoin(mm, ' | '));
    end
    fprintf('  RECOMMENDED: %s — %s\n', Sol.recommended, Sol.verdict);
end
