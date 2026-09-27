function SW = spaceweather(startDate, endDate, opts)
%DATA.SPACEWEATHER  Build a uniform space-weather index table from OPEN sources.
%   SW = data.spaceweather(startDate, endDate[, opts])
%
%   Uses the observatory-grade OPEN feeds you already validated (no login):
%     * F10.7 (+81-day centred mean)  <- NASA/SPDF OMNI2      via get_f107
%     * Kp, ap, Ap geomagnetic        <- GFZ Potsdam JSON      via get_gfz_hpo
%   and normalises them into ONE plain struct the propagator can look up per
%   epoch (atmos.spaceweather):
%     SW.mjd (Nx1) UTC MJD (daily)   SW.f107obs SW.f107c81 (=F107_bar)
%     SW.kp (Nx1 daily mean)         SW.apDaily (Nx1)       SW.ap (Nx8 3-hourly)
%     SW.source
%
%   CACHING: get_omni2 now caches the OMNI2 year files into <data.root>/spaceweather
%   (it previously downloaded ~20 MB to a tempname on EVERY call -- nothing was
%   reused, despite this header claiming otherwise). get_gfz_hpo still hits the GFZ
%   JSON service on every call: it has no cache layer. buildWorld calls this ONCE
%   per propagation, so that is one request per run, not per step -- but an N-config
%   compare_OD sweep is still N requests.
%   Runs in real MATLAB (timetables + web + datetime).
%   For JB2008 use data.jb2008_indices instead (different index set).
%
%   PSEUDOCODE
%     cache <- <data.root>/spaceweather
%     f107TT <- get_f107(startDate,endDate, cache)          % OMNI2
%     magTT  <- get_gfz_hpo(startDate,endDate,{'Kp','ap','Ap'}, cache)  % GFZ
%     for each day d in [startDate,endDate]:
%         SW.f107obs(d)=f107TT.F107(d); SW.f107c81(d)=f107TT.F107_bar(d)
%         SW.kp(d)=daily Kp; SW.apDaily(d)=Ap(d); SW.ap(d,:)=3-hourly ap
%     return SW
    if nargin<3, opts=struct(); end
    cacheDir = fullfile(data.root(),'spaceweather');
    if ~exist(cacheDir,'dir'), mkdir(cacheDir); end
    so = struct('cacheDir',cacheDir,'downloadDir',cacheDir, ...
                'force',getf(opts,'force',false),'verbose',getf(opts,'verbose',true));

    f107TT = get_f107(startDate, endDate, so);                 % OMNI2 (NASA/SPDF)
    magTT  = get_gfz_hpo(startDate, endDate, {'Kp','ap','Ap'}, so);  % GFZ Potsdam

    SW = mergeToStruct(f107TT, magTT);
    SW.source = 'OMNI2(F10.7)+GFZ(Kp/ap)';
end

function SW = mergeToStruct(f107TT, magTT)
% Convert the two UTC timetables into the uniform daily plain struct. ap (3-hourly)
% is folded to an Nx8 matrix per UTC day; Kp daily is the mean of its 8 values.
    fdays = dateshift(f107TT.Time,'start','day');
    udays = unique(dateshift(magTT.Time,'start','day'));
    N = numel(udays);
    SW.mjd=zeros(N,1); SW.f107obs=nan(N,1); SW.f107c81=nan(N,1);
    SW.kp=nan(N,1); SW.apDaily=nan(N,1); SW.ap=nan(N,8);
    hasKp = ismember('Kp', magTT.Properties.VariableNames);
    hasap = ismember('ap', magTT.Properties.VariableNames);
    hasAp = ismember('Ap', magTT.Properties.VariableNames);
    for i=1:N
        d = udays(i);
        SW.mjd(i) = datenum(d) - 678942;
        jf = find(fdays==d,1);
        if ~isempty(jf)
            SW.f107obs(i)=f107TT.F107(jf); SW.f107c81(i)=f107TT.F107_bar(jf);
        end
        sel = dateshift(magTT.Time,'start','day')==d;
        if hasKp, SW.kp(i)=mean(magTT.Kp(sel),'omitnan'); end
        if hasAp
            v=magTT.Ap(sel); v=v(~isnan(v)); if ~isempty(v), SW.apDaily(i)=v(1); end
        end
        if hasap
            av = magTT.ap(sel);
            n=min(8,numel(av)); SW.ap(i,1:n)=av(1:n).';
            if isnan(SW.apDaily(i)), SW.apDaily(i)=mean(av,'omitnan'); end
        end
    end
end
function val=getf(s,f,d), if isfield(s,f)&&~isempty(s.(f)), val=s.(f); else, val=d; end, end
