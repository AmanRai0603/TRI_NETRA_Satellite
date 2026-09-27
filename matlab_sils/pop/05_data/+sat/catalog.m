function C = catalog(name)
%SAT.CATALOG  Satellite physical parameters, read from itsg_catalog.csv.
%
%   The old sat_catalog.csv / data_availability.csv / rv_capability.csv are GONE.
%   ITSG is the only data source now, so its catalog is the only catalog: one file,
%   one truth. This shim exists so the few remaining consumers (track_reference for
%   the TU Delft overlay, compare_density, test_seed) keep working without a second
%   list that could drift out of step.
%
%   C = sat.catalog('CHAMP')  -> .name .dir .norad .alt_km .mass .Aref .Cd .Cr
%                                .has_acc .has_density .has_tudelft .tudelft_name .coverage
%   C = sat.catalog()         -> all of them
%
%   For what each satellite OFFERS (products, sampling, accelerometer, density):
%       itsg.list
    f = fullfile(fileparts(fileparts(mfilename('fullpath'))), 'sat_data', 'itsg_catalog.csv');
    txt = fileread(f);
    L = regexp(txt, '\r\n|\n|\r', 'split');
    L = L(~cellfun(@isempty, strtrim(L)));
    A = struct('name',{},'dir',{},'norad',{},'alt_km',{},'mass',{},'Aref',{}, ...
               'Cd',{},'Cr',{},'has_acc',{},'has_density',{},'has_tudelft',{},'tudelft_name',{},'coverage',{});
    for i = 2:numel(L)
    % CollapseDelimiters=false is NOT optional here. strsplit's DEFAULT merges
    % consecutive delimiters, so an empty CSV field (",,") silently DISAPPEARS and
    % every later column shifts left by one. That is exactly what happened when
    % has_tudelft/tudelft_name were added: rows with no TU Delft name (",,") came
    % back with tudelft_name = the coverage string and coverage = ''. Rows that
    % happened to have every field populated parsed fine, which is what makes this
    % kind of bug survive -- it only bites the sparse rows.
        c = strsplit(L{i}, ',', 'CollapseDelimiters', false);
        if numel(c) < 10, continue, end
        A(end+1) = struct('name',strtrim(c{1}), 'dir',strtrim(c{2}), ...
                          'norad',str2double(c{3}), 'alt_km',str2double(c{4}), ...
                          'mass',str2double(c{5}),  'Aref',str2double(c{6}), ...
                          'Cd',str2double(c{7}),    'Cr',str2double(c{8}), ...
                          'has_acc',strtrim(c{9}),  'has_density',strtrim(c{10}), ...
                          'has_tudelft',logical(strcmpi(strtrim(c{11}),'yes')), ...
                          'tudelft_name',strtrim(c{12}), ...
                          'coverage',strtrim(strjoin(c(13:end),','))); %#ok<AGROW>   % 13: after has_tudelft/tudelft_name
    end
    if nargin==0, C = A; return, end
    s = normalise(name);
    ix = find(strcmpi({A.name}, s), 1);
    if isempty(ix)
        error('sat:catalog:unknown', ['"%s" is not an ITSG satellite. Run itsg.list ' ...
              'for all %d.\nNOTE: GOCE is NOT on ITSG (ESA only, manual download).'], ...
              name, numel(A));
    end
    C = A(ix);
end

function s = normalise(n)
%NORMALISE  Accept the spellings people actually type.
    s = upper(strtrim(n));
    s = strrep(strrep(s,' ',''),'_','-');
    map = {'GRACE1','GRACE-A'; 'GRACE-1','GRACE-A'; 'GRACE2','GRACE-B'; 'GRACE-2','GRACE-B'; ...
           'GRACEFO-1','GRACE-FO-1'; 'GRACEFO1','GRACE-FO-1'; ...
           'GRACEFO-2','GRACE-FO-2'; 'GRACEFO2','GRACE-FO-2'; ...
           'SWARM1','SWARM-A'; 'SWARM-1','SWARM-A'; 'SWARMA','SWARM-A'; ...
           'SWARM2','SWARM-B'; 'SWARM-2','SWARM-B'; 'SWARMB','SWARM-B'; ...
           'SWARM3','SWARM-C'; 'SWARM-3','SWARM-C'; 'SWARMC','SWARM-C'};
    ix = find(strcmpi(map(:,1), s), 1);
    if ~isempty(ix), s = map{ix,2}; end
    if strcmp(s,'SWARM')
        error('sat:catalog:ambiguous', ...
              '"SWARM" is ambiguous: A, B and C are different orbits. Use SWARM-A/B/C.');
    end
end
