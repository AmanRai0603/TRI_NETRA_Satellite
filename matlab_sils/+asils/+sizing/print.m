function print(Z)
%ASILS.SIZING.PRINT  Console summary of a case's sizing.
    D = Z.demand;
    fprintf('\nSizing for %s (%s class)\n', Z.case, Z.class);
    fprintf('  disturbance peak %.3g N m (%s), stored %.3g N m s, secular %.3g N m s/orbit\n', ...
        D.tau_dist, D.worst_attitude, D.h_dist, D.h_secular);
    fprintf('  slew %g deg in %g s: %.3g N m s, %.3g N m; detumble %.3g N m s; B_min %.1f uT\n', ...
        D.slew_deg, D.slew_s, D.h_slew, D.tau_slew, D.h_detumble, D.B_min*1e6);
    fprintf('  required (margins x%.1f / x%.1f): h %.3g N m s, tau %.3g N m\n', D.k_h, D.k_tau, D.h_req, D.tau_req);
    for n = D.notes, fprintf('  note: %s\n', n{1}); end
    f = fieldnames(Z.families);
    fprintf('  %-16s %-10s %9s %9s %9s\n', 'family', 'role', 'mass kg', 'power W', 'vol L');
    for i = 1:numel(f)
        x = Z.families.(f{i});
        fprintf('  %-16s %-10s %9.3f %9.2f %9.3f\n', f{i}, x.role, x.mass_kg, x.power_W, x.volume_L);
    end
end
