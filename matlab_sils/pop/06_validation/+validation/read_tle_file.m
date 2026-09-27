function sats = read_tle_file(fname)
%VALIDATION.READ_TLE_FILE  Read TLEs from a local text file (offline path).
%   sats = validation.read_tle_file('stations.txt')
    txt = fileread(fname);
    lines = regexp(txt, '\r\n|\n|\r', 'split');
    lines = lines(~cellfun(@isempty, strtrim(lines)));
    sats = struct('name',{},'l1',{},'l2',{}); i=1;
    while i<=numel(lines)
        if lines{i}(1)=='1'
            sats(end+1)=struct('name','','l1',lines{i},'l2',lines{i+1}); i=i+2; %#ok
        elseif i+2<=numel(lines) && lines{i+1}(1)=='1'
            sats(end+1)=struct('name',strtrim(lines{i}),'l1',lines{i+1},'l2',lines{i+2}); i=i+3; %#ok
        else, i=i+1; end
    end
end
