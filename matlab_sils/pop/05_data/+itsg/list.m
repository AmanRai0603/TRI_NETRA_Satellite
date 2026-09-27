function T = list()
%ITSG.LIST  Every satellite on the ITSG server, with what each one offers.
%   itsg.list()      print the table
%   T = itsg.list()  return it
%
%   ONE source, NO login: "Access is granted without any registration and free of
%   charge." (provider readme). Everything below is plain HTTPS.
%
%   ALL satellites have: reducedDynamicOrbit (r,v ECI, 10 s), kinematicOrbit (r ECI),
%   kinematicOrbitCovariance, attitude (quaternions).
%   The ACC / DENSITY columns say which ones ALSO carry an accelerometer-based
%   non-conservative force product and a neutral density product -- those are the
%   ones that let you validate drag/SRP DIRECTLY instead of inferring it from a
%   position residual.
    f = fullfile(fileparts(fileparts(mfilename('fullpath'))),'sat_data','itsg_catalog.csv');
    R = readTable(f);
    if nargout>0, T = R; return, end
    fprintf('\n%-12s %-12s %-7s %-7s %-8s %-5s %-8s %s\n', ...
            'NAME','ITSG DIR','NORAD','ALT km','MASS kg','ACC','DENSITY','COVERAGE');
    fprintf('%s\n', repmat('-',1,96));
    for i=1:numel(R)
        fprintf('%-12s %-12s %-7s %-7s %-8s %-5s %-8s %s\n', R(i).name, R(i).itsg_dir, ...
                R(i).norad, R(i).alt_km, R(i).mass_kg, R(i).has_acc, R(i).has_density, ...
                R(i).coverage_note);
    end
    fprintf(['\nEvery satellite has r,v (reducedDynamicOrbit, CELESTIAL frame, 10 s) +\n' ...
             'kinematicOrbit + covariance + attitude.  ACC = non-conservative force\n' ...
             '(accelerometer) also available -> lets validate_OD compare MEASURED vs\n' ...
             'MODELLED drag+SRP directly. DENSITY = neutral density also available.\n' ...
             'GOCE is NOT here (ESA only, manual download).\n' ...
             'Products: data.itsg(<name>,<date>,<product>)\n\n']);
end

function R = readTable(f)
    txt = fileread(f); L = regexp(txt,'\r\n|\n|\r','split');
    L = L(~cellfun(@isempty,strtrim(L)));
    hdr = strsplit(L{1},','); R = struct();
    for i=2:numel(L)
        c = strsplit(L{i},',');
        for k=1:min(numel(hdr),numel(c)), R(i-1).(strtrim(hdr{k})) = strtrim(c{k}); end
    end
end
