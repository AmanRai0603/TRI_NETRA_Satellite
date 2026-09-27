function a = thirdbody(ctx)
%FORCES.THIRDBODY  Sun+Moon point-mass third-body acceleration (ECI).
%   Reads ctx.cfg.forces.thirdbody.model ('battin'|'direct'|'tidal'|'legendre').
%   Uses ctx.E (ephemInputs) for Sun/Moon geocentric positions + GMs.
    c = ctx.cfg.forces.thirdbody;
    model = getf(c,'model','battin');
    a = thirdbody.total(ctx.r_eci, ctx.E, model);   % vendored package
end
function val=getf(s,f,d), if isfield(s,f)&&~isempty(s.(f)), val=s.(f); else, val=d; end, end
