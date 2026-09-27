function a = erp(ctx)
%FORCES.ERP  Earth radiation pressure (albedo + IR) acceleration (ECI).
%   Reads ctx.cfg.forces.erp.model ('knocke'|'simple'|'ceres') and .CrAoM.
%   Uses ctx.E.sun_eci and day-of-year.  ceres needs a CERES grid fcn (see
%   Extra Perturbation Forces/Albedo_IR_Model/data).
    c = ctx.cfg.forces.erp;
    % Cr PRECEDENCE: forces.erp.CrAoM -> forces.erp.Cr -> spacecraft.Cr -> 1.3.
    % ERP is the worst case of this bug: unlike SRP, NO script sets
    % cfg.forces.erp.Cr, so this silently used 1.3 for every satellite ever
    % propagated, ignoring cfg.spacecraft.Cr entirely. CHAMP's catalog Cr is
    % also 1.3, so on CHAMP it happened to be right -- which is exactly how a
    % bug like this survives.
    CrAoM = getf(c,'CrAoM', getf(c,'Cr', getf(ctx.sc,'Cr',1.3))*ctx.sc.Aref/ctx.sc.mass);
    model = getf(c,'model','knocke');
    % 'boxwing' reads the ATTITUDE and the FACETS instead of CrAoM. Everything else
    % here is a cannonball and cannot see either -- which is why this toggle used to
    % be a lie: whatever attitude you set, ERP ignored it.
    switch lower(model)
        case 'boxwing'
            % reads the ATTITUDE and the FACETS; CrAoM above is not consulted.
            % nrings/nseg default to Knocke's 16x48 = 768 elements, and boxwing
            % evaluates every facet at every element -- 768*nFacets per RHS call,
            % inside the integrator. That is genuinely expensive: a full orbit of
            % rk78 on an 8-facet 16U is ~10^7 facet evaluations. The cap integral
            % converges fast, so 8x16 = 128 elements costs 6x less and is usually
            % within a percent. Exposed rather than hidden:
            a = erp.accel(ctx.r_eci, ctx.E.sun_eci, [], 'boxwing', ...
                          getf(ctx.sc,'R_bi',eye(3)), ctx.sc, ctx.T.doy, ...
                          getf(c,'nrings',[]), getf(c,'nseg',[]));
        case {'knocke','simple'}
            a = erp.accel(ctx.r_eci, ctx.E.sun_eci, CrAoM, model, ctx.T.doy);
        case 'ceres'
            a = erp.accel(ctx.r_eci, ctx.E.sun_eci, CrAoM, 'ceres', getf(c,'gridFcn',[]));
        otherwise
            % There was no otherwise. An unknown model fell through the switch, left
            % `a` unassigned, and surfaced as "'a' undefined" from the caller -- the
            % same shape as the sesam bug in drag.force. Say what is wrong, here.
            error('forces:erp:model', ...
              ['unknown erp model "%s". Known: knocke, simple, ceres (all CANNONBALL ' ...
               '-- they read CrAoM and cannot see attitude), boxwing (reads sc.facets ' ...
               'and sc.R_bi).'], model);
    end
end
function val=getf(s,f,d), if isfield(s,f)&&~isempty(s.(f)), val=s.(f); else, val=d; end, end
