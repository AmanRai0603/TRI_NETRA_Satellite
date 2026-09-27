function eph = open(bspPath)
%DE440.OPEN  Load and parse a JPL DE-series SPK (Type 2) binary kernel (.bsp).
%   eph = de440.open()           uses data/de440s.bsp shipped with this tool
%   eph = de440.open(bspPath)    loads a specific kernel
%
%   Returns a struct with the flat double view of the file and a segment map.
%   Pure base MATLAB / Octave: no toolboxes required. Result is cached.
%
%   Frame  : ICRF  (== GCRF/J2000 ECI to < 1 mas, directly usable as ECI)
%   Time   : ephemeris epochs are TDB (see de440.state)
%   Units  : native km, km/s at this level (SI wrappers: de440.sun/moon)
    persistent CACHE
    if nargin < 1 || isempty(bspPath)
        bspPath = fullfile(fileparts(fileparts(mfilename('fullpath'))), 'data', 'de440s.bsp');
    end
    if ~isempty(CACHE) && strcmp(CACHE.path, bspPath)
        eph = CACHE; return
    end
    fid = fopen(bspPath, 'r', 'ieee-le');
    assert(fid > 0, 'de440:open', 'cannot open kernel: %s', bspPath);
    bytes = fread(fid, inf, '*uint8'); fclose(fid);

    locidw = char(bytes(1:8)');
    assert(strncmp(locidw,'DAF/SPK',7), 'de440:open', 'not a DAF/SPK file: %s', bspPath);
    nd    = double(typecast(bytes(9:12),  'int32'));
    ni    = double(typecast(bytes(13:16), 'int32'));
    fward = double(typecast(bytes(77:80), 'int32'));
    locfmt = strtrim(char(bytes(89:96)'));

    D = typecast(bytes, 'double');          % flat 1-based word view (word W -> D(W))
    if strcmp(locfmt,'BIG-IEEE'); D = swapbytes(D); end

    ss = nd + floor((ni+1)/2);              % doubles per array summary
    keys = {}; seg = zeros(0,2);
    recno = fward;
    while recno ~= 0
        base = (recno-1)*128;               % record start, in words
        nsum = D(base+3);                    % NEXT,PREV,NSUM
        for s = 1:nsum
            off  = base + 3 + (s-1)*ss;
            summ = D(off+1:off+ss);
            ints = typecast(summ(nd+1:ss), 'int32');   % [tgt ctr frame type SA EA]
            tgt = double(ints(1)); ctr = double(ints(2));
            sa  = double(ints(5)); ea  = double(ints(6));
            keys{end+1} = sprintf('%d_%d', ctr, tgt);  %#ok<AGROW>
            seg(end+1,:) = [sa ea];                    %#ok<AGROW>
        end
        recno = D(base+1);                  % NEXT record
    end

    eph.path  = bspPath;
    eph.D     = D;
    eph.keys  = keys;
    eph.seg   = seg;
    eph.const = de440.constants();
    CACHE = eph;
end
