function atm = jb2008(geo, sw)
%ATMOS.JB2008  Adapter to the JB2008 wrapper (jb2008_density.m).
%
%   JB2008 does NOT run on F10.7/ap. It runs on its own index set --
%   F10, S10, M10, Y10 (each with an 81-day mean) and the Dst-driven DSTDTC --
%   distributed by Space Environment Technologies as SOLFSMY.TXT and DTCFILE.TXT.
%   That is why data.spaceweather's own header says "For JB2008 use
%   data.jb2008_indices instead (different index set)".
%
%   ---------------------------------------------------------------------------
%   THIS FUNCTION COULD NEVER WORK BEFORE.
%   ---------------------------------------------------------------------------
%   It read  idx = getf(sw,'jb_idx',[])  and passed that straight to
%   jb2008_density. But atmos.spaceweather NEVER sets jb_idx -- it only ever
%   returns F107/F107a/Kp/ap/ap3. So idx was ALWAYS [], and jb2008_density line 67
%   does datenum(idx.sol.Time) on it and dies. data.jb2008_indices -- the correct
%   fetcher -- exists and had no callers. The wire was simply never connected.
%
%   So: if the caller supplies sw.jb_idx we use it; otherwise we load the indices
%   ourselves and CACHE them, because this runs inside the integrator's RHS and
%   must not touch the disk or the network thousands of times per arc.
%
%   The lags are handled inside jb2008_density, per the JB2008 spec:
%     F10, S10 at t-24h | M10 at t-48h | Y10 at t-120h | DSTDTC hourly, no lag.
%   Do not "helpfully" lag them again here.
%
%   NOTE ON EXTRAPOLATION: jb2008_density interpolates the index tables with
%   'extrap'. Past the end of SOLFSMY.TXT/DTCFILE.TXT it will silently give you a
%   linear extrapolation rather than an error. Check the file coverage before
%   trusting a recent or future epoch.
    % The indices come from the PIPELINE: op.buildWorld resolves them for the epoch
    % window -> W.jbidx -> op.accel -> ctx.jbidx -> forces.drag -> sw.jb_idx.
    %
    % An earlier version of this fix auto-loaded them here behind a `persistent`.
    % That was wrong twice over: it does file I/O from inside the integrator's RHS,
    % and a persistent is keyed to nothing -- so sweeping DATE would silently reuse
    % the FIRST date's fetch for every later date. Driver data is date-dependent;
    % it belongs in buildWorld, resolved once per run, visible in W.drivers.
    idx = getf(sw, 'jb_idx', []);
    if isempty(idx)
        error('atmos:jb2008:noIndices', ...
          ['JB2008 has no indices. They are resolved by op.buildWorld into W.jbidx and ' ...
           'carried to sw.jb_idx -- so reaching here means the pipeline was bypassed ' ...
           '(calling atmos.provider(''jb2008'',geo,sw) by hand with an sw that has no ' ...
           '.jb_idx). Fix: idx = data.jb2008_indices(startDate,endDate); then pass ' ...
           'sw.jb_idx = idx. JB2008 does NOT run on F10.7/ap.']);
    end
    t = datetime(geo.utc(1),geo.utc(2),geo.utc(3),geo.utc(4),geo.utc(5),geo.utc(6));
    rho = jb2008_density(t, geo.lon_deg, geo.lat_deg, geo.alt_km, idx);
    if ~isfinite(rho)
        error('atmos:jb2008:nonfinite', ...
          ['JB2008 returned a non-finite density at %s, alt %.1f km. Usually the epoch ' ...
           'is outside SOLFSMY.TXT / DTCFILE.TXT coverage.'], datestr(t), geo.alt_km);
    end
    atm.rho = rho; atm.Mmol = 16; atm.nO = rho*avog16();
    atm.T = getf(sw,'Tinf',1000);   % JB2008's core returns TEMP; this wrapper does
                                    % not surface it, so single-species drag using
                                    % atm.T is running on a 1000 K placeholder.
end
function val=getf(s,f,d), if isfield(s,f)&&~isempty(s.(f)), val=s.(f); else, val=d; end, end
function y=avog16(), K=de440.constants(); y=K.N_A*1000/16; end
