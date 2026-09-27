function [s_meas, valid] = css(s_B, nu, nadir_B, earth_ang, D, c)
%ASILS.DEVICES.CSS  Coarse sun sensor: one cosine photodiode per face.
%   Current of diode i (fraction of full sun):
%     I_i = k_i [ nu max(0, n_i.s) + albedo * F_i ] + noise
%   F_i: Earth-albedo irradiance on face i -- the sunlit Earth disc seen
%   through the face's cosine, (sin rho)^2 max(0, n_i.nadir) max(0, -s.nadir)
%   (rho the Earth angular radius). The FSW estimate is the difference of
%   opposite faces, normalised: albedo is its dominant error in LEO.
    n = c.normals; k = numel(D.scale);
    geo = sin(earth_ang)^2*max(0, -s_B'*nadir_B);
    Iv = D.scale(:).*( nu*max(0, n'*s_B) + c.albedo*geo*max(0, n'*nadir_B) ) + c.noise*randn(k,1);
    Iv(D.dead) = 0;
    est = zeros(3,1);
    for ax = 1:3
        p = find(n(ax,:) > 0.9); m = find(n(ax,:) < -0.9);
        if ~isempty(p) && ~isempty(m), est(ax) = Iv(p(1)) - Iv(m(1)); end
    end
    valid = nu > 0.5 && norm(est) > 0.3;
    if valid, s_meas = D.R*est/norm(est); else, s_meas = [0;0;0]; end
end
