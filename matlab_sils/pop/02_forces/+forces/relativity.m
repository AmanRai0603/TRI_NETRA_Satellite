function a = relativity(ctx)
%FORCES.RELATIVITY  Post-Newtonian (IERS2010) acceleration (ECI), term-switchable.
%   Reads ctx.cfg.forces.relativity.terms (subset of 'schwarzschild',
%   'lensethirring','desitter').  Schwarzschild dominates (~1e-9 m/s^2 at LEO);
%   Lense-Thirring & de Sitter are ~1e-11.
%
%   ONE DISPATCH. relativity.total already owns the term switch, and this function
%   used to carry a SECOND copy of it while relativity.total sat uncalled. Two
%   switches for one decision is precisely how drag.force and drag.panelCoeffs
%   drifted apart -- 'sesam' existed in one and not the other, fell through both
%   cases, and surfaced as "'ct' undefined" three lines later.
%
%   mu is passed through deliberately: the sub-terms take the GRAVITY FIELD's own
%   mu, not de440's. Those differ by ~7.5e-10 relative (different fits), and using
%   the field's keeps this consistent with the gravity that dominates the orbit.
    c     = ctx.cfg.forces.relativity;
    terms = getf(c,'terms',{'schwarzschild'});
    a = relativity.total(ctx.r_eci, ctx.v_eci, getf(ctx,'E',[]), terms, ctx.grav.mu);
end
function val=getf(s,f,d), if isfield(s,f)&&~isempty(s.(f)), val=s.(f); else, val=d; end, end
