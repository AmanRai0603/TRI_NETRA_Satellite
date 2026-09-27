function [cmd_r, cmd_g] = allocate(tau_rot, z, F, P)
%ASILS.FSW.ALLOCATE  Body torque -> momentum-exchange device commands.
%
%   tau_rot      body torque the momentum-exchange devices must deliver [N m]
%   z.h, z.delta measured rotor momenta and gimbal angles (tachometers, encoders)
%
%   Fixed-axis rotors (reaction wheels, fluid rings): minimum-norm
%     cmd_r = -pinv(A_ok) tau_rot over the rotors NOT isolated by FDIR.
%   Gimballed rotors: singularity-robust steering (asils.fsw.steer_sr);
%     a VSCMG adds wheel-mode torque near the gimbal singularities.
    dev = P.dev; M = F.M;
    nr = M.nr; ng = M.ng;
    cmd_r = zeros(nr, 1); cmd_g = zeros(ng, 1);
    if nr == 0, return, end
    A = asils.plant.axes(M, z.delta);
    fixed = find(M.gi(:) == 0 & ~F.rot_failed(:));
    if ~isempty(fixed)
        Af = A(:, fixed);
        cmd_r(fixed) = -pinv(Af)*tau_rot;
    end
    if ng > 0
        wheels = any(strcmp(dev.mex.kind, 'vscmg'));
        [cmd_g, hdot, ~] = asils.fsw.steer_sr(tau_rot, A, z.h, M, dev.mex.gimbal_rate_max, wheels, P.fsw.cmg.lam0, P.fsw.cmg.mu);
        if wheels
            g = M.gi > 0;
            % null motion on the wheels: pull rotor speeds back to h0 slowly
            cmd_r(g) = hdot(g) - P.fsw.cmg.k_null*(z.h(g) - dev.mex.h0(g)');
        end
    end
end
