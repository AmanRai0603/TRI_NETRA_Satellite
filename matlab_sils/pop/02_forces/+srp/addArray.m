function F = addArray(F, A, axis, opt)
%SRP.ADDARRAY  Append one double-sided solar array (area [m^2], pivot AXIS).
    F(end+1) = srp.facet('array', [0;0;0], A, opt.alpha, opt.rho_s, opt.rho_d, axis, true);
end
