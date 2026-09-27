function sats = fetch_tle(group)
%VALIDATION.FETCH_TLE  Download current TLEs from CelesTrak (needs internet).
%   sats = validation.fetch_tle(group)
%     group : CelesTrak GP group, e.g. 'stations','active','starlink',
%             'last-30-days', or a NORAD id as 'CATNR=25544'.
%   Returns a struct array with .name .l1 .l2 (raw TLE lines) ready for
%   validation.parseTLE.  Uses webread/urlread; if offline, load a local file
%   with validation.read_tle_file instead.
    base = 'https://celestrak.org/NORAD/elements/gp.php?';
    if ~isempty(strfind(upper(group),'CATNR'))
        url = [base upper(group) '&FORMAT=TLE'];
    else
        url = [base 'GROUP=' group '&FORMAT=TLE'];
    end
    try
        txt = webread(url);
    catch
        txt = urlread(url); %#ok<URLRD>
    end
    sats = splitTLE(txt);
end

function sats = splitTLE(txt)
    lines = regexp(txt, '\r\n|\n|\r', 'split');
    lines = lines(~cellfun(@isempty, strtrim(lines)));
    sats = struct('name',{},'l1',{},'l2',{});
    i=1;
    while i+2 <= numel(lines)+1
        if i+1>numel(lines), break; end
        nm = strtrim(lines{i});
        if numel(lines)>=i+2 && lines{i+1}(1)=='1' && lines{i+2}(1)=='2'
            sats(end+1)=struct('name',nm,'l1',lines{i+1},'l2',lines{i+2}); %#ok
            i=i+3;
        elseif lines{i}(1)=='1' && lines{i+1}(1)=='2'
            sats(end+1)=struct('name','','l1',lines{i},'l2',lines{i+1}); %#ok
            i=i+2;
        else, i=i+1; end
    end
end
