function out = cannonball(r, v, rho, Cd, A, opts)
%DRAG.CANNONBALL  Basic (isotropic) drag: constant Cd, fixed area, no attitude,
%   no lift. a = -1/2 Cd (A/m) rho |v_rel| v_rel  (co-rotating v_rel).
%   The reference model to compare the panel models (drag.force) against.
%
%   ---------------------------------------------------------------------------
%   THIS WAS DEAD CODE, AND forces/drag.m HAD ITS OWN COPY
%   ---------------------------------------------------------------------------
%   Nothing called this. forces/drag.m carried the cannonball INLINE:
%       a = -0.5*rho*Cd*AoM*norm(v_rel)*v_rel;
%   Two implementations of one model. And they DISAGREED: this one built v_rel from
%   a SCALAR omega via drag.relVelocity (cross([0;0;omega],r)), while forces/drag.m
%   used the true Earth-rate VECTOR om_eci, which points along the CIP -- ~168
%   arcsec off the ECI z-axis, worth ~0.4 m/s of v_rel. Whichever you called, you
%   got a different answer, and nothing compared them.
%
%   Now: opts.omega accepts a scalar OR a 3-vector, and forces/drag.m calls this
%   instead of re-implementing it. One model, one place.
%
%   opts: .mass [kg] (required), .wind (ECI m/s), .omega (scalar rad/s or 3x1 ECI)
    if ~isfield(opts,'wind') || isempty(opts.wind),  opts.wind=[0;0;0]; end
    if ~isfield(opts,'omega')||isempty(opts.omega), opts.omega=7.2921150e-5; end
    r=r(:); v=v(:);
    om = opts.omega;
    if isscalar(om), om = [0;0;om]; end        % scalar = the naive z-axis rate
    vrel = v - cross(om, r) - opts.wind(:);
    V = norm(vrel); uhat = vrel/V;
    F = -0.5*Cd*A*rho*V*vrel;                 % Newtons
    out.F=F; out.a=F/opts.mass; out.drag=0.5*Cd*A*rho*V^2;
    out.drag_vec=F; out.lift=0; out.lift_vec=[0;0;0];
    out.uhat=uhat; out.Vrel=V; out.Cd=Cd; out.Cl=0;
    % same diagnostic contract as drag.force, so a caller can treat them alike:
    out.qd=0.5*rho*V^2; out.A_proj=A; out.Cd_A=Cd; out.Aref_used=A;
end
