function G = geometry(sc)
%ASILS.ENV.GEOMETRY  Facet model of the spacecraft for the disturbance torques.
%   A 3U box (long axis +X_B, 0.34 x 0.10 x 0.10 m by default) as six flat
%   plates. Each facet: outward normal n (3xN), area A (1xN), centroid rho
%   relative to the CENTRE OF MASS (3xN) -- the CM is offset from the
%   geometric centre by sc.cm_offset_m, which is what makes aero and SRP
%   torques nonzero, exactly as the case's CP-CM offset intends.
    Lx = sc.box_m(1); Ly = sc.box_m(2); Lz = sc.box_m(3);
    n = [ 1 -1  0  0  0  0;
          0  0  1 -1  0  0;
          0  0  0  0  1 -1];
    A = [Ly*Lz, Ly*Lz, Lx*Lz, Lx*Lz, Lx*Ly, Lx*Ly];
    c = n .* repmat([Lx; Ly; Lz]/2, 1, 6);
    G.n = n; G.A = A;
    G.rho = c - repmat(sc.cm_offset_m(:), 1, 6);
    G.sigma_n = sc.sigma_n; G.sigma_t = sc.sigma_t; G.vb_ratio = sc.vb_ratio;
    G.rho_spec = sc.refl*sc.spec_frac; G.rho_diff = sc.refl*(1 - sc.spec_frac);
end
