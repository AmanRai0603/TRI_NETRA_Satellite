function C = itsg_catalog(name)
%VALIDATION.ITSG_CATALOG  One row of 05_data/sat_data/itsg_catalog.csv, as a struct.
%   C = validation.itsg_catalog('CHAMP')
%
%   OUTPUT  C.name .dir .norad .alt .mass .area .Cd .Cr .has_acc .has_density
%           .has_tudelft (logical) .tudelft_name
%
%   NAMING: the reference area is .Aref, matching cfg.spacecraft and every force.
%   It used to be .area here and .Aref_m2 in sweep_knob -- three names for one
%   quantity, held together by a translation line in validate_OD. See sat.spacecraft.
%
%   The CSV is the single source of mass/area/Cd/Cr/NORAD for every ITSG satellite.
%   `itsg.list` prints the same table with coverage. There is no other catalog: the
%   old sat_catalog is retired.
    f = fullfile(fileparts(fileparts(fileparts(mfilename('fullpath')))), ...
                 '05_data','sat_data','itsg_catalog.csv');
    if ~exist(f,'file'), f = which('itsg_catalog.csv'); end
    if isempty(f) || ~exist(f,'file')
        error('validation:itsg_catalog:nofile','itsg_catalog.csv not found -- run setup_paths.');
    end
    txt = fileread(f);
    L = regexp(txt,'\r\n|\n|\r','split');
    L = L(~cellfun(@isempty, strtrim(L)));
    for i = 2:numel(L)
    % CollapseDelimiters=false is NOT optional here. strsplit's DEFAULT merges
    % consecutive delimiters, so an empty CSV field (",,") silently DISAPPEARS and
    % every later column shifts left by one. That is exactly what happened when
    % has_tudelft/tudelft_name were added: rows with no TU Delft name (",,") came
    % back with tudelft_name = the coverage string and coverage = ''. Rows that
    % happened to have every field populated parsed fine, which is what makes this
    % kind of bug survive -- it only bites the sparse rows.
        c = strsplit(L{i}, ',', 'CollapseDelimiters', false);
        if numel(c) >= 12 && strcmpi(strtrim(c{1}), strtrim(name))
            C = struct('name',strtrim(c{1}), 'dir',strtrim(c{2}), ...
                       'norad',str2double(c{3}), 'alt',str2double(c{4}), ...
                       'mass',str2double(c{5}),  'Aref',str2double(c{6}), ...   % canonical: was '.area'
                       'Cd',str2double(c{7}),    'Cr',str2double(c{8}), ...
                       'has_acc',strtrim(c{9}),  'has_density',strtrim(c{10}), ...
                       'has_tudelft',logical(strcmpi(strtrim(c{11}),'yes')), ...
                       'tudelft_name',strtrim(c{12}), ...
                       'has_geometry',false, 'geom_dims',[], ...
                       ... % coverage_note is COLUMN 15 of the CSV and this reader
                       ... % dropped it on the floor -- which is precisely why nothing
                       ... % ever checked coverage before spending four HTTP round
                       ... % trips to discover a 404. The data was sitting in the file
                       ... % the whole time. A parser that silently discards a column
                       ... % is a quieter version of a swallowing catch: the
                       ... % information existed, and the code decided you did not
                       ... % need it without telling anyone.
                       'coverage', '', ...
                       ... % density_product is column 16. The REAL name on the server
                       ... % is 'neutralDensity_1.0' -- a versioned directory. The code
                       ... % guessed 'neutralDensity' / 'neutralDensity_ACC', got 404s,
                       ... % and reported "not published for this satellite/epoch": a
                       ... % claim about the world built from a failure of our own
                       ... % spelling. A product name is DATA about the server, so it
                       ... % lives in the catalog next to every other server fact --
                       ... % not in an if-branch in the middle of validate_OD.
                       'density_product', '');
            if numel(c) >= 15, C.coverage        = strtrim(c{15}); end
            if numel(c) >= 16, C.density_product = strtrim(c{16}); end
            % GEOMETRY, if the row carries it. Older CSVs stop at column 12, so read
            % defensively rather than erroring on a short row -- the columns are
            % additive by design.
            if numel(c) >= 14
                C.has_geometry = logical(strcmpi(strtrim(c{13}),'yes'));
                C.geom_dims    = parseDims(c{14});
            end
            if C.has_geometry && isempty(C.geom_dims)
                error('validation:itsg_catalog:dims', ...
                  ['%s is marked has_geometry=yes but geom_dims_m is empty or ' ...
                   'unparseable. Either give "Lx;Ly;Lz" in metres or set ' ...
                   'has_geometry=no. A half-declared geometry is worse than none.'], C.name);
            end
            return
        end
    end
    error('validation:itsg_catalog:sat','"%s" is not in itsg_catalog.csv. Run itsg.list.', name);
end

function d = parseDims(s)
%PARSEDIMS  'Lx;Ly;Lz' -> [Lx Ly Lz], or [] when the column is empty.
%   Empty is the honest answer for every satellite in this catalog: nothing in this
%   toolbox carries CHAMP/GRACE/Swarm box dimensions, and a box invented to make a
%   number move is fitting, not modelling.
    d = [];
    s = strtrim(s);
    if isempty(s), return, end
    v = str2double(strsplit(s,';'));
    if numel(v)==3 && all(isfinite(v)) && all(v>0), d = v; end
end

