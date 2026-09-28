% ATMOS_DRAG  Reference vectors for the Rust port of 04_atmosphere + 02_forces/+drag.
%   Prints tests/data/atmos_drag.json (every number %.17g) from the MATLAB POP
%   functions themselves:
%     dtm_oper      dtm2020_oper_density  (random + the SILS F10.7=130/Kp=2 grid)
%     dtm_oper_atm  atmos.provider('dtm2020', geo, atmos.spaceweather(manual))
%     dtm_res       dtm2020_density       (research, F30/ap60)
%     dtm_res_atm   atmos.dtm2020_research (F30 rescaling with the decimal year)
%     bint_oe, geogm
%     jb2008        JB2008 core + the jb2008_density wrapper arithmetic (see below)
%     exponential   atmos.exponential
%     sw_manual     atmos.spaceweather(utc, struct('manual',..))
%     sw_table      atmos.spaceweather(utc, struct('table',..)) on a synthetic table
%     gsi           drag.sentman / cll / dria / sesam / speedRatio
%     cannonball    drag.cannonball
%     attitude      dgeom.ramAttitude / attitudeFromAoA, sgeom.R_lvlh, srp.arrayNormal
%     panel         drag.force (sentman/dria/sesam/cll) on three geometries
%     chain         forces.drag(ctx) end to end (cannonball/panel x dtm2020/exponential)
%   Run from matlab_sils/pop after setup_paths:
%     octave-cli --no-gui -q --eval "setup_paths; run('<abs>/refgen/atmos_drag.m')"
1;  % script file: the helper functions below must be defined before use
%% ============================ helpers =========================================
function s = jn(x)
    if isempty(x), s = 'null'; elseif isnan(x), s = 'null'; elseif isinf(x), s = sprintf('%s1e999', repmat('-', 1, x < 0)); else, s = sprintf('%.17g', x); end
end
function s = jv(v)
    v = v(:).'; p = cell(1, numel(v));
    for k = 1:numel(v), p{k} = jn(v(k)); end
    s = ['[' strjoin(p, ',') ']'];
end
function s = jm(M)   % row-major nested
    s = ['[' jv(M(1,:)) ',' jv(M(2,:)) ',' jv(M(3,:)) ']'];
end
function s = jb(b)
    if b, s = 'true'; else, s = 'false'; end
end
function wlist(fid, name, C, last)
    if nargin < 4, last = false; end
    fprintf(fid, '"%s":[\n', name);
    for k = 1:numel(C)
        if k < numel(C), fprintf(fid, '%s,\n', C{k}); else, fprintf(fid, '%s\n', C{k}); end
    end
    if last, fprintf(fid, ']\n'); else, fprintf(fid, '],\n'); end
end
function s = dtmcase(alt, lat, lon, lst, doy, f, fb, kp, o)
    s = ['{"alt":' jn(alt) ',"lat":' jn(lat) ',"lon":' jn(lon) ',"lst":' jn(lst) ',"doy":' jn(doy) ...
         ',"f":' jn(f) ',"fbar":' jn(fb) ',"kp":' jv(kp) ',"rho":' jn(o.rho_kgm3) ',"T":' jn(o.T_K) ...
         ',"Tinf":' jn(o.Tinf_K) ',"mbar":' jn(o.mbar_amu) ',"n":' jv([o.n.H o.n.He o.n.O o.n.N2 o.n.O2 o.n.N]) '}'];
end
function s = manualjson(m)
    f = fieldnames(m); p = cell(1, numel(f));
    for k = 1:numel(f), p{k} = ['"' f{k} '":' jn(m.(f{k}))]; end
    s = ['{' strjoin(p, ',') '}'];
end
function s = geojson(g)
    s = ['{"alt_km":' jn(g.alt_km) ',"lat_deg":' jn(g.lat_deg) ',"lon_deg":' jn(g.lon_deg) ',"lst_h":' jn(g.lst_h) ...
         ',"doy":' jn(g.doy) ',"utc":' jv(g.utc) '}'];
end
function u = unitv(x)
    u = x/norm(x);
end
function r = randorbit()
    rr = 6378137 + 1e3*(160 + 840*rand());
    r = rr*unitv(randn(3,1));
end
function v = randvel(r)
    h = unitv(cross(r, randn(3,1)));
    t = unitv(cross(h, r));
    vc = sqrt(3.986004418e14/norm(r));
    v = vc*(1 + 0.02*randn())*t + 30*randn(3,1);
end
function IDX = jb_parse(fsol, fdtc)
    % parse_solfsmy / parse_dtcfile of get_jb2008_indices, on datenum instead of datetime
    txt = strsplit(fileread(fsol), {"\r\n", "\n", "\r"});
    D = nan(numel(txt), 9); k = 0;
    for i = 1:numel(txt)
        L = strtrim(txt{i});
        if isempty(L) || L(1) == '#' || L(1) == '%', continue; end
        t = sscanf(strrep(L, char(9), ' '), '%f');
        if numel(t) >= 11 && t(1) > 1900 && t(1) < 2100
            k = k + 1; D(k,:) = [datenum(t(1),1,1)+t(2)-1, t(4),t(5),t(6),t(7),t(8),t(9),t(10),t(11)];
        end
    end
    D = sortrows(D(1:k,:), 1);
    IDX.solT = D(:,1); IDX.sol = D(:,2:9);
    txt = strsplit(fileread(fdtc), {"\r\n", "\n", "\r"});
    Tm = []; V = [];
    for i = 1:numel(txt)
        L = strtrim(txt{i});
        if isempty(L) || ~strncmpi(L, 'DTC', 3), continue; end
        p = sscanf(strrep(L(4:end), char(9), ' '), '%f');
        if numel(p) >= 26
            base = datenum(p(1),1,1) + p(2) - 1;
            Tm = [Tm; base + (0:23)'/24]; V = [V; p(3:26)];
        end
    end
    [IDX.dtcT, ix] = sort(Tm); IDX.dtc = V(ix);
end
function [rho, core] = jb_wrapper(utc, lon_deg, lat_deg, alt_km, IDX)
    % jb2008_density for ONE epoch (N = 1: no decimation, no log interpolation)
    dnum = datenum(utc(1), utc(2), utc(3), utc(4), utc(5), utc(6));
    mjd = dnum - 678942;
    s = IDX.sol; solT = IDX.solT;
    F10 = interp1(solT, s(:,1), dnum-1, 'linear', 'extrap');  F10B = interp1(solT, s(:,2), dnum-1, 'linear', 'extrap');
    S10 = interp1(solT, s(:,3), dnum-1, 'linear', 'extrap');  S10B = interp1(solT, s(:,4), dnum-1, 'linear', 'extrap');
    M10 = interp1(solT, s(:,5), dnum-2, 'linear', 'extrap');  M10B = interp1(solT, s(:,6), dnum-2, 'linear', 'extrap');
    Y10 = interp1(solT, s(:,7), dnum-5, 'linear', 'extrap');  Y10B = interp1(solT, s(:,8), dnum-5, 'linear', 'extrap');
    DSTDTC = interp1(IDX.dtcT, IDX.dtc, dnum, 'linear', 'extrap');
    % sun_radec / gmst_rad of jb2008_density.m
    T  = (mjd - 51544.5)/36525;
    L  = mod(280.460 + 36000.771*T, 360);
    M  = deg2rad(mod(357.528 + 35999.050*T, 360));
    lam= deg2rad(L + 1.915*sin(M) + 0.020*sin(2*M));
    ep = deg2rad(23.439 - 0.0130*T);
    raSun = mod(atan2(cos(ep).*sin(lam), cos(lam)), 2*pi);
    decSun = asin(sin(ep).*sin(lam));
    Tu    = (floor(mjd) - 51544.5)/36525;
    gmst0 = 24110.54841 + 8640184.812866*Tu + 0.093104*Tu.^2 - 6.2e-6*Tu.^3;
    frac  = mod(mjd, 1);
    gsec  = mod(gmst0 + 1.0027379093*86400*frac, 86400);
    gmst = deg2rad(gsec/240);
    raSat = mod(deg2rad(lon_deg) + gmst, 2*pi);
    latR  = deg2rad(lat_deg);
    [TEMP, RHO] = JB2008(mjd, [raSun; decSun], [raSat; latR; alt_km], F10,F10B, S10,S10B, M10,M10B, Y10,Y10B, DSTDTC);
    rho = RHO;
    core = struct('mjd', mjd, 'sun', [raSun decSun], 'sat', [raSat latR alt_km], ...
                  'ind', [F10 F10B S10 S10B M10 M10B Y10 Y10B DSTDTC], 'temp', TEMP, 'rho', RHO);
end

warning('off', 'all');
here = fileparts(mfilename('fullpath'));
outf = fullfile(here, '..', 'tests', 'data', 'atmos_drag.json');
fid = fopen(outf, 'w');
fprintf(fid, '{\n');
K = de440.constants();

%% ---------------- DTM2020 operational (dtm2020_oper_density) ----------------
ST = DTM2020_F107_coeffs_init();
rand('seed', 11);
C = {};
N = 2500;
for i = 1:N
    alt = 121 + 879*rand()^1.3;
    if rand() < 0.1, alt = 120 + 80*rand() + 0.01; end
    lat = -90 + 180*rand();  if rand() < 0.04, lat = 90*sign(rand()-0.5); end
    lon = -180 + 540*rand();
    lst = 24*rand();
    doy = 1 + floor(366*rand());  if rand() < 0.2, doy = 1 + 365*rand(); end
    f107 = 70 + 180*rand();
    f107a = min(260, max(65, f107 + 60*(rand()-0.5)));
    if rand() < 0.15
        kp = [9*rand(); 2*(rand()-0.5); 9*rand(); 2*(rand()-0.5)];
    elseif rand() < 0.2
        kp = floor(10*rand());
    else
        kp = 7*rand();
    end
    o = dtm2020_oper_density(alt, lat, lon, lst, doy, f107, f107a, kp, ST);
    C{end+1} = dtmcase(alt, lat, lon, lst, doy, f107, f107a, kp, o);
end
% the SILS in-loop configuration: F10.7 = F10.7a = 130, Kp = 2
for alt = 200:50:1000
  for lat = -90:30:90
    for lst = 0:3:21
      for doy = [1 172 266]
        o = dtm2020_oper_density(alt, lat, 77.5, lst, doy, 130, 130, 2, ST);
        C{end+1} = dtmcase(alt, lat, 77.5, lst, doy, 130, 130, 2, o);
      end
    end
  end
end
wlist(fid, 'dtm_oper', C);

%% ---------------- atmos.provider('dtm2020') with manual space weather ------
rand('seed', 12);
C = {};
for i = 1:300
    man = struct('F107', 70 + 180*rand(), 'F107a', 70 + 180*rand(), 'Kp', 7*rand(), 'ap', 50*rand());
    if rand() < 0.3, man = rmfield(man, 'Kp'); end
    if rand() < 0.3 && isfield(man, 'Kp'), man = rmfield(man, 'ap'); end
    if rand() < 0.2, man = rmfield(man, 'F107a'); end
    utc = [2020 3 15 floor(24*rand()) floor(60*rand()) 60*rand()];
    sw = atmos.spaceweather(utc, struct('manual', man));
    geo = struct('alt_km', 150 + 850*rand(), 'lat_deg', -90 + 180*rand(), 'lon_deg', -180 + 360*rand(), ...
                 'lst_h', 24*rand(), 'doy', 1 + floor(365*rand()), 'utc', utc);
    atm = atmos.provider('dtm2020', geo, sw);
    s = ['{"manual":' manualjson(man) ',"geo":' geojson(geo) ',"rho":' jn(atm.rho) ',"T":' jn(atm.T) ...
         ',"n":' jv([atm.n.H atm.n.He atm.n.O atm.n.N2 atm.n.O2 atm.n.N]) '}'];
    C{end+1} = s;
end
wlist(fid, 'dtm_oper_atm', C);

%% ---------------- DTM2020 research (dtm2020_density) -------------------------
STR = DTM2020_coeffs_init();
rand('seed', 13);
C = {};
for i = 1:2000
    alt = 121 + 879*rand()^1.3;
    lat = -90 + 180*rand();  if rand() < 0.05, lat = 90*sign(rand()-0.5); end
    lon = -180 + 540*rand();  if rand() < 0.01, lon = 291; end
    lst = 24*rand();
    doy = 1 + floor(366*rand());
    f30 = 70 + 180*rand();  f30b = min(260, max(65, f30 + 60*(rand()-0.5)));
    r = rand();
    if r < 0.3
        ap60 = 400*rand()^2;
    elseif r < 0.4
        ap60 = [0 2 3 4 5 6 7 9 12 15 18 22 27 32 39 48 56 67 80 94 111 132 154 179 207 236 265 294 324 355 388 421 456 494 534 574 617 657](1 + floor(38*rand()));
    elseif r < 0.55
        ap60 = 300 + 350*rand(10,1);   % storm: open-ended Kp >= 9 saturation branch
    else
        ap60 = 300*rand(10,1).^2;
    end
    o = dtm2020_density(alt, lat, lon, lst, doy, f30, f30b, ap60, STR);
    C{end+1} = dtmcase(alt, lat, lon, lst, doy, f30, f30b, ap60, o);
end
wlist(fid, 'dtm_res', C);

%% ---------------- atmos.dtm2020_research (F30 rescale) -----------------------
rand('seed', 14);
C = {};
for i = 1:200
    utc = [2003 + floor(22*rand()), 1 + floor(12*rand()), 1 + floor(28*rand()), floor(24*rand()), floor(60*rand()), 60*rand()];
    geo = struct('alt_km', 150 + 850*rand(), 'lat_deg', -90 + 180*rand(), 'lon_deg', -180 + 360*rand(), ...
                 'lst_h', 24*rand(), 'doy', 1 + floor(365*rand()), 'utc', utc);
    sw = struct('F30', 50 + 150*rand(), 'F30_bar', 50 + 150*rand(), 'ap60', 80*rand(), 'f30_is_derived', rand() < 0.3);
    atm = atmos.provider('dtm2020_research', geo, sw);
    s = ['{"geo":' geojson(geo) ',"F30":' jn(sw.F30) ',"F30_bar":' jn(sw.F30_bar) ',"ap60":' jn(sw.ap60) ...
         ',"f30_is_derived":' jb(sw.f30_is_derived) ',"rho":' jn(atm.rho) ',"T":' jn(atm.T) ...
         ',"n":' jv([atm.n.H atm.n.He atm.n.O atm.n.N2 atm.n.O2 atm.n.N]) '}'];
    C{end+1} = s;
end
wlist(fid, 'dtm_res_atm', C);

%% ---------------- bint_oe / geogm -----------------------------------------
aps = [0:0.25:700, 657, 1000, 2.5, 3.3, 455.9];
kps = arrayfun(@bint_oe, aps);
fprintf(fid, '"bint_oe":{"ap":%s,"kp":%s},\n', jv(aps), jv(kps));
rand('seed', 15);
G = zeros(400, 4);
for i = 1:400
    la = -90 + 180*rand(); lo = -180 + 540*rand(); if i < 5, lo = 291; end
    [gl, gg] = geogm(la, lo); G(i,:) = [la lo gl gg];
end
fprintf(fid, '"geogm":{"lat":%s,"lon":%s,"gmlat":%s,"gmlon":%s},\n', jv(G(:,1)), jv(G(:,2)), jv(G(:,3)), jv(G(:,4)));

%% ---------------- JB2008 -----------------------------------------------------
% jb2008_density takes MATLAB datetime objects, which Octave does not have. The
% wrapper's arithmetic is replicated here line for line on datenum values (the
% same numbers datenum(datetime) returns), around the unmodified JB2008 core, and
% the SOLFSMY/DTCFILE parse mirrors get_jb2008_indices' parse_solfsmy/parse_dtcfile.
jbdir = fileparts(which('JB2008'));
IDX = jb_parse(fullfile(jbdir, 'SOLFSMY.TXT'), fullfile(jbdir, 'DTCFILE.TXT'));
rand('seed', 16);
C = {};
for i = 1:400
    utc = [1998 + floor(27*rand()), 1 + floor(12*rand()), 1 + floor(28*rand()), floor(24*rand()), floor(60*rand()), floor(600*rand())/10];
    if i <= 3, utc = [2026 5 20 12 0 0]; end            % past the file end: 'extrap'
    alt = 200 + 800*rand();
    if rand() < 0.1, alt = 120 + 80*rand(); end
    if rand() < 0.05, alt = 90 + 15*rand(); end
    if i == 10, alt = 1000; end
    lat = -90 + 180*rand(); lon = -180 + 360*rand();
    [rho, core] = jb_wrapper(utc, lon, lat, alt, IDX);
    s = ['{"utc":' jv(utc) ',"lon":' jn(lon) ',"lat":' jn(lat) ',"alt":' jn(alt) ',"rho":' jn(rho) ...
         ',"core":{"mjd":' jn(core.mjd) ',"sun":' jv(core.sun) ',"sat":' jv(core.sat) ',"ind":' jv(core.ind) ...
         ',"temp":' jv(core.temp) ',"rho":' jn(core.rho) '}}'];
    C{end+1} = s;
end
wlist(fid, 'jb2008', C);
% atmos.provider('jb2008') for a few points: rho, Mmol=16, nO, T=1000 placeholder
fprintf(fid, '"jb_nO_factor":%s,\n', jn(K.N_A*1000/16));

%% ---------------- exponential ----------------------------------------------
alts = [-5, 0:7:1200, 25, 30, 150, 180, 500, 1000, 1000.5, 89.9, 90, 100, 149.99];
E = zeros(numel(alts), 4);
for i = 1:numel(alts)
    a = atmos.exponential(alts(i)); E(i,:) = [a.rho a.T a.Mmol a.nO];
end
fprintf(fid, '"exponential":{"alt":%s,"rho":%s,"T":%s,"Mmol":%s,"nO":%s},\n', jv(alts), jv(E(:,1)), jv(E(:,2)), jv(E(:,3)), jv(E(:,4)));

%% ---------------- atmos.spaceweather manual ---------------------------------
C = {};
mans = {struct('F107',130,'F107a',130,'Kp',2,'ap',7), struct('F107',90,'F107a',90,'ap',8), ...
        struct('F107',150,'Kp',4.4), struct('F107',70,'F107a',75,'Kp',2.1,'ap',7,'ap3',12), ...
        struct('F107',100,'ap',400.5), struct('F107',100,'Kp',9.5), struct('F107',100,'ap',-3)};
for kp = 0:0.05:9.5, mans{end+1} = struct('F107', 120, 'Kp', kp); end
for ap = 0:0.5:410, mans{end+1} = struct('F107', 120, 'ap', ap); end
for i = 1:numel(mans)
    sw = atmos.spaceweather([2021 6 1 0 0 0], struct('manual', mans{i}));
    C{end+1} = ['{"manual":' manualjson(mans{i}) ',"F107":' jn(sw.F107) ',"F107a":' jn(sw.F107a) ',"Kp":' jn(sw.Kp) ...
                ',"ap":' jn(sw.ap) ',"ap3":' jn(sw.ap3) ',"aph":' jv(sw.aph) ',"F107_today":' jn(sw.F107_today) '}'];
end
wlist(fid, 'sw_manual', C);

%% ---------------- atmos.spaceweather table (synthetic table) -------------------
rand('seed', 17);
nd = 12;
T = struct();
T.mjd = (59000:59000+nd-1).';
T.f107obs = 70 + 150*rand(nd,1); T.f107c81 = 80 + 100*rand(nd,1);
T.kp = 5*rand(nd,1); T.apDaily = 40*rand(nd,1); T.ap = floor(100*rand(nd,8));
T.source = 'synthetic';
fprintf(fid, '"sw_table_def":{"mjd":%s,"f107obs":%s,"f107c81":%s,"kp":%s,"apDaily":%s,"ap":%s},\n', ...
        jv(T.mjd), jv(T.f107obs), jv(T.f107c81), jv(T.kp), jv(T.apDaily), jv(reshape(T.ap.', 1, [])));
C = {};
for i = 1:120
    dnum = 59000 + 678942 + nd*rand();
    dv = datevec(dnum); dv(6) = floor(dv(6));
    lag = rand() < 0.5; hist = rand() < 0.5;
    opts = struct('table', T, 'lag_f107', lag);
    if hist, opts.aph_mode = 'history'; end
    sw = atmos.spaceweather(dv, opts);
    C{end+1} = ['{"utc":' jv(dv) ',"lag":' jb(lag) ',"history":' jb(hist) ',"F107":' jn(sw.F107) ',"F107a":' jn(sw.F107a) ...
                ',"Kp":' jn(sw.Kp) ',"ap":' jn(sw.ap) ',"ap3":' jn(sw.ap3) ',"aph":' jv(sw.aph) ',"F107_today":' jn(sw.F107_today) '}'];
end
wlist(fid, 'sw_table', C);

%% ---------------- GSI panel coefficients -------------------------------------
rand('seed', 18);
n = 600;
s = 0.3 + 15*rand(n,1); dl = (pi/2)*rand(n,1); dl(1:10) = 0; dl(11:20) = pi/2;
aT = 0.5 + 0.5*rand(n,1); Tw = 200 + 200*rand(n,1); Ta = 400 + 1200*rand(n,1);
sn = 0.5 + 0.5*rand(n,1); st = 0.5 + 0.5*rand(n,1); nO = 10.^(12 + 4*rand(n,1));
R = zeros(n, 11);
for i = 1:n
    [cp, ct] = drag.sentman(s(i), dl(i), aT(i), Tw(i), Ta(i));
    [cp2, ct2] = drag.cll(s(i), dl(i), sn(i), st(i), Tw(i), Ta(i));
    [cp3, ct3] = drag.dria(s(i), dl(i), nO(i), Ta(i), Tw(i));
    R(i,:) = [cp ct cp2 ct2 cp3 ct3 drag.sesam(nO(i), Ta(i)) drag.speedRatio(7600*s(i)/5, Ta(i), 15.9994) 0 0 0];
end
fprintf(fid, '"gsi":{"s":%s,"delta":%s,"aT":%s,"Tw":%s,"Talt":%s,"sig_n":%s,"sig_t":%s,"nO":%s,', jv(s), jv(dl), jv(aT), jv(Tw), jv(Ta), jv(sn), jv(st), jv(nO));
fprintf(fid, '"sentman_cp":%s,"sentman_ct":%s,"cll_cp":%s,"cll_ct":%s,"dria_cp":%s,"dria_ct":%s,"sesam":%s,"speed_ratio":%s},\n', ...
        jv(R(:,1)), jv(R(:,2)), jv(R(:,3)), jv(R(:,4)), jv(R(:,5)), jv(R(:,6)), jv(R(:,7)), jv(R(:,8)));

%% ---------------- cannonball -------------------------------------------------
rand('seed', 19);
C = {};
for i = 1:200
    r = randorbit(); v = randvel(r);
    om = [1e-9*randn(2,1); 7.2921150e-5];
    if rand() < 0.2, om = 7.2921150e-5; end
    if rand() < 0.1, om = [0;0;0]; end
    wind = [0;0;0]; if rand() < 0.2, wind = 100*randn(3,1); end
    rho = 10^(-15 + 6*rand()); Cd = 1.5 + 2*rand(); A = 0.01 + rand(); m = 1 + 100*rand();
    oc = struct('mass', m, 'wind', wind, 'omega', om);
    out = drag.cannonball(r, v, rho, Cd, A, oc);
    C{end+1} = ['{"r":' jv(r) ',"v":' jv(v) ',"omega":' jv(om) ',"wind":' jv(wind) ',"rho":' jn(rho) ',"Cd":' jn(Cd) ...
                ',"A":' jn(A) ',"mass":' jn(m) ',"a":' jv(out.a) ',"F":' jv(out.F) ',"drag":' jn(out.drag) ',"Vrel":' jn(out.Vrel) '}'];
end
wlist(fid, 'cannonball', C);

%% ---------------- attitude helpers -------------------------------------------
rand('seed', 20);
C = {};
for i = 1:100
    r = randorbit(); v = randvel(r); vr = v - cross([0;0;7.2921150e-5], r);
    if i == 1, vr = [0; 0; 7000]; end                   % |x.z| > 0.98 branch
    aoa = 80*(rand()-0.5); ss = 40*(rand()-0.5);
    ax = randn(3,1); sb = randn(3,1); sb = sb/norm(sb);
    if i == 2, sb = ax/norm(ax); end                     % Sun along the pivot axis
    C{end+1} = ['{"r":' jv(r) ',"v":' jv(v) ',"vrel":' jv(vr) ',"aoa":' jn(aoa) ',"ss":' jn(ss) ...
                ',"ram":' jm(dgeom.ramAttitude(vr)) ',"aoa_R":' jm(dgeom.attitudeFromAoA(vr, aoa, ss)) ...
                ',"aoa_R0":' jm(dgeom.attitudeFromAoA(vr, aoa)) ',"lvlh":' jm(sgeom.R_lvlh(r, v)) ...
                ',"axis":' jv(ax) ',"sun_b":' jv(sb) ',"array_n":' jv(srp.arrayNormal(ax, sb)) '}'];
end
wlist(fid, 'attitude', C);

%% ---------------- drag.force (panel models) ----------------------------------
V16 = sgeom.vleo16u();
geoms = {V16.facets, dgeom.addArray(dgeom.buildBox(0.34, 0.20, 0.20), [0;0;1], 0.12), struct('n',[1;0;0],'A',0.04)};
gnames = {'vleo16u', 'box_array', 'plate'};
models = {'sentman', 'dria', 'sesam', 'cll'};
rand('seed', 21);
C = {};
for i = 1:320
    gi = 1 + mod(i-1, 3); mi = 1 + mod(floor((i-1)/3), 4);
    r = randorbit(); v = randvel(r);
    om = [1e-9*randn(2,1); 7.2921150e-5]; if rand() < 0.15, om = 7.2921150e-5; end
    vr = v - cross([0;0;7.2921150e-5], r);
    q = rand();
    if q < 0.35
        Rbi = dgeom.ramAttitude(vr);
    elseif q < 0.7
        Rbi = dgeom.attitudeFromAoA(vr, 90*(rand()-0.5), 60*(rand()-0.5));
    elseif q < 0.85
        Rbi = sgeom.R_lvlh(r, v);
    else
        [Qm, ~] = qr(randn(3)); if det(Qm) < 0, Qm(:,1) = -Qm(:,1); end, Rbi = Qm;
    end
    sun = randn(3,1); sun = 1.496e11*sun/norm(sun);
    if rand() < 0.5
        geo = struct('alt_km', 180 + 700*rand(), 'lat_deg', -90 + 180*rand(), 'lon_deg', 360*rand(), ...
                     'lst_h', 24*rand(), 'doy', 1 + floor(365*rand()), 'utc', [2020 1 1 0 0 0]);
        sw = atmos.spaceweather([2020 1 1 0 0 0], struct('manual', struct('F107', 70 + 180*rand(), 'F107a', 70 + 180*rand(), 'Kp', 6*rand())));
        atm = atmos.provider('dtm2020', geo, sw);
        atmj = ['{"species":true,"T":' jn(atm.T) ',"n":' jv([atm.n.H atm.n.He atm.n.O atm.n.N2 atm.n.O2 atm.n.N]) '}'];
    else
        atm = atmos.exponential(150 + 800*rand());
        atmj = ['{"species":false,"T":' jn(atm.T) ',"rho":' jn(atm.rho) ',"Mmol":' jn(atm.Mmol) ',"nO":' jn(atm.nO) '}'];
    end
    gsi = struct('Tw', 250 + 150*rand(), 'aT', 0.6 + 0.4*rand(), 'sig_n', 0.6 + 0.4*rand(), 'sig_t', 0.6 + 0.4*rand());
    mass = 5 + 50*rand(); Aref = 0.02 + 0.1*rand();
    opts = struct('mass', mass, 'Aref', Aref, 'R_bi', Rbi, 'wind', [0;0;0], 'omega', om, 'sunHat_eci', sun);
    out = drag.force(r, v, atm, geoms{gi}, models{mi}, gsi, opts);
    C{end+1} = ['{"geom":"' gnames{gi} '","model":"' models{mi} '","r":' jv(r) ',"v":' jv(v) ',"omega":' jv(om) ...
                ',"R_bi":' jm(Rbi) ',"sun":' jv(sun) ',"atm":' atmj ',"Tw":' jn(gsi.Tw) ',"aT":' jn(gsi.aT) ...
                ',"sig_n":' jn(gsi.sig_n) ',"sig_t":' jn(gsi.sig_t) ',"mass":' jn(mass) ',"Aref":' jn(Aref) ...
                ',"a":' jv(out.a) ',"F":' jv(out.F) ',"Fbody":' jv(out.Fbody) ',"drag":' jn(out.drag) ',"lift":' jn(out.lift) ...
                ',"side":' jn(out.side) ',"Cd":' jn(out.Cd) ',"Cd_A":' jn(out.Cd_A) ',"A_proj":' jn(out.A_proj) ...
                ',"alpha":' jn(out.alpha) ',"beta":' jn(out.beta) ',"qd":' jn(out.qd) ',"s_O":' jn(out.s_O) '}'];
end
wlist(fid, 'panel', C);

%% ---------------- forces.drag end to end -------------------------------------
rand('seed', 22);
C = {};
SC16 = sgeom.vleo16u();
for i = 1:160
    kind = mod(i-1, 8);
    r = randorbit(); v = randvel(r);
    th = 2*pi*rand(); Ce = [cos(th) sin(th) 0; -sin(th) cos(th) 0; 0 0 1];
    ctx = struct();
    ctx.r_eci = r; ctx.v_eci = v; ctx.r_ecef = Ce*r;
    ctx.utc = [2019 + floor(6*rand()), 1 + floor(12*rand()), 1 + floor(28*rand()), floor(24*rand()), floor(60*rand()), 60*rand()];
    ctx.T = struct('doy', 1 + floor(365*rand()));
    ctx.omega_eci = [1e-9*randn(2,1); 7.2921150e-5];
    ctx.E = struct('sun_eci', 1.496e11*unitv(randn(3,1)));
    ctx.swtable = []; ctx.jbidx = []; ctx.drv = [];
    ctx.swmanual = struct('F107', 130, 'F107a', 130, 'Kp', 2, 'ap', 7);
    sc = struct('mass', 4 + 20*rand(), 'Aref', 0.01 + 0.08*rand(), 'Cd', 2 + rand(), 'R_bi', eye(3));
    d = struct('on', true, 'model', 'cannonball', 'atmos', 'dtm2020', 'corotate', true);
    switch kind
        case 0   % the SILS path: cannonball + DTM2020 operational + manual 130/130/2/7
        case 1   % Cd from forces.drag overrides sc.Cd; corotate off
            d.Cd = 2.5 + rand(); d.corotate = false;
        case 2   % exponential
            d.atmos = 'exponential';
        case 3   % manual indices with only ap (Kp derived) and no F107a
            ctx.swmanual = struct('F107', 70 + 180*rand(), 'ap', 30*rand());
        case 4   % dria on the 16U, ram attitude, tracking arrays need the Sun
            d.model = 'dria'; sc.facets = SC16.facets;
            sc.R_bi = dgeom.ramAttitude(v - cross(ctx.omega_eci, r));
        case 5   % cll on a box without facets field -> single plate; corotate off
            d.model = 'cll'; d.gsi = struct('Tw', 300, 'aT', 0.9, 'sig_n', 0.8, 'sig_t', 0.7); d.corotate = false;
            d.atmos = 'exponential';
        case 6   % sentman on the 16U at an angle of attack
            d.model = 'sentman'; sc.facets = SC16.facets;
            sc.R_bi = dgeom.attitudeFromAoA(v - cross(ctx.omega_eci, r), 60*(rand()-0.5), 30*(rand()-0.5));
            ctx.swmanual = struct('F107', 70 + 180*rand(), 'F107a', 70 + 180*rand(), 'Kp', 7*rand());
        case 7   % sesam, no omega_eci in ctx (scalar default path) , dgeom box
            d.model = 'sesam'; sc.facets = dgeom.buildBox(0.3, 0.1, 0.1);
            sc.R_bi = dgeom.ramAttitude(v - cross(ctx.omega_eci, r));
            ctx = rmfield(ctx, 'omega_eci');
    end
    ctx.cfg.forces.drag = d; ctx.sc = sc;
    [a, info] = forces.drag(ctx);
    [lat, lon, alt] = op.geodetic(ctx.r_ecef);
    gj = '"none"';
    if isfield(sc, 'facets'), if kind == 7, gj = '"box_0.3_0.1_0.1"'; else, gj = '"vleo16u"'; end, end
    cdj = 'null'; if isfield(d, 'Cd'), cdj = jn(d.Cd); end
    gsij = 'null'; if isfield(d, 'gsi'), gsij = ['[' jn(d.gsi.Tw) ',' jn(d.gsi.aT) ',' jn(d.gsi.sig_n) ',' jn(d.gsi.sig_t) ']']; end
    omj = 'null'; if isfield(ctx, 'omega_eci'), omj = jv(ctx.omega_eci); end
    C{end+1} = ['{"kind":' jn(kind) ',"model":"' d.model '","atmos":"' d.atmos '","corotate":' jb(d.corotate) ',"cfg_Cd":' cdj ',"gsi":' gsij ...
                ',"r":' jv(r) ',"v":' jv(v) ',"lat":' jn(lat) ',"lon":' jn(lon) ',"alt":' jn(alt) ',"utc":' jv(ctx.utc) ...
                ',"doy":' jn(ctx.T.doy) ',"omega":' omj ',"sun":' jv(ctx.E.sun_eci) ',"manual":' manualjson(ctx.swmanual) ...
                ',"mass":' jn(sc.mass) ',"Aref":' jn(sc.Aref) ',"sc_Cd":' jn(sc.Cd) ',"R_bi":' jm(sc.R_bi) ',"geom":' gj ...
                ',"a":' jv(a) ',"rho":' jn(info.atm.rho) ',"T":' jn(info.atm.T) ',"lst":' jn(info.geo.lst_h) ',"v_rel":' jv(info.v_rel) '}'];
end
wlist(fid, 'chain', C, true);
fprintf(fid, '}\n');
fclose(fid);
printf('wrote %s\n', outf);

