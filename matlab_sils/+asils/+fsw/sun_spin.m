function [m0, V] = sun_spin(B, w, s, eclipse, J, g)
%ASILS.FSW.SUN_SPIN  L2 Sun-pointing spin law, coils only (He et al. 2023).
%   PORTED from Standard Code ctrl.sunSpin (theory doc sec. 5, eq 5.1-5.6):
%     h = J w,  h_d = sigma J_z w* s,  w* = -w_s  (sign erratum, theory 5.4)
%     R_z = diag(J_z - J_x, J_z - J_y, 0)
%     A = B x (k1 (h - h_d) + k2 R_z w),   m0 = -A / |B|^2
%   sigma = sign(w_z) is LIVE (never latched). The law needs only the rate, the
%   field and the Sun vector -- no attitude solution.
%   Eclipse: g.eclipse 'E1' coils off (Standard Code baseline); 'E2' the caller
%   passes the gyro-propagated Sun vector (ctrl.propagateSun) and eclipse=false.
%   g.rz_floor (0 = the published law): lower bound on the R_z diagonal as a
%   fraction of J_z. With J_y = J_z (a 3U box) the published R_z has no
%   damping on w_y at all; the floor restores nutation damping on both
%   transverse axes (variant sunspin_damped, this SILS).
%   V is the Lyapunov value (diagnostic). Unsaturated output.
    V = NaN;
    if eclipse, m0 = zeros(3,1); return, end
    fl = 0; if isfield(g, 'rz_floor'), fl = g.rz_floor*J(3,3); end
    ws = -abs(g.spin_dps*pi/180);
    sg = sign(w(3)); if sg == 0, sg = 1; end
    h = J*w; hd = sg*J(3,3)*ws*s(:);
    ht = h - hd;
    Rz = diag([max(J(3,3) - J(1,1), fl), max(J(3,3) - J(2,2), fl), 0]);
    A = asils.util.cross3(B, g.k1*ht + g.k2*(Rz*w));
    Bs = B'*B;
    if Bs < 1e-18, m0 = zeros(3,1); else, m0 = -A/Bs; end
    V = 0.5*(g.k1*(ht'*ht) + g.k2*(w'*Rz*J*w));
end
