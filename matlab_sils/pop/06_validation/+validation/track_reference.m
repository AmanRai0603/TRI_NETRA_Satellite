function ref = track_reference(satName, startDate, endDate, opts)
%VALIDATION.TRACK_REFERENCE  Measured ECI r,v from TU Delft GPS-derived track.
%   ref = validation.track_reference(sat, startDate, endDate, opts)
%
%   NO-LOGIN measured-orbit reference. TU Delft publishes the GPS-derived
%   along-track GEODETIC position (altitude, latitude, longitude at 10 s) that
%   underpins their thermosphere density product for GOCE/GRACE/CHAMP/Swarm.
%   We reconstruct the measured ECI orbit from it:
%     geodetic(alt,lat,lon) --WGS84--> ECEF r  --frame--> ECI r
%     ECI v <- 4th-order finite difference of ECI r (validation.fd_velocity).
%   Position is at the few-metre level (the track is derived from a 2 cm precise
%   orbit but reported as geodetic); the finite-difference velocity is adequate
%   to seed a propagation. For cm-level truth use an SP3 (refSource='sp3').
%
%   opts.thin (default 30) keeps every Nth 10 s sample (=> 300 s spacing) so the
%   comparison epochs are manageable; opts.build ('gmst'|'B'|'C') sets the frame.
%
%   Returns ref.epoch(1x6) ref.t(Nx1 s from epoch) ref.r ref.v (Nx3 ECI, m, m/s)
%   ref.source.  Runs in real MATLAB (data.tudelft_density uses timetables).
    if nargin<4, opts=struct(); end
    % Default to the FULL IAU chain: this function converts a REAL ITRF geodetic
    % track, and 'gmst' would ignore precession/nutation (~208 arcsec of CIP offset
    % by 2010 -> ~6.7 km at LEO radius, rotating with the orbit). 'gmst' remains
    % available for synthetic/offline work, where it is self-consistent.
    build = getf(opts,'build','C');
    if isfield(opts,'frame') && ischar(opts.frame), build = opts.frame; end
    if strcmpi(build,'gmst')
        warning('validation:track_reference:gmst', ...
          ['using frame ''gmst'' on a REAL ITRF track: precession/nutation are ' ...
           'ignored, which puts the reference km off and inflates the residual. ' ...
           'Use ''C'' unless this is synthetic data.']);
    end
    thin  = getf(opts,'thin',30);

    cat = sat.catalog(satName);
    tdName = cat.tudelft_name; if isempty(tdName), tdName = satName; end
    assert(cat.has_tudelft, sprintf('%s has no TU Delft track; use refSource=sp3', satName));

    T = data.tudelft_density(tdName, startDate, endDate, opts);   % table (DateTime,...)
    % pull columns by name (read_tudelft_density_file names them explicitly)
    tvec = getcol(T,{'DateTime','Time','Timestamp'});     % UTC datetime (FULL 10 s res)
    alt  = getcol(T,{'Altitude_m','altitude','alt','Height_m'});
    lat  = getcol(T,{'Latitude_deg','latitude','lat'});
    lon  = getcol(T,{'Longitude_deg','longitude','lon'});
    nF = numel(tvec);

    % epoch = first sample
    dv0 = datevec(tvec(1)); ref.epoch = dv0(1:6);
    tF = seconds(tvec - tvec(1));

    % geodetic -> ECEF -> ECI at FULL resolution (Ct = C' maps ECEF->ECI)
    reciF = zeros(nF,3);
    for k=1:nF
        recef = geo2ecef(deg2rad(lat(k)), deg2rad(lon(k)), alt(k));
        utc = datevec(tvec(k)); utc = utc(1:6);
        [~, Ct] = frames.eci2ecef(utc, build);   % C: ECI->ECEF, Ct: ECEF->ECI
        reciF(k,:) = (Ct * recef).';
    end
    % Velocity at FULL 10 s resolution with a 4th-order stencil (incl. high-order
    % one-sided at the ends). Two things matter here:
    %  (1) differentiate BEFORE thinning -- differencing 5-min-thinned points gives
    %      a chord velocity ~2% low and off-tangent;
    %  (2) use high order at the FIRST point, which is the SEED -- a one-sided
    %      first-order difference there is ~44 m/s off at 10 s and would dominate
    %      the whole residual.
    % TU Delft carries POSITION ONLY, so velocity must be derived. Use the
    % noise-suppressing least-squares fit ('auto' -> 'poly' for this uniform 10 s
    % series): a bare finite difference amplifies the track's position
    % quantisation into ~0.3-3 m/s of SEED velocity error, which alone is
    % kilometres of semi-major axis.
    veciF = validation.fd_velocity(reciF, tF, getf(opts,'fdVelocity',struct()));
    fprintf('[TUD] velocity DERIVED from the track (position-only product), LSQ fit\n');

    % NOW thin to manageable comparison epochs (r,v,t all from the fine solution)
    idx = 1:thin:nF;
    ref.t = tF(idx); ref.r = reciF(idx,:); ref.v = veciF(idx,:);
    ref.source='TU Delft GPS-derived track (no login)';
end

function r = geo2ecef(lat, lon, h)
% WGS84 geodetic -> ECEF (m). lat,lon in rad, h in m.
    Kc=de440.constants(); a=Kc.Re_earth; f=Kc.f_earth; e2=f*(2-f);   % one source of truth
    N = a/sqrt(1-e2*sin(lat)^2);
    r = [ (N+h)*cos(lat)*cos(lon);
          (N+h)*cos(lat)*sin(lon);
          (N*(1-e2)+h)*sin(lat) ];
end

function v = getcol(T, names)
    vn = T.Properties.VariableNames;
    for i=1:numel(names)
        k = find(strcmpi(vn,names{i}),1);
        if ~isempty(k), v = T{:,k}; return; end
    end
    error('track_reference:col','none of columns %s found in TU Delft table', strjoin(names,','));
end
function val=getf(s,f,d), if isfield(s,f)&&~isempty(s.(f)), val=s.(f); else, val=d; end, end
