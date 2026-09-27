function W = buildWorld(cfg)
%OP.BUILDWORLD  Pre-load everything reused across acceleration evaluations.
%   W = op.buildWorld(cfg)
%   Loads the gravity field, opens the DE440 kernel once, resolves the spacecraft
%   and epoch, and captures frame settings.  Passed to op.accel each step so no
%   file I/O happens inside the integrator loop.
    W.cfg   = cfg;
    W.epoch = cfg.epoch(:).';
    W.grav  = op.gravLoad(getf(cfg,'gravityField',struct()));
    W.sc    = cfg.spacecraft;

    % ---- ATTITUDE. If the caller supplied a MEASURED quaternion series, unwrap it
    % ONCE here. q and -q are the same rotation and ITSG's files flip sign on about
    % half the epochs; interpolating across a flip swings through the far side of the
    % hypersphere and gives up to 180 deg of attitude error. Doing it per step would
    % be both wrong (each step sees only a local pair) and slow.
    W.att = getf(cfg.spacecraft,'attitude',[]);
    if ischar(W.att) && strcmpi(W.att,'ram')
        fprintf('[attitude] RAM-pointing (modelled): body +x along v_rel. A(t) constant, Cd(t) from GSI.\n');
    elseif isstruct(W.att) && isfield(W.att,'q') && ~isempty(W.att.q)
        [W.att.q, nflip] = validation.quat_continuous(W.att.q);
        fprintf('[attitude] MEASURED: %d epochs, %d sign flips unwrapped ONCE (not per step)\n', ...
                numel(W.att.mjd), nflip);
    end
    W.frame = getf(cfg,'frame',struct('build','gmst','dUT1',0));

    % --- EOP CACHE. The A/B/C builds download finals2000A / EOP 20 C04 and cache
    % them in opt.data_dir. That defaults to '' -> a folder beside the function, so
    % EOP landed OUTSIDE data.root() and "clear the cache" never cleared it.
    % data.eop_dir()'s own header tells you to pass this; nothing ever did. Wire it
    % here so every build in the toolbox shares one cache, without each script
    % having to remember.
    if ~isfield(W.frame,'data_dir') || isempty(W.frame.data_dir)
        % A SWALLOWED FAILURE HERE IS EXPENSIVE. If data.eop_dir() throws, this used
        % to leave W.frame.data_dir unset and say nothing -- so the frame build
        % silently fell back to whatever it does without EOP (dUT1=0 costs ~450 m on
        % build 'gmst'), and every downstream number carried that with no trace.
        % The eop_dir call CAN legitimately fail offline, so this is not an error --
        % but it is not nothing either. Set the field and say what happened.
        W.frame.data_dir = '';
        try
            W.frame.data_dir = data.eop_dir();
        catch err_
            warning('op:buildWorld:noEOP', ...
              ['could not resolve the EOP directory (%s). The frame build will use ' ...
               'whatever defaults it has -- on build ''gmst'' that means dUT1=0, ' ...
               'worth ~450 m. Set cfg.frame.data_dir explicitly if you have EOP ' ...
               'cached.'], regexprep(err_.message,'\n.*',''));
        end
    end

    % --- TRUE Earth rotation vector in ECI (NOT [0;0;omega]) --------------------
    % The Earth turns about the CIP. For the IAU2006 builds ('B','C') the CIP has
    % precessed off the ECI z-axis by ~168 arcsec by 2008 (~20 arcsec/yr), so
    % assuming [0;0;omega] misplaces any rotating<->inertial velocity by
    % omega*theta*r ~ 0.4 m/s at LEO. Recover it from the rotation matrix itself:
    %   r_eci = Ct*r_ecef  =>  d/dt = Cdot*r_ecef = (Cdot*C)*r_eci = omega x r_eci
    % so [omega]x = Cdot*C. Computed ONCE at the epoch (the CIP is effectively
    % fixed over an arc), and it is exact for gmst/A/B/C alike.
    W.omega_eci = earthRateECI(cfg.epoch, getf(W.frame,'build','gmst'), W.frame);

    % --- validate spacecraft when non-conservative forces are on (prevents a
    %     silent NaN/Inf from a zero/absent mass or area poisoning the run) ---
    dON = isfield(cfg.forces,'drag') && isfield(cfg.forces.drag,'on') && cfg.forces.drag.on;
    sON = isfield(cfg.forces,'srp')  && isfield(cfg.forces.srp,'on')  && cfg.forces.srp.on;
    if dON || sON
        if ~(isfield(W.sc,'mass') && isscalar(W.sc.mass) && W.sc.mass>0)
            error('buildWorld:spacecraft','spacecraft.mass must be a positive scalar for drag/SRP (got %s)', mat2str(getf(W.sc,'mass',NaN)));
        end
        if ~(isfield(W.sc,'Aref') && isscalar(W.sc.Aref) && W.sc.Aref>0)
            error('buildWorld:spacecraft','spacecraft.Aref must be a positive scalar for drag/SRP (got %s)', mat2str(getf(W.sc,'Aref',NaN)));
        end
    end

    % open the ephemeris kernel once (reused by ephemInputs)
    W.eph = []; ephErr='';
    try,  W.eph = de440.open();  catch ME,  W.eph = []; ephErr=ME.message; end
    if isempty(ephErr), ephErr='unknown'; end
    % if a force needs Sun/Moon but the kernel failed, do NOT fail silently
    needE = false; en={'thirdbody','srp','erp','relativity','solidtides','oceantides'};
    for i=1:numel(en), n=en{i}; if isfield(cfg.forces,n)&&isfield(cfg.forces.(n),'on')&&cfg.forces.(n).on, needE=true; break; end, end
    if isempty(W.eph) && needE
        warning('buildWorld:ephemeris', ['DE440 kernel did not load (%s). Third-body/SRP/ERP/tides need Sun/Moon ' ...
            'positions; ensure 03_frames_time/ephemeris/data/de440s.bsp is present. Results for those terms will be wrong.'], ephErr);
    end

    % --- warn ONCE if the requested gravity degree exceeds the loaded field ---
    g = getf(cfg.forces,'gravity',struct());
    if isfield(g,'degree') && strcmpi(getf(g,'model',''),'sphharm') ...
            && isfield(W.grav,'nmax') && g.degree > W.grav.nmax
        warning('buildWorld:gravDegree', ['requested gravity degree %d exceeds the loaded field max degree %d ' ...
            '(field=%s) -- using %d. For higher degree set cfg.gravityField.field=''EGM2008'' (or ''EIGEN-6C4'').'], ...
            g.degree, W.grav.nmax, getf(W.grav,'name','default'), W.grav.nmax);
    end

    % ========================================================================
    %  DATE-DEPENDENT DRIVER DATA -- resolved ONCE, here, for the selected epoch.
    % ========================================================================
    %  This is the ONLY place driver data is fetched. Nothing below op.accel may
    %  touch disk or network: it runs inside the integrator's RHS, thousands of
    %  times per arc.
    %
    %  WHICH drivers depends on WHICH model -- each density model eats a different
    %  set, and they are not interchangeable. That mapping is not decided here; it
    %  lives in data.drivers, which mirrors the segmentation proven in
    %  GOCE_density_study (2_spaceweather/ + align_drivers_to_track.m).
    %
    %    exponential       -> nothing
    %    nrlmsise          -> F10.7 + ap (+57 h aph history)
    %    dtm2020           -> F10.7 + Kp/ap            (operational, dtm3)
    %    dtm2020_research  -> F30 + ap60               (research, dtm5)
    %    jb2008            -> F10/S10/M10/Y10 + DSTDTC (SET)
    %
    %  The window is driven by cfg.epoch + cfg.tspan, so a sweep over DATE refetches
    %  for the new date instead of reusing whatever the first run pulled.
    W.swtable = []; W.swmanual = []; W.jbidx = []; W.drv = [];
    scfg   = getf(cfg,'spaceweather',struct());
    W.swmanual = getf(scfg,'manual',[]);
    d      = getf(cfg.forces,'drag',struct('on',false));
    dragOn = isfield(d,'on') && d.on;
    model  = lower(getf(d,'atmos','exponential'));

    % pad=3, not 2: NRLMSISE's aph(7) is the mean ap over t-36..-57h = 2.375 days
    % back, and the F10.7 the models want is t-24h. At a 00:00 UTC epoch a 2-day pad
    % put the -57h bin BEFORE the table started, so the storm history silently
    % degraded to the daily Ap and the aph array became decorative.
    pad = getf(scfg,'padDays',3);
    if isscalar(cfg.tspan), dur=cfg.tspan; else, dur=cfg.tspan(2)-cfg.tspan(1); end
    d0 = datenum(cfg.epoch(1),cfg.epoch(2),cfg.epoch(3)) - pad;
    d1 = d0 + dur/86400 + 2*pad;
    s0 = datestr(d0,'yyyy-mm-dd'); s1 = datestr(d1,'yyyy-mm-dd');

    if dragOn
        dopts = scfg;
        if ~isempty(W.swmanual), dopts.manual = W.swmanual; end
        try
            W.drv = data.drivers(model, s0, s1, dopts);
        catch ME
            error('buildWorld:drivers', ...
              ['could not resolve the %s drivers for %s..%s: %s\n' ...
               'Each density model needs its OWN indices (see data.drivers). Fix the ' ...
               'fetch, or set cfg.spaceweather.manual for the F10.7-driven models, or ' ...
               'use atmos=''exponential'' if you genuinely want no space weather.'], ...
               model, s0, s1, regexprep(ME.message,'\n.*',''));
        end
        % keep the legacy fields populated: op.accel/forces.drag and several
        % scripts still read W.swtable/W.jbidx by name.
        if isstruct(W.drv)
            if isfield(W.drv,'swtable'), W.swtable = W.drv.swtable; end
            if isfield(W.drv,'jbidx'),   W.jbidx   = W.drv.jbidx;   end
        end
    end

    % ---- say what was resolved. Every space-weather bug in this toolbox has been
    %      silent; a driver you cannot see is a driver you cannot audit.
    % NB the {{...}} is not a typo: struct() unwraps ONE cell level, so a bare cell
    % value would silently build a 1x2 STRUCT ARRAY instead of one struct.
    f30src = '';
    if isstruct(W.drv) && isfield(W.drv,'f30src'), f30src = W.drv.f30src; end
    W.drivers = struct('window',{{s0,s1}}, 'atmos',model, ...
                       'swtable', ~isempty(W.swtable), 'swmanual', ~isempty(W.swmanual), ...
                       'jbidx',   ~isempty(W.jbidx), 'f30src', f30src, ...
                       'eop_dir', getf(W.frame,'data_dir',''), 'frame', getf(W.frame,'build','gmst'));
end
function val=getf(s,f,d), if isfield(s,f)&&~isempty(s.(f)), val=s.(f); else, val=d; end, end

function w = earthRateECI(epoch, build, fopt)
%EARTHRATEECI  Earth angular-velocity vector expressed in ECI [rad/s].
    try
        d = 15.0;   % see note below
        up = op.addsec(epoch,  d); um = op.addsec(epoch, -d);
        Tp = timeconv.convertUTC(up(1),up(2),up(3),up(4),up(5),up(6),0);
        Tm = timeconv.convertUTC(um(1),um(2),um(3),um(4),um(5),um(6),0);
        T0 = timeconv.convertUTC(epoch(1),epoch(2),epoch(3),epoch(4),epoch(5),epoch(6),0);
        fp = fopt; fp.gmst_rad = Tp.gmst_rad;
        fm = fopt; fm.gmst_rad = Tm.gmst_rad;
        f0 = fopt; f0.gmst_rad = T0.gmst_rad;
        [~, Ctp] = frames.eci2ecef(up,    build, fp);
        [~, Ctm] = frames.eci2ecef(um,    build, fm);
        [C0, ~ ] = frames.eci2ecef(epoch, build, f0);
        Cdot = (Ctp - Ctm)/(2*d);
        M = Cdot * C0;                       % = [omega]x  (skew-symmetric)
        w = [M(3,2); M(1,3); M(2,1)];
    catch ME
        % SILENT NO LONGER. Falling back to [0;0;omega] is precisely the error this
        % function's own header (30 lines up) says costs ~0.4 m/s at LEO, because the
        % Earth turns about the CIP and the CIP is ~168 arcsec off the ECI z-axis by
        % 2008. Swallowing that quietly meant a failed EOP fetch downgraded your
        % frame accuracy with no trace in the log.
        K = de440.constants(); w = [0;0;K.omega_earth];
        warning('op:buildWorld:earthRate', ...
          ['could not derive the true Earth rate from the ''%s'' frame build (%s).\n' ...
           'Falling back to [0;0;omega], which ignores CIP precession: expect ~0.4 m/s ' ...
           'of error in any rotating<->inertial velocity, and a corresponding co-rotating ' ...
           'drag error. Usually an EOP fetch failure -- fix that rather than accept this.'], ...
           build, regexprep(ME.message,'\n.*',''));
    end
end
