function T = list_available_satellites(group)
%VALIDATION.LIST_AVAILABLE_SATELLITES  Fetch a group and tabulate the members.
%   T = validation.list_available_satellites('stations')
%   Prints name, NORAD id, epoch, altitude class and returns a table/struct so
%   you can pick which objects to validate against.
    if nargin<1, group='stations'; end
    sats = validation.fetch_tle(group);
    K=de440.constants(); mu=K.mu_earth; Re=K.Re_earth;
    fprintf('\n  %-24s %-7s  %-19s  %-8s\n','name','NORAD','epoch (UTC)','alt [km]');
    fprintf('  %s\n', repmat('-',1,66));
    name={};num=[];alt=[];
    for k=1:numel(sats)
        tle=validation.parseTLE(sats(k).l1,sats(k).l2,sats(k).name);
        n=tle.n*2*pi/86400; a=(mu/n^2)^(1/3); h=(a-Re)/1000;
        fprintf('  %-24s %-7d  %04d-%02d-%02d %02d:%02d:%02.0f  %8.1f\n', ...
            tle.name, tle.satnum, tle.epoch(1:6), h);
        name{end+1}=tle.name; num(end+1)=tle.satnum; alt(end+1)=h; %#ok
    end
    T.name=name; T.satnum=num; T.alt_km=alt; T.sats=sats;
end
