function [gdot, hdot, msing] = steer_sr(tau, A, h, M, gmax, wheels, lam0, mu)
%ASILS.FSW.STEER_SR  Singularity-robust steering of a CMG / VSCMG cluster.
%   Torque on the body: tau = Jg*gdot + Jw*hdot,
%     Jg(:,j) = -h_i (g_j x a_i)   (gimbal j carries rotor i)
%     Jw(:,i) = -a_i               (VSCMG wheel mode)
%   u = W J' (J W J' + lambda I)^-1 tau,  lambda = lam0 exp(-mu m),
%   m = det(Jg Jg')/h0^6 the gimbal singularity measure (Wie 2008). For a
%   VSCMG the wheel columns are weighted up as m falls, so the cluster keeps
%   full torque through a gimbal singularity (Schaub & Junkins).
    ng = M.ng; nr = M.nr;
    Jg = zeros(3, ng);
    for i = 1:nr
        j = M.gi(i);
        if j > 0
            a = A(:,i); gg = M.G(:,j);
            Jg(:,j) = -h(i)*[gg(2)*a(3)-gg(3)*a(2); gg(3)*a(1)-gg(1)*a(3); gg(1)*a(2)-gg(2)*a(1)];
        end
    end
    h0 = max(abs(h(M.gi > 0)));
    msing = det(Jg*Jg')/max(h0, 1e-12)^6;
    if wheels
        Jw = -A; Ww = 0.01 + 2*exp(-10*msing);         % more wheel mode near singularity
        J = [Jg, Jw]; W = blkdiag(eye(ng), Ww*eye(nr));
    else
        J = Jg; W = eye(ng);
    end
    lam = lam0*exp(-mu*msing);
    u = W*J'*((J*W*J' + lam*eye(3))\tau);
    gdot = u(1:ng);
    s = max(1, max(abs(gdot))/gmax); gdot = gdot/s;          % keep the direction when rate-limited
    hdot = zeros(nr, 1);
    if wheels, hdot = u(ng+1:end)/s; end
end
