function D = itsg(sat, dateStr, product, opts)
%DATA.ITSG  The single open data source: ITSG / TU Graz satellite orbit products.
%
%   D = data.itsg('CHAMP','2007-01-01','reducedDynamicOrbit')
%   D = data.itsg('GRACE-FO-1','2019-05-01','nonConservativeForces')
%
%   NO CREDENTIALS. NO FTPS. NO ARCHIVE WALKING.
%   Quoting the provider's own readme:
%       "Access is granted without any registration and free of charge."
%   Plain HTTPS + a normal Apache directory index. This is why the whole credential
%   subsystem is gone.
%
%   ---------------------------------------------------------------------------
%   PRODUCTS (all <SAT>_<product>_<YYYY-MM-DD>.txt.gz, GROOPS ASCII, gzipped)
%   ---------------------------------------------------------------------------
%     reducedDynamicOrbit       MJD, x,y,z [m], vx,vy,vz [m/s]   10 s   <- SEED FROM THIS
%     kinematicOrbit            MJD, x,y,z [m]                   measurement epochs
%     kinematicOrbitCovariance  MJD, xx,yy,zz,xy,xz,yz [m^2]     as kinematic
%     attitude                  MJD, q0,qx,qy,qz                 10 s  (sat -> celestial)
%     nonConservativeForces     MJD, ax,ay,az [m/s^2]            10 s  (SATELLITE frame)
%     neutralDensity            MJD, lon,lat,alt,LST,rho   (r1.5/2.0)  5-60 s
%     neutralDensity_ACC        MJD, rho [kg/m^3]          (r1.0)      60 s
%
%   ***  POSITIONS AND VELOCITIES ARE IN THE CELESTIAL (INERTIAL) FRAME.  ***
%   The readme states it for both orbit products. So there is NO ITRF->ECI rotation
%   on this path -- which removes the entire bug class that dominated this project
%   (CIP/omega seed error, 'gmst' on a real ITRF track, dCt/dt velocity conversion).
%   Read -> seed -> propagate -> compare, all in one frame.
%
%   Column 1 is MJD in EVERY product, so all products join on one key. That is what
%   makes orbit/attitude/accelerometer/density comparison trivial.
%
%   RETURNS a struct:
%     D.mjd      [N x 1]  epoch (MJD, days)
%     D.data     [N x M]  the remaining columns, units exactly as above
%     D.r, D.v            [N x 3] for orbit products (m, m/s), celestial frame
%     D.q                 [N x 4] for attitude (q0,qx,qy,qz)
%     D.a_sat             [N x 3] for nonConservativeForces (m/s^2, satellite frame)
%     D.rho               [N x 1] for neutralDensity
%     D.source   struct with .url .retrieved .product .satellite .frame .sampling_s
%                -> PROVENANCE. Every run can say exactly where its numbers came from.
%
%   opts: .force (re-download), .root (cache dir), .quiet
    % ---- test entry points -------------------------------------------------
    % These let the product table and the URL builder be checked in one line instead
    % of only through a live request. The version-stripping rule rejected a name the
    % catalog had read off the server, and nothing could catch that offline: the only
    % way to exercise the rule was to watch a real fetch 404. A rule that decides
    % whether data is reachable must be testable without the network.
    if strcmp(sat, '__productinfo__'), D = productInfo(product); return, end
    if strcmp(sat, '__urls__')
        D = itsgURLs(product{1}, product{2}, product{3}, dateStr); return
    end

    if nargin<4, opts = struct(); end
    S = itsgName(sat);
    P = productInfo(product);

    cacheDir = fullfile(data.root(), 'itsg', S);
    if ~exist(cacheDir,'dir'), mkdir(cacheDir); end
    fname = sprintf('%s_%s_%s.txt.gz', S, P.name, dateStr);   % stem: no version
    local = fullfile(cacheDir, fname);

    % ---- fetch (cache-first: these products are IMMUTABLE, so never re-download) --
    force = isfield(opts,'force') && ~isempty(opts.force) && opts.force;
    url = '';
    if exist(local,'file') && ~force
        if ~isfield(opts,'quiet'), fprintf('[itsg] cache: %s\n', fname); end
        url = '(cache)';
    else
        % ---- DOWNLOAD TO A TEMP FILE, THEN RENAME ---------------------------
        % websave() used to write STRAIGHT INTO the cache path. Interrupt the
        % download -- network drop, Ctrl-C, a full disk -- and a PARTIAL file is
        % left sitting at `local`, where the cache-first check above will find it
        % and treat it as a complete product FOREVER. The delete in the catch only
        % fires if websave itself throws; a killed process never reaches it.
        %
        % That failure is nasty precisely because it is silent and sticky: every
        % later run reads the truncated file, gunzip dies with something cryptic
        % about an unexpected end of stream, and nothing points at the cache. The
        % fix is the standard one -- download to a temp name and RENAME only on
        % success, because rename is atomic and a half-written temp file is
        % obviously not the cache.
        %
        % Also verify it IS a gzip (magic bytes 1f 8b). Some servers answer a
        % missing file with 200 and an HTML error page, which websave stores
        % happily -- and then we would cache the word "404" under a .txt.gz name.
        cand = itsgURLs(S, P.dir, P.name, dateStr);
        ok = false;
        tmpf = [local '.part'];
        for i = 1:numel(cand)
            try
                if ~isfield(opts,'quiet')
                    fprintf('[itsg] GET %d/%d %s\n', i, numel(cand), cand{i});
                end
                if exist(tmpf,'file'), delete(tmpf); end
                websave(tmpf, cand{i});
                assertGzip(tmpf, cand{i});
                movefile(tmpf, local);          % atomic: now it is cached
                url = cand{i}; ok = true; break
            catch
                if exist(tmpf,'file'), delete(tmpf); end
            end
        end
        if ~ok
            % Directory index is plain HTML here, so listing is cheap and reliable.
            u = itsgFind(S, P.dir, P.name, dateStr);
            if isempty(u)
                % We guessed a filename and it was not there. That does NOT mean the
                % data is absent -- it means our GUESS was wrong, and the two are
                % completely different conclusions. So list what the server actually
                % HAS before claiming anything: the product directory names, and the
                % first few real filenames. "neutralDensity_ACC not found" sent us
                % looking for missing data when the product was simply called
                % something else.
                avail = itsgListing(S, dateStr);
                % Does the CATALOG claim this date is covered? If it does and the
                % server disagrees, the catalog is wrong -- and that is a different
                % fix from "pick another date". The coverage_note column has been
                % sitting there unread since it was written; nobody ever checked it
                % against the server, which is how "2002-2017" survives next to a
                % 404 on 2012.
                % `sat`, NOT `S`. S is the DIRECTORY ('GRACE-1'); the catalog is
                % keyed by NAME ('GRACE-A'). I looked up the wrong one and wrapped it
                % in a try/catch, so it failed silently and printed nothing -- the
                % exact swallowing pattern this audit has been removing, added by me
                % three edits ago while writing about swallowing catches. The catch
                % stays (a diagnostic must not throw while explaining a failure) but
                % it now SAYS it gave up.
                cov = '';
                try
                    yy_ = str2double(dateStr(1:4));
                    Cc_ = validation.itsg_catalog(sat);
                    [ya_, yb_] = validation.coverage_years(Cc_.coverage);
                    if isfinite(ya_) && yy_ >= ya_ && yy_ <= yb_
                        cov = sprintf(['  NOTE: itsg_catalog.csv CLAIMS coverage %s, so %d should be\n' ...
                               '  available and is not. The catalog note is UNVERIFIED metadata --\n' ...
                               '  nobody ever checked it against the server. Trust the 404.\n'], ...
                               Cc_.coverage, yy_);
                    elseif isfinite(ya_)
                        cov = sprintf(['  NOTE: itsg_catalog.csv says coverage is %s and you asked for\n' ...
                               '  %d -- OUTSIDE it. That is the whole explanation.\n'], Cc_.coverage, yy_);
                    end
                catch cov_err_
                    cov = sprintf('  (could not check the catalog''s coverage claim: %s)\n', ...
                                  regexprep(cov_err_.message,'\n.*',''));
                end
                avail = [avail cov];
                error('data:itsg:notfound', ...
                  ['could not find %s for %s on %s.\n' ...
                   'This means the FILENAME we built was not on the server -- not\n' ...
                   'necessarily that the data is missing.\n' ...
                   '  we looked for : %s_%s_%s.txt.gz\n' ...
                   '  under         : %s%s/\n%s' ...
                   'Products and coverage differ per satellite (e.g. CHAMP has no\n' ...
                   'data after 2010; Swarm starts 2013).'], ...
                   P.name, S, dateStr, S, P.name, dateStr, itsgBase(), S, avail);
            end
            websave(local, u); url = u;
        end
    end

    % ---- read (GROOPS ASCII, gzipped; '#' comments) ------------------------------
    tmp = tempname; mkdir(tmp);
    f = local;
    try
        g = gunzip(local, tmp);
        if iscell(g), f = g{1}; else, f = g; end
    catch
        f = local;                      % already plain text
    end
    [M, H] = readGroops(f);
    if isempty(M), error('data:itsg:empty','no numeric rows parsed from %s', fname); end

    D = struct();
    D.mjd  = M(:,1);
    D.data = M(:,2:end);
    switch P.name
        case {'reducedDynamicOrbit'}
            % REAL header: data0..data8 = pos x,y,z | vel x,y,z | acc x,y,z
            % (the product readme documents only pos+vel; the file also carries the
            %  acceleration of the reduced-dynamic solution -- keep it, it is useful
            %  as an independent cross-check on our own force model.)
            need(M,7,P.name);
            D.r = M(:,2:4); D.v = M(:,5:7);
            if size(M,2) >= 10, D.a_rdo = M(:,8:10); end
        case {'kinematicOrbit'}
            % Also declares 9 data cols; only the position block is meaningful for a
            % geometry-only solution, but read whatever is there.
            need(M,4,P.name);
            D.r = M(:,2:4);
            if size(M,2) >= 7,  D.v_kin = M(:,5:7);  end
            if size(M,2) >= 10, D.a_kin = M(:,8:10); end
        case {'kinematicOrbitCovariance'}
            need(M,7,P.name); D.cov = M(:,2:7);      % xx yy zz xy xz yz
        case 'attitude'
            need(M,5,P.name); D.q = M(:,2:5);        % q0 qx qy qz (sat -> celestial)
        case 'nonConservativeForces'
            need(M,4,P.name); D.a_sat = M(:,2:4);    % m/s^2, SATELLITE frame
        case 'neutralDensity'
            if size(M,2) >= 6
                D.lon=M(:,2); D.lat=M(:,3); D.alt=M(:,4); D.lst=M(:,5); D.rho=M(:,6);
            else
                need(M,2,P.name); D.rho = M(:,2);    % release 1.0 layout
            end
        case 'neutralDensity_ACC'
            need(M,2,P.name); D.rho = M(:,2);
    end

    % ---- PROVENANCE: every number can name its origin ----------------------------
    D.source = struct('provider','ITSG / TU Graz (Institute of Geodesy)', ...
                      'satellite',S, 'product',P.name, 'date',dateStr, ...
                      'url',url, 'file',local, 'retrieved',datestr(now,31), ...
                      'frame',P.frame, 'sampling_s',P.sampling_s, ...
                      'units',P.units, 'access','open (no registration)', ...
                      'software','GROOPS', 'nEpochs',numel(D.mjd), ...
                      'groops_type',H.type, 'ncol',H.ncol, 'labels',{H.labels});
    if ~isfield(opts,'quiet')
        fprintf('[itsg] %s %s %s: %d epochs, %s frame\n', S, P.name, dateStr, ...
                numel(D.mjd), P.frame);
    end
end

% ============================================================ helpers ==========
function b = itsgBase()
    b = 'https://ftp.tugraz.at/pub/ITSG/satelliteOrbitProducts/operational/';
end

function u = itsgURLs(S, pdir, stem, dateStr)
%ITSGURLS  Candidate locations for one product on one date.
%
%   pdir : the DIRECTORY on the server   e.g. 'neutralDensity_1.0'
%   stem : the product name in the FILE  e.g. 'neutralDensity'
%
%   These are different strings and one argument could not be both. The real layout:
%       .../operational/CHAMP/neutralDensity_1.0/2003/CHAMP_neutralDensity_2003-01-01.txt.gz
%       \_____ pdir _____/      \_ stem _/
%   The old signature took one `prod` and used it for both, so it could only ever
%   find a product whose directory and filename agreed -- which the versioned ones
%   do not. Both spellings of the FILE are still tried, cheaply, because the
%   convention is the provider's and may not be uniform across missions.
    yy = dateStr(1:4);
    b  = [itsgBase() S '/'];

    stems = {stem};
    if ~strcmp(pdir, stem), stems{end+1} = pdir; end   % in case the file DOES repeat it

    u = {};
    for i = 1:numel(stems)
        f = sprintf('%s_%s_%s.txt.gz', S, stems{i}, dateStr);
        u = [u, { [b pdir '/' yy '/' f], ...
                  [b pdir '/' f], ...
                  [b yy '/' f], ...
                  [b f] }]; %#ok<AGROW>
    end
    u = unique(u, 'stable');
end

function names = itsgProducts(S)
%ITSGPRODUCTS  What products does the server ACTUALLY have for this satellite?
%   Returns the real sub-directory names under <base>/<S>/.
%
%   This exists because we were GUESSING product names ('neutralDensity',
%   'neutralDensity_ACC') and reporting the guess's failure as a fact about the
%   data: "not published for this satellite/epoch". The catalog says
%   has_density = yes for GRACE-A; the provenance figure said NOT PUBLISHED. Both
%   cannot be right, and the one that was guessing was us.
%
%   The orbit downloads fine from .../GRACE-1/reducedDynamicOrbit/2010/. Same
%   server, same folder, same pattern. So the density is either under a directory
%   we did not think of or in a file named differently -- and the server will simply
%   TELL US if we ask, instead of us trying four spellings and giving up.
    names = {};
    try
        html = webread([itsgBase() S '/']);
        d = regexp(html, 'href="([A-Za-z_0-9\-]+)/"', 'tokens');
        for k = 1:numel(d)
            n = d{k}{1};
            if any(strcmpi(n, {'..','.'})), continue, end
            if ~isempty(regexp(n, '^(19|20)\d{2}$', 'once')), continue, end  % year dirs
            names{end+1} = n; %#ok<AGROW>
        end
        names = unique(names);
    catch
        % offline or the satellite dir is gone: the caller falls back to guessing and
        % says so. Returning {} is honest -- it means "we do not know", which is
        % different from "there are none".
    end
end

function u = itsgFind(S, pdir, stem, dateStr)
%ITSGFIND  Ask the server what is in the directory and match on the DATE.
%
%   The date is the one part of the filename we actually know. Everything else --
%   the version, the separator, whether the stem repeats -- is the provider's
%   convention, and every time we have assumed it we have been wrong and then
%   reported our wrong assumption as "the data is not published".
    u = '';
    yy = str2double(dateStr(1:4));
    files = data.itsg_files(S, pdir, yy);
    if isempty(files)
        % EMPTY MEANS TWO DIFFERENT THINGS and the caller cannot tell them apart:
        %   (a) the directory exists and holds nothing for this date
        %   (b) we could not read the directory at all
        % (a) is a fact about the data. (b) is a fact about our network. Reporting
        % (b) as (a) is the "not published for this satellite/epoch" mistake.
        fprintf(['[itsg] listing %s%s/%s/%d/ returned nothing.\n' ...
                 '       Either the date is absent OR the listing failed (network /\n' ...
                 '       webread could not read the index). Different problems --\n' ...
                 '       open that URL in a browser to tell them apart.\n'], ...
                 itsgBase(), S, pdir, yy);
        return
    end
    hit = files(~cellfun(@isempty, strfind(files, dateStr)));
    if isempty(hit)
        % some ITSG products date-stamp yyyyddd rather than yyyy-mm-dd
        try
            doy = day(datetime(dateStr,'InputFormat','yyyy-MM-dd'),'dayofyear');
            hit = files(~cellfun(@isempty, strfind(files, sprintf('%04d%03d', yy, doy))));
        catch
        end
    end
    if isempty(hit)
        fprintf(['[itsg] %s%s/%s/%d/ has %d file(s) but none match %s.\n' ...
                 '       First few: %s\n'], itsgBase(), S, pdir, yy, numel(files), ...
                 dateStr, strjoin(files(1:min(3,numel(files))), ', '));
        return
    end
    u = sprintf('%s%s/%s/%d/%s', itsgBase(), S, pdir, yy, hit{1});
    fprintf('[itsg] listing found: %s\n', hit{1});
end

function [M, H] = readGroops(f)
%READGROOPS  Read a GROOPS instrument file. VERIFIED against real ITSG files.
%
%   The real layout is NOT what the product readme implies -- it is:
%       1: groops instrument version=20200123      <- version line (no '#')
%       2: # ORBIT | STARCAMERA | ACCELEROMETER | COVARIANCE3D | MISCVALUE
%       3:        -6          1                    <- type code + count (BARE NUMBERS)
%       4: # Time [MJD]  data0: pos x [m] ... data8: acc z [m/s^2]
%       5: # ==========================================
%       6:       8640                              <- epoch count (BARE NUMBER)
%       7+: data rows
%
%   Lines 3 and 6 are BARE NUMBERS with no '#', so a naive "skip comments, sscanf
%   the rest" parser silently eats them as data rows -- injecting a fake epoch at
%   MJD -6 and another at MJD 8640. Both are rejected here by requiring the exact
%   column count declared in the header, which is the only robust discriminator.
    M = []; H = struct('type','','ncol',NaN,'labels',{{}},'nEpochs',NaN);
    fid = fopen(f,'r'); if fid<0, return, end
    c = onCleanup(@() fclose(fid));

    ncol = NaN;                       % expected columns = 1 (MJD) + #data fields
    rows = {}; n = 0;
    while true
        ln = fgetl(fid);
        if ~ischar(ln), break, end
        t = strtrim(ln);
        if isempty(t), continue, end

        if strncmpi(t,'groops',6), continue, end            % version line
        if t(1)=='#'
            % '# ORBIT' etc, and the '# Time [MJD] data0: ... dataN: ...' line
            if isempty(H.type)
                tok = regexp(t,'^#\s*([A-Z0-9_]+)\s*$','tokens','once');
                if ~isempty(tok), H.type = tok{1}; continue, end
            end
            lab = regexp(t,'data\d+:\s*([^\[]+)\[([^\]]*)\]','tokens');
            if ~isempty(lab)
                H.labels = cellfun(@(x) strtrim(x{1}), lab, 'UniformOutput', false);
                ncol = numel(lab) + 1;                       % + the MJD column
                H.ncol = ncol;
            end
            continue
        end

        v = sscanf(t,'%f').';
        if isempty(v), continue, end

        if isnan(ncol)
            % MISCVALUE files declare no 'dataN:' labels -> infer from the first row
            % that looks like a real epoch (MJD is ~4e4-7e4 for 1958..2050).
            if numel(v)>=2 && v(1)>4e4 && v(1)<7e4, ncol = numel(v); H.ncol = ncol;
            else, continue, end                              % type-code / count line
        end
        if numel(v) ~= ncol, continue, end                   % rejects '-6 1' and '8640'
        if ~(v(1) > 4e4 && v(1) < 7e4), continue, end        % rejects anything non-MJD
        n = n + 1; rows{n} = v; %#ok<AGROW>
    end
    if n==0, return, end
    M = zeros(n, ncol);
    for i = 1:n, M(i,:) = rows{i}; end
    H.nEpochs = n;
end

function need(M, k, name)
    if size(M,2) < k
        error('data:itsg:cols','%s needs %d columns, got %d', name, k, size(M,2));
    end
end

function P = productInfo(p)
%PRODUCTINFO  Column layout / frame / sampling, straight from the provider readme.
%
%   ---------------------------------------------------------------------------
%   THE DIRECTORY IS NOT THE PRODUCT
%   ---------------------------------------------------------------------------
%   The server holds the density under a VERSIONED directory:
%       .../operational/CHAMP/neutralDensity_1.0/2003/
%   but its COLUMNS, frame and sampling are the neutralDensity product's. Those are
%   two different facts and this function used to conflate them: it matched on
%   lower(strrep(p,'_','')), which turns "neutralDensity_1.0" into
%   "neutraldensity1.0" and matches nothing -- so a name the CATALOG had recorded
%   from the server was rejected by our own whitelist as an "unknown product",
%   before a single URL was tried.
%
%   That is a third place product names were defined (catalog column 16, validate_OD,
%   and here) and they disagreed. One quantity, three names -- the bug class this
%   audit has hit more than any other.
%
%   So: strip a trailing _1.0 / _2 / _1.0.3 to find the LAYOUT, and keep the string
%   the caller gave us as the DIRECTORY.
    pdir = p;
    base = regexprep(p, '_\d+(\.\d+)*$', '');
    switch lower(strrep(base,'_',''))
        case {'reduceddynamicorbit','rdo','orbit'}
            P = mk('reducedDynamicOrbit','celestial (inertial)',10, ...
                   'MJD[d], x,y,z[m], vx,vy,vz[m/s]');
        case {'kinematicorbit','kin'}
            P = mk('kinematicOrbit','celestial (inertial)',NaN, ...
                   'MJD[d], x,y,z[m]  (measurement epochs: NOT equidistant)');
        case {'kinematicorbitcovariance','cov'}
            P = mk('kinematicOrbitCovariance','celestial (inertial)',NaN, ...
                   'MJD[d], xx,yy,zz,xy,xz,yz [m^2]');
        case {'attitude','att'}
            P = mk('attitude','satellite -> celestial',10,'MJD[d], q0,qx,qy,qz');
        case {'nonconservativeforces','ncf','acc'}
            P = mk('nonConservativeForces','SATELLITE body frame',10, ...
                   'MJD[d], ax,ay,az [m/s^2]');
        case {'neutraldensity','density','rho'}
            P = mk('neutralDensity','GRS80 geodetic + density',NaN, ...
                   'MJD[d], lon[deg], lat[deg], alt[m], LST[h], rho[kg/m^3]');
        case {'neutraldensityacc'}
            P = mk('neutralDensity_ACC','density only',60,'MJD[d], rho[kg/m^3]');
        otherwise
            error('data:itsg:product', ...
                 ['unknown product "%s" (version-stripped: "%s").\n' ...
                  'This is OUR whitelist rejecting it, NOT the server.\n' ...
                  'Known layouts: reducedDynamicOrbit, kinematicOrbit, ' ...
                  'kinematicOrbitCovariance, attitude, nonConservativeForces, ' ...
                  'neutralDensity, neutralDensity_ACC.\n' ...
                  'A trailing version (_1.0) is stripped automatically, so a name ' ...
                  'reaching here is genuinely new: add its COLUMN LAYOUT to ' ...
                  'productInfo in 05_data/+data/itsg.m.'], p, base);
    end
    % The DIRECTORY as the caller gave it. P.name stays the LOGICAL product, because
    % the file inside a versioned directory does not repeat the version:
    %     .../neutralDensity_1.0/2003/CHAMP_neutralDensity_2003-01-01.txt.gz
    % One string cannot be both, and pretending it can is what broke this.
    P.dir = pdir;
end
function P = mk(n,f,s,u), P = struct('name',n,'frame',f,'sampling_s',s,'units',u); end

function S = itsgName(sat)
%ITSGNAME  Map our catalog names onto ITSG's directory names.
%   ITSG uses GRACE-1/2, GRACEFO-1/2 and Swarm-1/2/3 (= Swarm A/B/C).
    s = upper(strtrim(strrep(strrep(sat,' ',''),'_','-')));
    map = { 'CHAMP','CHAMP'; ...
            'GRACE-A','GRACE-1'; 'GRACE1','GRACE-1'; 'GRACE-1','GRACE-1'; ...
            'GRACE-B','GRACE-2'; 'GRACE2','GRACE-2'; 'GRACE-2','GRACE-2'; ...
            'GRACE-FO-1','GRACEFO-1'; 'GRACEFO-1','GRACEFO-1'; 'GRACE-FO1','GRACEFO-1'; ...
            'GRACE-FO-2','GRACEFO-2'; 'GRACEFO-2','GRACEFO-2'; 'GRACE-FO2','GRACEFO-2'; ...
            'SWARM-A','Swarm-1'; 'SWARM1','Swarm-1'; 'SWARM-1','Swarm-1'; ...
            'SWARM-B','Swarm-2'; 'SWARM2','Swarm-2'; 'SWARM-2','Swarm-2'; ...
            'SWARM-C','Swarm-3'; 'SWARM3','Swarm-3'; 'SWARM-3','Swarm-3'; ...
            'JASON-1','Jason-1'; 'JASON-2','Jason-2'; 'JASON-3','Jason-3'; ...
            'METOP-A','MetOp-A'; 'METOP-B','MetOp-B'; ...
            'SENTINEL-1A','Sentinel-1A'; 'SENTINEL-1B','Sentinel-1B'; 'SENTINEL-1C','Sentinel-1C'; ...
            'SENTINEL-2A','Sentinel-2A'; 'SENTINEL-2B','Sentinel-2B'; 'SENTINEL-2C','Sentinel-2C'; ...
            'SENTINEL-3A','Sentinel-3A'; 'SENTINEL-3B','Sentinel-3B'; 'SENTINEL-6A','Sentinel-6A'; ...
            'TERRASAR-X','TerraSAR-X'; 'TANDEM-X','TanDEM-X' };
    ix = find(strcmpi(map(:,1), s), 1);
    if isempty(ix)
        error('data:itsg:sat', ['"%s" is not an ITSG satellite. Run itsg.list to see all ' ...
              '23.\nNOTE: GOCE is NOT on ITSG -- it stays a manual ESA download.'], sat);
    end
    S = map{ix,2};
end

function txt = itsgListing(S, dateStr)
%   Two failures look identical from here and need OPPOSITE fixes:
%     - the satellite DIRECTORY is misnamed  -> the parent listing shows the truth
%     - the DATE is outside coverage          -> the year listing shows the truth
%   So try the satellite dir, and if that 404s, walk UP to the parent.
%ITSGLISTING  What does the server ACTUALLY have for this satellite?
%   Used only to build a useful error. Reporting "not found" after guessing a
%   filename is close to useless: it conflates "the data does not exist" with "we
%   named it wrong", and those need opposite responses. Listing the real directory
%   settles it in one line.
    txt = '';
    try
        try
            html = webread([itsgBase() S '/']);
        catch
            % the satellite directory itself is not there: the NAME is wrong, not
            % the date. Listing the parent says which names DO exist -- that is a
            % completely different fix from "pick another date", and an error that
            % cannot tell them apart sends you looking in the wrong place.
            hp = webread(itsgBase());
            dp = regexp(hp, 'href="([A-Za-z0-9_\-]+)/"', 'tokens');
            nm = {}; for k=1:numel(dp)
                if ~any(strcmpi(dp{k}{1},{'..','.'})), nm{end+1} = dp{k}{1}; end %#ok<AGROW>
            end
            txt = sprintf(['  the directory ''%s'' DOES NOT EXIST on the server.\n' ...
                   '  the SATELLITE NAME is wrong, not the date. Server has:\n    %s\n' ...
                   '  (this maps from itsg_dir in 05_data/sat_data/itsg_catalog.csv)\n'], ...
                   S, strjoin(unique(nm), ', '));
            return
        end
        d = regexp(html, 'href="([A-Za-z_0-9]+)/"', 'tokens');
        names = {};
        for k = 1:numel(d)
            n = d{k}{1};
            if ~any(strcmpi(n, {'..','.'})) && isempty(regexp(n,'^(19|20)\d{2}$','once'))
                names{end+1} = n; %#ok<AGROW>
            end
        end
        names = unique(names);
        if ~isempty(names)
            txt = sprintf('  server HAS these product dirs for %s:\n    %s\n', S, strjoin(names, ', '));
        end
        % and a sample of real filenames from the year folder of the first product
        if ~isempty(names)
            yy = dateStr(1:4);
            for k = 1:min(numel(names),6)
                try
                    h2 = webread([itsgBase() S '/' names{k} '/' yy '/']);
                    fm = regexp(h2, 'href="([^"]*\.txt\.gz)"', 'tokens');
                    if ~isempty(fm)
                        txt = [txt sprintf('  e.g. in %s/%s/: %s\n', names{k}, yy, fm{1}{1})];
                    end
                catch
                    % LEGITIMATE swallow: this is building an ERROR MESSAGE. A
                    % diagnostic that throws while explaining a failure replaces a
                    % useful message with a useless one.
                end
            end
        end
    catch err_
        % sprintf, NOT a single-quoted literal. MATLAB single quotes do not
        % interpret escapes, so '...\n' printed a literal backslash-n INSIDE the
        % error message -- in the very function whose job was to make the error
        % clearer. Round 26 added this and shipped it unrendered because the path
        % only runs when the listing fails, which is exactly when nobody is looking.
        txt = sprintf('  (could not list the server directory: %s)\n', ...
                      regexprep(err_.message, '\n.*', ''));
    end
end

function assertGzip(f, url)
%ASSERTGZIP  Is this actually a gzip? Magic bytes 1f 8b.
%   Some servers answer a missing file with HTTP 200 and an HTML error page.
%   websave stores that happily, and we would cache the word "404" under a .txt.gz
%   name -- then fail at gunzip, on every later run, with a message about the file
%   format that points nowhere near the real problem. Check it once, here, while we
%   still know which URL it came from.
    d = dir(f);
    if isempty(d) || d.bytes < 20
        error('data:itsg:tinyFile', 'downloaded %d bytes from %s -- not a product file', ...
              subsref_tern(isempty(d),0,d.bytes), url);
    end
    fid = fopen(f,'rb');
    if fid < 0, error('data:itsg:unreadable','cannot read %s', f); end
    magic = fread(fid, 2, 'uint8').';
    fclose(fid);
    if ~isequal(magic, [31 139])
        error('data:itsg:notGzip', ...
          ['%s returned %d bytes that are NOT gzip (magic %02x %02x, expected 1f 8b).\n' ...
           'That is usually an HTML error page served with HTTP 200. NOT cached.'], ...
           url, d.bytes, magic(1), magic(2));
    end
end
