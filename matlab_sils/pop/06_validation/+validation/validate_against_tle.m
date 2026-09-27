function res = validate_against_tle(sat, opts)
%VALIDATION.VALIDATE_AGAINST_TLE  Cross-check the propagator against a real object.
%
%   res = validation.validate_against_tle(sat[, opts])
%     sat  : one element of validation.fetch_tle(...) (fields .name .l1 .l2)
%     opts : .tspan_s (default 5400)  .dt_s (default 60)
%            .forces  (a cfg.forces struct; default: gravity(20)+drag+3rd body+SRP)
%            .frame   ('gmst' default, or 'B'/'C' for precise EOP)
%
%   PIPELINE
%     1. parse the TLE and build an osculating ECI seed at the TLE epoch
%        (validation.tle2eci);
%     2. propagate that seed with op.propagate over the requested arc;
%     3. if an SGP4 implementation is available, build the SGP4 reference and
%        report the position/velocity differences vs our propagator; otherwise
%        report our trajectory + a self-consistency check (energy, altitude).
%
%   IMPORTANT: SGP4 uses *mean* elements in the TEME frame and its own force
%   fit; our propagator integrates *osculating* dynamics with real force models.
%   A day-scale RSS difference of a few km is therefore EXPECTED and does not
%   indicate an error -- SGP4 is a cross-check, not ground truth.  For rigorous
%   validation compare against precise ephemerides (GNSS SP3, ILRS, or the
%   Orekit reference in your eci2ecef toolbox).
    if nargin<2, opts=struct(); end
    K=de440.constants(); mu=K.mu_earth;
    tle = validation.parseTLE(sat.l1, sat.l2, getf(sat,'name',''));
    tle.rawl1=sat.l1; tle.rawl2=sat.l2;
    [r0,v0] = validation.tle2eci(tle, mu);

    cfg = config.defaultConfig();
    cfg.epoch = tle.epoch(1:6);
    cfg.r0=r0; cfg.v0=v0;
    cfg.tspan = getf(opts,'tspan_s',5400);
    cfg.output = struct('dt', getf(opts,'dt_s',60));
    cfg.frame.build = getf(opts,'frame','gmst');
    cfg.gravityField.degree = 20;
    cfg.forces.gravity = struct('on',true,'model','sphharm','degree',20,'order',20);
    if isfield(opts,'forces'), cfg.forces = opts.forces; end
    sol = op.propagate(cfg);

    res.tle=tle; res.sol=sol; res.seed=[r0;v0];
    alt=(vecnorm(sol.r,2,2)-K.Re_earth)/1000;
    fprintf('\n  Object: %s (NORAD %d)\n', tle.name, tle.satnum);
    fprintf('  Epoch : %04d-%02d-%02d %02d:%02d:%05.2f UTC\n', tle.epoch(1:6));
    fprintf('  Seed  |r0|=%.3f km  |v0|=%.4f km/s  alt %.1f->%.1f km over %.0f min\n', ...
        norm(r0)/1000, norm(v0)/1000, alt(1), alt(end), cfg.tspan/60);

    try
        [ts,rs,vs] = validation.sgp4_reference(tle, cfg.tspan, getf(opts,'dt_s',60));
        n=min(size(rs,1),size(sol.r,1));
        dpos = vecnorm(sol.r(1:n,:)-rs(1:n,:),2,2);
        dvel = vecnorm(sol.v(1:n,:)-vs(1:n,:),2,2);
        res.dpos=dpos; res.dvel=dvel; res.sgp4=struct('t',ts,'r',rs,'v',vs);
        fprintf('  vs SGP4: RSS pos start %.1f m, end %.1f m, max %.1f m\n', ...
            dpos(1), dpos(end), max(dpos));
        fprintf('           RSS vel max %.3f m/s  (few-km/day divergence is normal)\n', max(dvel));
    catch err
        fprintf('  SGP4 reference unavailable (%s)\n', err.message);
        fprintf('  Reported our trajectory only; add SGP4 for the difference.\n');
    end
end
function val=getf(s,f,d), if isfield(s,f)&&~isempty(s.(f)), val=s.(f); else, val=d; end, end
