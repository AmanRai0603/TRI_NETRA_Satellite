function code = try_model(SAT, C, CAP, kind, model, ALT_KM, RUN_LIVE, K)
%VALIDATION.TRY_MODEL  Is this model allowed for this satellite -- and does it RUN?
%   code: 1 = allowed and ran   0 = not allowed (data missing)   -1 = allowed but FAILED
%
%   A file, not a script local: MATLAB hoists a script's locals and Octave does not,
%   which is the bug that started this whole audit.
%
%   "Allowed" comes from validation.capability, i.e. from the DATA in the catalog.
%   Then we RUN it, because a capability table that only agrees with itself proves
%   nothing -- sat.reads went stale within one round of erp.boxwing landing and
%   still claimed no ERP box-wing existed while one was running.
    allowed = CAP.(kind);
    ok = false;
    for i=1:numel(allowed)
        if strcmpi(model, allowed{i}) || ~isempty(strfind(lower(allowed{i}), [lower(model) '(']))
            ok = true; break
        end
    end
    if ~ok, code = 0; return, end
    if ~RUN_LIVE, code = 1; return, end

    try
        alt = ALT_KM; if isfinite(C.alt) && C.alt>100, alt = C.alt; end
        a  = K.Re_earth + alt*1000;
        r  = [a;0;0]; v = [0; sqrt(K.mu_earth/a); 0];
        % aref_box: the ASSUMED shape, ram face == Aref by construction
        OPT = struct('alpha',0.2,'rho_s',0.3,'rho_d',0.5);
        Ly = sqrt(C.Aref); Lz = C.Aref/Ly; Lx = Ly;
        F  = srp.buildBox(Lx, Ly, Lz, OPT);

        cfg = config.defaultConfig();
        cfg.epoch=[2007 1 1 0 0 0]; cfg.r0=r; cfg.v0=v; cfg.tspan=60;
        cfg.output=struct('times',[0;60]);
        cfg.gravityField=struct('field','default','degree',4);
        cfg.spaceweather.manual=struct('F107',90,'F107a',90,'ap',8,'Kp',2);
        cfg.spacecraft=struct('mass',C.mass,'Aref',C.Aref,'Cd',C.Cd,'Cr',C.Cr, ...
                              'facets',F,'attitude','ram','R_bi',eye(3));
        cfg.forces=struct('gravity',struct('on',true,'model','twobody'), ...
          'drag',struct('on',false),'thirdbody',struct('on',false), ...
          'srp',struct('on',false),'erp',struct('on',false), ...
          'relativity',struct('on',false),'solidtides',struct('on',false), ...
          'oceantides',struct('on',false));
        switch kind
            case 'drag'
                cfg.forces.drag = struct('on',true,'model',model,'atmos','dtm2020', ...
                    'corotate',true,'gsi',struct('Tw',300,'aT',0.9,'sig_n',0.9,'sig_t',0.9));
            case 'srp'
                cfg.forces.srp = struct('on',true,'model',model);
            case 'erp'
                cfg.forces.erp = struct('on',true,'model',model,'nrings',4,'nseg',8);
        end
        W = op.buildWorld(cfg);
        [~, parts] = op.accel(0, r, v, W);
        val = parts.(kind);
        if ~all(isfinite(val)) || norm(val) <= 0
            code = -1;    % ran, produced nothing usable -- the zero-drag trap
        else
            code = 1;
        end
    catch
        code = -1;
    end
end
