function [a, info] = drag(ctx)
%   [a, info] = forces.drag(ctx) also returns what the model actually used:
%     info.atm (rho, T, composition), info.geo (the geodetic probe point),
%     info.v_rel, and info.out (the panel model's own output: Cd, alpha, beta, qd).
%   This exists so a diagnostic can SEE the intermediates without reimplementing the
%   chain -- reimplementing it is how two copies drift apart (see drag.force vs
%   drag.panelCoeffs). Nothing in the force path changes: MATLAB and Octave both
%   allow ignoring a second output.
%FORCES.DRAG  Aerodynamic drag acceleration (ECI), model + atmosphere switchable.
%   Reads ctx.cfg.forces.drag:
%     .model  'cannonball' | 'sentman' | 'dria' | 'cll'
%     .atmos  'exponential' | 'nrlmsise' | 'jb2008' | 'dtm2020'
%     .Cd     drag coefficient (cannonball)      .corotate true/false
%     .gsi    struct (Tw, aT / sig_n,sig_t) for panel models
%   Spacecraft mass/area/facets come from ctx.sc.  Density comes from
%   atmos.provider using geodetic alt/lat/lon + LST derived from ctx.r_ecef.
    d  = ctx.cfg.forces.drag;
    sc = ctx.sc;
    omega = de440.constants(); omega = omega.omega_earth;

    % ---- geodetic + local solar time for the atmosphere lookup ----
    [lat,lon,alt] = op.geodetic(ctx.r_ecef);
    geo.alt_km = alt/1000; geo.lat_deg=rad2deg(lat); geo.lon_deg=rad2deg(lon);
    utsec = ctx.utc(4)*3600+ctx.utc(5)*60+ctx.utc(6);
    geo.lst_h = mod(utsec/3600 + geo.lon_deg/15, 24);
    geo.doy   = ctx.T.doy;  geo.utc = ctx.utc;
    % ---- the driver each model actually eats -------------------------------
    % Not one-size-fits-all. This used to call atmos.spaceweather for EVERY
    % non-exponential model, which meant a jb2008 run demanded an F10.7/ap table
    % it never opens -- and errored when you had not supplied one -- while the
    % SET indices it does need were never passed at all.
    model = lower(getf(d,'atmos','exponential'));
    switch model
        case 'exponential'
            sw = struct();                       % altitude only: no drivers
        case 'jb2008'
            % F10/S10/M10/Y10 + DSTDTC, resolved once in buildWorld for the epoch
            % window. jb2008_density applies the spec lags itself (F10,S10 at t-24h;
            % M10 t-48h; Y10 t-120h), so hand it the whole table, not a sample.
            sw = struct('jb_idx', getf(ctx,'jbidx',[]));
        case {'dtm2020_research','dtm2020_res'}
            % F30 + ap60. Sampled 'previous' (most recent value at or before this
            % epoch) -- the same rule align_drivers_to_track uses in the GOCE study.
            % Daily F30 by most-recent-day, hourly ap60 by most-recent-sample.
            sw = atmos.research_drivers(getf(ctx,'drv',[]), ctx.utc);
        otherwise                                % nrlmsise, dtm2020
            swopts = struct();
            if ~isempty(getf(ctx,'swmanual',[])), swopts.manual = ctx.swmanual; end
            if ~isempty(getf(ctx,'swtable',[])),  swopts.table  = ctx.swtable;  end
            sw = atmos.spaceweather(ctx.utc, swopts);
    end
    atm = atmos.provider(model, geo, sw);
    info = struct('atm',atm,'geo',geo,'sw',sw,'out',[],'v_rel',[]);

    % ---- relative velocity (co-rotating atmosphere) ----
    if getf(d,'corotate',true)
        % co-rotating atmosphere: use the TRUE Earth rate (about the CIP), not
        % [0;0;omega]. Small for drag (~0.4 m/s out of ~7.6 km/s) but it is the
        % same error class that cost 0.4 m/s on the SP3 seed -- keep it exact.
        om_eci = getf(ctx,'omega_eci',[0;0;omega]);
        v_rel = ctx.v_eci - cross(om_eci, ctx.r_eci);
        info.v_rel = v_rel;
    else
        v_rel = ctx.v_eci;
        info.v_rel = v_rel;
    end

    switch lower(getf(d,'model','cannonball'))
        case 'cannonball'
            rho = atm.rho;
            % Cd PRECEDENCE: forces.drag.Cd -> spacecraft.Cd -> 2.2.
            %
            % The spacecraft.Cd fallback is not cosmetic, it is a BUG FIX. Area and
            % mass on the next line come from sc, but Cd used to come only from
            % cfg.forces.drag, which every calling script leaves unset: they all
            % write Cd into cfg.spacecraft and then REPLACE cfg.forces wholesale
            % with their own FORCES struct, which has no Cd field. So this silently
            % read 2.2 for everything -- CHAMP propagated at 2.2 instead of its
            % catalog 3.0 (a 27% drag error) while config.report printed "Cd : 3",
            % because the report reads spacecraft and the physics read forces.drag.
            % EXAMPLE_16U's documented 'SC.Cd' sweep knob was a no-op for the same
            % reason. Note cfg.forces.srp.Cr = cfg.spacecraft.Cr appears in three
            % scripts -- somebody hit this for SRP and patched it there, and never
            % came back for drag.
            %
            % Fixing it HERE rather than in each script means it cannot be missed
            % again by the next script somebody writes.
            Cd  = getf(d, 'Cd', getf(sc, 'Cd', 2.2));
            % Call drag.cannonball rather than repeating its one line here. This
            % branch used to be an inline copy while drag.cannonball sat uncalled --
            % and the two disagreed, because this one used the true Earth-rate vector
            % and that one used a scalar z-rate. om_eci is passed through so the
            % co-rotating v_rel keeps the CIP tilt (~0.4 m/s).
            oc  = struct('mass',sc.mass,'wind',[0;0;0], ...
                         'omega', getf(ctx,'omega_eci',[0;0;omega]));
            if ~getf(d,'corotate',true), oc.omega = [0;0;0]; end
            out = drag.cannonball(ctx.r_eci, ctx.v_eci, rho, Cd, sc.Aref, oc);
            a = out.a;
            info.out = out;
        otherwise                                   % full free-molecular panel model
            gsi  = getf(d,'gsi',struct('Tw',300,'aT',0.9,'sig_n',0.9,'sig_t',0.9));
            % facets: use the spacecraft geometry if defined, else synthesise a
            % single flat plate of area Aref facing +x (ram) so mass/area/Cd-only
            % spacecraft (e.g. a simple 16U) still run the panel models.
            facets = getf(sc,'facets',[]);
            if isempty(facets)
                facets = struct('n',[1;0;0],'A',sc.Aref);
            end
            % OMEGA. This passed the scalar constant `omega` UNCONDITIONALLY, which
            % had two consequences and neither announced itself:
            %   1. forces.drag.corotate = false did NOTHING to the panel models. The
            %      knob is documented, sweepable, and was silently ignored -- a flat
            %      column in a sweep table that looks like a physics result.
            %   2. The panel models used the naive z-axis rate while the cannonball
            %      branch used the true Earth rate about the CIP (~168 arcsec off z,
            %      ~0.4 m/s of v_rel). The same model, two co-rotation conventions,
            %      depending on which branch you took.
            % drag.force now accepts a vector omega, so hand it the same one the
            % cannonball gets.
            om_drag = getf(ctx,'omega_eci',[0;0;omega]);
            if ~getf(d,'corotate',true), om_drag = [0;0;0]; end
            opts = struct('mass',sc.mass,'Aref',sc.Aref, ...
                          'R_bi',getf(sc,'R_bi',eye(3)), 'wind',[0;0;0], 'omega',om_drag);
            % Tracking solar arrays need the Sun to know where they are pointing.
            % ctx.E is only populated when a force that needs the ephemeris is on
            % (see op.accel's needE list) -- if it is absent and the geometry has
            % arrays, drag.force errors rather than inventing a panel orientation.
            if isfield(ctx,'E') && ~isempty(ctx.E) && isfield(ctx.E,'sun_eci')
                opts.sunHat_eci = ctx.E.sun_eci;
            end
            out  = drag.force(ctx.r_eci, ctx.v_eci, atm, facets, ...
                              lower(d.model), gsi, opts);
            a = out.a;
            info.out = out;
    end
end
function val=getf(s,f,dv), if isfield(s,f)&&~isempty(s.(f)), val=s.(f); else, val=dv; end, end
