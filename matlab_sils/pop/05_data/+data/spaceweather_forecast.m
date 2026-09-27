function SW = spaceweather_forecast(startDate, endDate, opts)
%DATA.SPACEWEATHER_FORECAST  FORECAST space weather for a FUTURE epoch (no login).
%   SW = data.spaceweather_forecast(startDate, endDate[, opts])
%
%   For a future mission (e.g. a 16U launching in 2028) there is no observed
%   space weather -- you must PREDICT it. This pulls NOAA/SWPC open forecasts and
%   returns the SAME uniform struct atmos.spaceweather consumes, so a forecast run
%   is identical to a historical one except the indices are predicted.
%
%   SOURCES (open, no login, NOAA/SWPC):
%     long-range F10.7 : https://services.swpc.noaa.gov/json/solar-cycle/predicted-solar-cycle.json
%                        (monthly smoothed predicted F10.7 + high/low bands, years out)
%     near-term (<=45d): https://services.swpc.noaa.gov/text/45-day-forecast.txt
%                        (daily Ap + F10.7)
%   Geomagnetic activity is NOT predictable years ahead, so beyond the 45-day
%   window Ap/Kp fall back to a nominal value (opts.apNominal, default 15 ~ quiet-
%   to-moderate). F10.7 is the dominant VLEO-drag driver and IS forecast.
%
%   opts.band : 'predicted' (default) | 'high' | 'low'  -- which F10.7 curve to
%               use, so you can BRACKET decay for worst/best-case solar activity.
%   opts.apNominal : assumed daily Ap beyond the 45-day forecast (default 15).
%   opts.f107 / opts.ap : hard override (skip fetch entirely).
%
%   Returns SW.mjd f107obs f107c81 kp apDaily ap(Nx8) source. Runs in real MATLAB.
    if nargin<3, opts=struct(); end
    band = getf(opts,'band','predicted');
    apNom = getf(opts,'apNominal',15);

    d0 = datenum(startDate); d1 = datenum(endDate);
    days = (floor(d0):ceil(d1)).';
    N = numel(days);
    SW.mjd = days - 678942;

    % ---- hard override path ----
    if isfield(opts,'f107') && ~isempty(opts.f107)
        f = opts.f107;
        SW.f107obs = repmat(f,N,1); SW.f107c81 = repmat(f,N,1);
        ap = getf(opts,'ap',apNom);
        SW.apDaily = repmat(ap,N,1); SW.ap = repmat(ap,N,8);
        SW.kp = repmat(ap2kp(ap),N,1); SW.source='manual forecast override';
        return
    end

    cacheDir = fullfile(data.root(),'spaceweather'); if ~exist(cacheDir,'dir'), mkdir(cacheDir); end

    % ---- long-range monthly F10.7 from NOAA predicted-solar-cycle.json ----
    jf = data.ensure('spaceweather','predicted-solar-cycle.json', ...
        {'https://services.swpc.noaa.gov/json/solar-cycle/predicted-solar-cycle.json'}, 7, opts);
    P = readJson(jf);                       % array of structs with time-tag + f10.7 fields
    [mt, mf_pred, mf_hi, mf_lo] = parsePredicted(P);
    switch lower(band)
        case 'high', mf = mf_hi; case 'low', mf = mf_lo; otherwise, mf = mf_pred;
    end
    % monthly -> daily by linear interpolation on datenum
    f107daily = interp1(mt, mf, days, 'linear', 'extrap');
    SW.f107obs = f107daily;
    SW.f107c81 = f107daily;                 % monthly smoothed already ~ 81-day mean

    % ---- near-term (<=45 days from now) refine F10.7 + real Ap ----
    apDaily = repmat(apNom, N, 1);
    try
        if d0 <= now + 45
            F = fetch45day(cacheDir, opts);         % struct .date .f107 .ap (daily)
            for k=1:N
                j = find(F.date==days(k),1);
                if ~isempty(j)
                    SW.f107obs(k)=F.f107(j); SW.f107c81(k)=F.f107(j);
                    apDaily(k)=F.ap(j);
                end
            end
        end
    catch
        % keep nominal Ap / monthly F10.7 if the 45-day file is unavailable
    end
    SW.apDaily = apDaily; SW.ap = repmat(apDaily,1,8);
    SW.kp = arrayfun(@ap2kp, apDaily);
    SW.source = sprintf('NOAA/SWPC forecast (F10.7 %s band, Ap nominal=%g)', band, apNom);
end

% ---- helpers ----
function [mt,fp,fh,fl] = parsePredicted(P)
% P: struct array with fields like 'time-tag' ('YYYY-MM') and predicted/high/low F10.7.
    n=numel(P); mt=zeros(n,1); fp=zeros(n,1); fh=zeros(n,1); fl=zeros(n,1);
    for i=1:n
        tt = getfld(P(i),{'time_tag','time-tag'});
        mt(i) = datenum([tt '-15'],'yyyy-mm-dd');           % mid-month
        fp(i) = num(getfld(P(i),{'predicted_f10_7','predicted_f107','f10_7'}));
        fh(i) = num(getfld(P(i),{'high_f10_7','high_f107'}, fp(i)));
        fl(i) = num(getfld(P(i),{'low_f10_7','low_f107'},  fp(i)));
    end
    [mt,ix]=sort(mt); fp=fp(ix); fh=fh(ix); fl=fl(ix);
end
function F = fetch45day(cacheDir, opts)
    p = data.ensure('spaceweather','45-day-forecast.txt', ...
        {'https://services.swpc.noaa.gov/text/45-day-forecast.txt'}, 1, opts);
    txt = fileread(p); lines = regexp(txt,'\r\n|\n','split');
    D=[]; Fx=[]; A=[];
    for i=1:numel(lines)
        % lines look like: "12 Aug 2026   150   12" (date, F10.7, Ap) -- tolerant parse
        t = regexp(strtrim(lines{i}),'^(\d{1,2})\s+(\w{3})\s+(\d{4})\s+(\d+)\s+(\d+)','tokens','once');
        if ~isempty(t)
            dn = datenum([t{1} ' ' t{2} ' ' t{3}],'dd mmm yyyy');
            D(end+1,1)=dn; Fx(end+1,1)=str2double(t{4}); A(end+1,1)=str2double(t{5}); %#ok
        end
    end
    F.date=D; F.f107=Fx; F.ap=A;
end
function P = readJson(f)
    txt = fileread(f);
    if exist('jsondecode','builtin') || exist('jsondecode','file'), P = jsondecode(txt);
    else, error('spaceweather_forecast:json','jsondecode unavailable; run in MATLAB'); end
    if isstruct(P) && numel(P)==1 && isfield(P,'x'), P=P.x; end
end
function v = getfld(s, names, dflt)
    if nargin<3, dflt=''; end
    fn=fieldnames(s);
    for i=1:numel(names)
        nm = strrep(names{i},'-','_');
        k=find(strcmpi(fn,nm),1); if ~isempty(k), v=s.(fn{k}); return; end
    end
    v=dflt;
end
function x=num(v), if ischar(v), x=str2double(v); else, x=double(v); end, end
function kp=ap2kp(ap)
% coarse ap->Kp inverse (standard scale), for models that want Kp
    apv=[0 2 3 4 5 6 7 9 12 15 18 22 27 32 39 48 56 67 80 94 111 132 154 179 207 236 300 400];
    kpv=(0:27)/3;   % Kp 0,0.33,...,9
    kp = interp1(apv, kpv, min(ap,400), 'linear', 'extrap');
end
function val=getf(s,f,d), if isfield(s,f)&&~isempty(s.(f)), val=s.(f); else, val=d; end, end
