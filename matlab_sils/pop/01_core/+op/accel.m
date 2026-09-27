function [a, parts, ctx, info] = accel(t, r, v, W)
%OP.ACCEL  Total ECI acceleration (Cowell sum of all enabled force models).
%   [a, parts] = op.accel(t, r, v, W)
%     t : seconds since epoch (W.epoch, UTC)
%     r,v : ECI (GCRF/J2000) position [m] and velocity [m/s]
%     W : world struct from op.buildWorld (cfg, grav field, kernel, spacecraft)
%   Returns total acceleration a (3x1, ECI) and a struct 'parts' with each term's
%   contribution (for the force-impact studies).
%
%   The per-step CONTEXT (time scales, ephemeris, one cached ECI<->ECEF rotation,
%   ECEF position) is built ONCE here and shared by every force term -- this is
%   the single interface point, mirroring the ephemInputs design.
    r=r(:); v=v(:);
    cfg = W.cfg;  F = cfg.forces;

    % ---------- shared per-step context ----------
    ctx.t = t;
    ctx.utc = op.addsec(W.epoch, t);
    ctx.T = timeconv.convertUTC(ctx.utc(1),ctx.utc(2),ctx.utc(3), ...
                                ctx.utc(4),ctx.utc(5),ctx.utc(6), getf(W.frame,'dUT1',0));
    % ephemeris (Sun/Moon) only if a force actually needs it -- saves a costly
    % Chebyshev evaluation on two-body / zonal-only runs.
    needE = anyOn(F,{'thirdbody','srp','erp','relativity','solidtides','oceantides'});
    if needE
        if ~isempty(W.eph), ctx.E = ephemInputs(ctx.T.tdb_jd, W.eph);
        else,               ctx.E = ephemInputs(ctx.T.tdb_jd); end
    else
        ctx.E = [];
    end
    % one cached ECI<->ECEF transform (build 'gmst' is offline & fast)
    fopt = W.frame; fopt.gmst_rad = ctx.T.gmst_rad;
    [ctx.C, ctx.Ct] = frames.eci2ecef(ctx.utc, getf(W.frame,'build','gmst'), fopt);
    ctx.omega_eci = getf(W,'omega_eci',[0;0;7.2921150e-5]);   % true Earth rate in ECI
    ctx.r_eci=r; ctx.v_eci=v; ctx.r_ecef=ctx.C*r;
    % TRUE Earth-fixed velocity (C*v_eci alone is just the inertial velocity
    % expressed in ECEF axes -- NOT the ECEF velocity; that omission is a 490 m/s
    % trap for anything that consumes it).
    ctx.v_ecef = ctx.C*(v - cross(ctx.omega_eci, r));
    ctx.cfg=cfg; ctx.sc=W.sc; ctx.grav=W.grav; ctx.eph=W.eph;

    % ---- THE ATTITUDE, AT THIS EPOCH -------------------------------------------
    % This is what makes A(t) and Cd(t) real. The panel drag models, box-wing SRP
    % and box-wing ERP all read ctx.sc.R_bi. It used to be whatever constant sat in
    % cfg.spacecraft.R_bi (defaultConfig: eye(3)), so the "attitude-dependent"
    % models were handed a FROZEN attitude and returned a frozen area.
    %
    % Worse than frozen: eye(3) puts the BODY axes on the INERTIAL axes, so a +x
    % plate faces inertial +x rather than the flow. On a +y-moving satellite that is
    % n.v = 0 and the drag is EXACTLY ZERO -- a silent, plausible-looking nothing.
    %
    %   W.att = 'ram'      -> dgeom.ramAttitude: body +x along v_rel (modelled)
    %   W.att = struct     -> measured quaternions, interpolated (op.attitudeAt)
    %   W.att = []         -> leave sc.R_bi alone (cannonball never reads it)
    if isfield(W,'att') && ~isempty(W.att)
        om_ = getf(W,'omega_eci',[0;0;7.2921150e-5]);
        ctx.v_rel_eci = v(:) - cross(om_, r(:));
        if ischar(W.att) && strcmpi(W.att,'ram')
            ctx.sc.R_bi = dgeom.ramAttitude(ctx.v_rel_eci);
        else
            ctx.sc.R_bi = op.attitudeAt(W.att, ctx.utc);
        end
    end
    ctx.swtable=W.swtable; ctx.swmanual=W.swmanual;
    % The full per-model driver bundle from data.drivers (F30/ap60 for the research
    % DTM, SET indices for JB2008, ...). Carried down so nothing below here ever
    % touches the disk. ctx.jbidx kept for the readers that use it by name.
    ctx.jbidx = getf(W,'jbidx',[]);
    ctx.drv   = getf(W,'drv',[]);

    % ---------- sum enabled forces ----------
    a = zeros(3,1); parts = struct();
    % gravity is mandatory (central body)
    info = struct();
    ag = forces.gravity(ctx);  a=a+ag; parts.gravity=ag;
    order = {'thirdbody','drag','srp','erp','relativity','solidtides','oceantides','empirical'};
    for k=1:numel(order)
        nm = order{k};
        if isfield(F,nm) && isfield(F.(nm),'on') && F.(nm).on
            if strcmp(nm,'drag')
                % grab drag's internals for diagnostics; the force path is unchanged
                [ak, info.drag] = forces.drag(ctx);
            else
                ak = forces.(nm)(ctx);
            end
            a = a + ak;  parts.(nm) = ak;
        end
    end
end
function tf=anyOn(F,names)
    tf=false;
    for i=1:numel(names)
        n=names{i};
        if isfield(F,n)&&isfield(F.(n),'on')&&F.(n).on, tf=true; return; end
    end
end
function val=getf(s,f,d), if isfield(s,f)&&~isempty(s.(f)), val=s.(f); else, val=d; end, end
