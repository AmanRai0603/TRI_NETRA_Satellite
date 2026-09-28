% TIME_FRAMES  Reference vectors for adcs-pop time.rs / eop.rs / frames / geodetic.rs.
% Evaluates the MATLAB POP functions (timeconv.*, op.addsec, op.geodetic,
% frames.eci2ecef for builds gmst/A/B/C, the tidal EOP models, and a verbatim
% copy of op.buildWorld's local earthRateECI) and writes
%   ../tests/data/time_frames.json
% Builds A/B/C need IERS EOP files; POP's cache (data_cache/eop) is empty and
% there is no network, so this script first WRITES a synthetic EOP set in POP's
% own file formats (Leap_Second.dat from timeconv.leapTable, finals2000A.all and
% EOP 20 C04 with smooth made-up values) to ../tests/data/time_frames_eop/ and
% points opt.data_dir at it. The Rust tests parse the very same files, so the
% parsers, splice and interpolation are compared as well as the rotation kernel.
%
% Run from matlab_sils/pop:
%   octave-cli --no-gui -q --eval "setup_paths; run('<abs>/refgen/time_frames.m')"
1;

function s = jn(x)                     % number(s) -> JSON, full precision
    x = double(x);
    if isempty(x), s = '[]'; return; end
    if isscalar(x), s = jnum(x); return; end
    x = x(:).'; c = arrayfun(@jnum, x, 'UniformOutput', false);
    s = ['[' strjoin(c, ',') ']'];
end
function s = jnum(v)
    if isnan(v), s = 'null';
    elseif isinf(v), if v > 0, s = '1e999'; else, s = '-1e999'; end
    else, s = sprintf('%.17g', v); end
end
function s = jm(M), s = jn(reshape(M.', 1, [])); end   % 3x3 -> row-major
function s = jobj(varargin)            % jobj('k1', 'json1', 'k2', 'json2', ...)
    p = cell(1, numel(varargin)/2);
    for i = 1:2:numel(varargin), p{(i+1)/2} = sprintf('"%s":%s', varargin{i}, varargin{i+1}); end
    s = ['{' strjoin(p, ',') '}'];
end
function s = jlist(c), s = ['[' strjoin(c, ',') ']']; end
function s = pad(n), s = repmat(' ', 1, n); end
function L = put(L, a, b, str)         % right-align str into columns a..b (1-based)
    w = b - a + 1; assert(numel(str) <= w, 'field too wide: %s', str);
    L(a:b) = [pad(w - numel(str)) str];
end

function w = earthRateECI_copy(epoch, build, fopt)   % verbatim op.buildWorld/earthRateECI
    try
        d = 15.0;
        up = op.addsec(epoch,  d); um = op.addsec(epoch, -d);
        Tp = timeconv.convertUTC(up(1),up(2),up(3),up(4),up(5),up(6),0);
        Tm = timeconv.convertUTC(um(1),um(2),um(3),um(4),um(5),um(6),0);
        T0 = timeconv.convertUTC(epoch(1),epoch(2),epoch(3),epoch(4),epoch(5),epoch(6),0);
        fp = fopt; fp.gmst_rad = Tp.gmst_rad;
        fm = fopt; fm.gmst_rad = Tm.gmst_rad;
        f0 = fopt; f0.gmst_rad = T0.gmst_rad;
        [~, Ctp] = frames.eci2ecef(up,    build, fp);
        [~, Ctm] = frames.eci2ecef(um,    build, fm);
        [C0, ~ ] = frames.eci2ecef(epoch, build, f0);
        Cdot = (Ctp - Ctm)/(2*d);
        M = Cdot * C0;
        w = [M(3,2); M(1,3); M(2,1)];
    catch ME
        K = de440.constants(); w = [0;0;K.omega_earth];
    end
end

here   = fileparts(mfilename('fullpath'));
outdir = fullfile(here, '..', 'tests', 'data');
eopdir = fullfile(outdir, 'time_frames_eop');
if ~exist(eopdir, 'dir'), mkdir(eopdir); end

% ============================================================ synthetic EOP set
LT = timeconv.leapTable();
fid = fopen(fullfile(eopdir, 'Leap_Second.dat'), 'w');
fprintf(fid, '#  SYNTHETIC test file for adcs-pop (IERS Leap_Second.dat layout).\n#  Rows = timeconv.leapTable of POP.\n#\n#    MJD        Date        TAI-UTC (s)\n#           day month year\n#    ---    --------------   ------\n#\n');
for i = 1:size(LT,1)
    mj = timeconv.cal2jd(LT(i,1), LT(i,2), 1) - 2400000.5;
    fprintf(fid, '    %7.1f    1 %2d %4d       %2d\n', mj, LT(i,2), LT(i,1), LT(i,3));
end
fclose(fid);
lap = @(m) LT(find(timeconv.cal2jd(LT(:,1),LT(:,2),1)-2400000.5 <= m, 1, 'last'), 3);

M0 = 58635; M1 = 64700; MB = 60900; C1 = 60700;       % finals span, Bulletin-B end, C04 end
xs  = @(m) 0.12 + 0.15*sin(2*pi*(m-58000)/433) + 0.08*cos(2*pi*(m-58000)/365.25);
ys  = @(m) 0.35 + 0.15*cos(2*pi*(m-58000)/433) - 0.06*sin(2*pi*(m-58000)/365.25);
uts = @(m) -36.8 - 0.0001*(m-M0) + 0.02*sin(2*pi*m/182.6) + 0.01*cos(2*pi*m/13.66);   % UT1-TAI
dxs = @(m) 0.20*sin(2*pi*m/430) + 0.05;               % mas
dys = @(m) -0.15*cos(2*pi*m/430) - 0.02;              % mas
fid = fopen(fullfile(eopdir, 'finals2000A.all'), 'w');
for m = M0:M1+8
    [Y,Mo,D] = timeconv.jd2cal(m + 2400000.5);
    L = pad(185);
    L = put(L, 1, 6, sprintf('%02d%2d%2d', mod(Y,100), Mo, D));
    L = put(L, 8, 15, sprintf('%8.2f', m));
    if m > M1                                           % trailing rows without data
        L = L(1:70); fprintf(fid, '%s\n', L); continue;
    end
    L(17) = 'I';
    L = put(L, 19, 27, sprintf('%9.6f', xs(m)));   L = put(L, 28, 36, sprintf('%9.6f', 0.000091));
    L = put(L, 38, 46, sprintf('%9.6f', ys(m)));   L = put(L, 47, 55, sprintf('%9.6f', 0.000087));
    L(58) = 'I';
    L = put(L, 59, 68, sprintf('%10.7f', uts(m) + lap(m)));  L = put(L, 69, 78, sprintf('%10.7f', 0.0000123));
    L = put(L, 80, 86, sprintf('%7.4f', 0.8123));  L = put(L, 87, 93, sprintf('%7.4f', 0.0102));
    L(95) = 'I';
    L = put(L, 98, 106, sprintf('%9.3f', dxs(m))); L = put(L, 107, 115, sprintf('%9.3f', 0.1));
    L = put(L, 117, 125, sprintf('%9.3f', dys(m)));L = put(L, 126, 134, sprintf('%9.3f', 0.1));
    if m <= MB                                          % Bulletin B columns
        L = put(L, 135, 144, sprintf('%10.6f', xs(m) + 0.000213));
        L = put(L, 145, 154, sprintf('%10.6f', ys(m) - 0.000147));
        L = put(L, 155, 165, sprintf('%11.7f', uts(m) + lap(m) + 0.0000311));
        if m < 60800 || m > 60850                       % a window with no B dX,dY -> A's used
            L = put(L, 166, 175, sprintf('%10.3f', dxs(m) + 0.012));
            L = put(L, 176, 185, sprintf('%10.3f', dys(m) - 0.009));
        end
    else
        L = deblank(L(1:134));
    end
    fprintf(fid, '%s\n', L);
end
fclose(fid);

fid = fopen(fullfile(eopdir, 'eopc04_20.1962-now'), 'w');
fprintf(fid, '# SYNTHETIC test file for adcs-pop in the EOP 20 C04 (2023) layout.\n');
fprintf(fid, '# FORMAT(3(I4),I3,2X,F9.2,2(F12.6),F12.7,2(F12.6),2(F12.6),F12.7,2(F12.6),F12.7,2(F12.6),2(F12.6),F12.7)\n');
fprintf(fid, '#      YR  MM  DD  HH       MJD        x(")        y(")  UT1-UTC(s)       dX(")      dY(")       xrt(")      yrt(")      LOD(s)        x Er      y Er  UT1-UTC Er      dX Er       dY Er       xrt Er      yrt Er      LOD Er\n');
for m = M0:C1
    [Y,Mo,D] = timeconv.jd2cal(m + 2400000.5);
    fprintf(fid, '%4d%4d%4d%3d  %9.2f%12.6f%12.6f%12.7f%12.6f%12.6f%12.6f%12.6f%12.7f%12.6f%12.6f%12.7f%12.6f%12.6f%12.6f%12.6f%12.7f\n', ...
        Y, Mo, D, 0, m, xs(m) + 0.000105, ys(m) - 0.000088, uts(m) + lap(m) - 0.0000207, ...
        (dxs(m) + 0.031)/1000, (dys(m) - 0.027)/1000, 0.000312, -0.000211, 0.0008123, ...
        0.000031, 0.000029, 0.0000102, 0.000045, 0.000047, 0.000021, 0.000022, 0.0000081);
    if m == 59000, fprintf(fid, '%4d %9.2f %12.6f %12.6f %12.6f %12.6f %12.6f %12.6f\n', 2020, m, 1, 2, 3, 4, 5, 6); end  % odd-length row
end
fclose(fid);

% ============================================================ time scales
ep = [2020 1 1 0 0 0; 2020 3 15 12 34 56.789; 2021 7 1 0 0 0; 2022 12 31 23 59 59.5;
      2024 2 29 6 0 0; 2025 1 25 0 0 0; 2025 6 15 18 0 0; 2026 9 28 10 7 35.25;
      2027 1 1 6 0 0; 2027 1 1 6 0 0.1; 2028 12 31 23 59 59.999; 2030 5 5 5 5 5;
      2031 11 11 11 11 11.111; 2033 3 3 3 3 3; 2035 12 31 23 0 0;
      2015 6 30 23 59 59; 2015 6 30 23 59 60; 2015 7 1 0 0 0; 2016 12 31 23 59 59;
      2016 12 31 23 59 60; 2017 1 1 0 0 0; 2016 12 31 23 59 59.999999;
      1971 12 31 0 0 0; 1972 1 1 0 0 0; 2000 1 1 12 0 0; 1980 1 6 0 0 0];
for k = 0:23, ep(end+1,:) = [2020 + floor(k*16/24), 1 + mod(k*5,12), 1 + mod(k*7,28), mod(k*5,24), mod(k*13,60), mod(k*17.37,60)]; end %#ok
dus = [0 -0.2 0.35 0.9 -0.65];
cu = {};
for i = 1:size(ep,1)
    du = dus(1 + mod(i, numel(dus)));
    T = timeconv.convertUTC(ep(i,1),ep(i,2),ep(i,3),ep(i,4),ep(i,5),ep(i,6),du);
    o = jobj('utc_jd',jn(T.utc_jd),'tai_jd',jn(T.tai_jd),'tt_jd',jn(T.tt_jd),'tdb_jd',jn(T.tdb_jd), ...
        'ut1_jd',jn(T.ut1_jd),'gps_jd',jn(T.gps_jd),'utc_mjd',jn(T.utc_mjd),'tt_mjd',jn(T.tt_mjd), ...
        'tdb_mjd',jn(T.tdb_mjd),'leap',jn(T.leap),'dut1',jn(T.dUT1),'t_tt',jn(T.T_tt), ...
        'j2000_tt_sec',jn(T.j2000_tt_sec),'gps_week',jn(T.gps_week),'gps_sow',jn(T.gps_sow), ...
        'gmst_rad',jn(T.gmst_rad),'doy',jn(T.doy));
    [Y,Mo,D,h,mi,s] = timeconv.jd2cal(T.tt_jd);
    cu{end+1} = jobj('in', jn([ep(i,:) du]), 'out', o, 'jd2cal_tt', jn([Y Mo D h mi s]), ...
        'tt2utc', jn(timeconv.tt2utc(T.tt_jd)), 'tai2utc', jn(timeconv.tai2utc(T.tai_jd))); %#ok
end

% ============================================================ addsec
ae = [2027 1 1 6 0 0; 2026 12 31 23 59 59.9; 2024 2 28 23 59 59; 2020 1 1 0 0 0; 2016 12 31 23 59 59; 2030 6 30 12 30 30.5];
dts = [0 1 -1 0.1 -0.1 59.9 86400 -86400 3.2e7 -3.2e7 1e-3 12345.678 -0.5 0.2 15 -15 5400.25 1e9*0+7777777.7];
ad = {};
for i = 1:size(ae,1), for dt = dts
    ad{end+1} = jobj('utc', jn(ae(i,:)), 'sec', jn(dt), 'out', jn(op.addsec(ae(i,:), dt))); %#ok
end, end
for k = 0:40   % the propagator's own pattern: epoch + t, t = k*0.1 and k*10.3
    ad{end+1} = jobj('utc', jn([2027 1 1 6 0 0]), 'sec', jn(k*0.1), 'out', jn(op.addsec([2027 1 1 6 0 0], k*0.1))); %#ok
    ad{end+1} = jobj('utc', jn([2027 1 1 6 0 0]), 'sec', jn(k*10.3), 'out', jn(op.addsec([2027 1 1 6 0 0], k*10.3))); %#ok
end

% ============================================================ gmst build
gopts = {struct(), struct('dUT1',-0.3), struct('xp',1e-6,'yp',2e-6), struct('gmst_rad',1.234), ...
         struct('dUT1',0.5,'xp',-3e-7,'yp',4e-7), struct('dUT1',0.1,'gmst_rad',[])};
gm = {};
for i = 1:size(ep,1), for j = 1:numel(gopts)
    if mod(i+j,3) ~= 0 && i > 3, continue; end
    o = gopts{j};
    [C, Ct, info] = frames.eci2ecef(ep(i,:), 'gmst', o);
    dut1 = 0; xp = 0; yp = 0; g = [];
    if isfield(o,'dUT1'), dut1 = o.dUT1; end
    if isfield(o,'xp'), xp = o.xp; end
    if isfield(o,'yp'), yp = o.yp; end
    if isfield(o,'gmst_rad') && ~isempty(o.gmst_rad), g = o.gmst_rad; end
    gm{end+1} = jobj('utc', jn(ep(i,:)), 'dut1', jn(dut1), 'xp', jn(xp), 'yp', jn(yp), 'gmst_rad', jn(g), ...
                     'C', jm(C), 'Ct', jm(Ct), 'info_gmst', jn(info.gmst_rad)); %#ok
end, end

% ============================================================ builds A / B / C
ce = [2020 3 15 12 34 56.789; 2021 7 1 0 0 0; 2022 12 31 23 59 59.5; 2024 2 29 6 0 0;
      2025 1 25 0 0 0; 2025 1 26 7 0 0; 2025 6 15 18 0 0; 2025 7 20 3 0 0; 2027 1 1 6 0 0;
      2027 1 1 6 0 15; 2030 5 5 5 5 5; 2035 12 31 23 0 0; 2036 2 1 0 0 0; 2019 5 1 0 0 0;
      2019 6 1 0 0 0; 2036 1 7 12 0 0];
cc = {};
for b = {'A','B','C'}
    for i = 1:size(ce,1)
        tl = [0 1]; if strcmp(b{1},'C'), tl = 0; end
        for tidal = tl
            o = struct('data_dir', eopdir, 'verbose', false, 'tidal', logical(tidal));
            [C, Ct, info] = frames.eci2ecef(ce(i,:), b{1}, o);
            cc{end+1} = jobj('utc', jn(ce(i,:)), 'build', ['"' b{1} '"'], 'tidal', jn(tidal), 'override', '[]', ...
                'C', jm(C), 'Ct', jm(Ct), 'mjd_utc', jn(info.mjd_utc), 'dut1', jn(info.dUT1), 'xp', jn(info.xp), ...
                'yp', jn(info.yp), 'dx', jn(info.dX), 'dy', jn(info.dY), 'dat', jn(info.dAT), 'X', jn(info.X), ...
                'Y', jn(info.Y), 's', jn(info.s), 'era', jn(info.era), 'eop_flag', jn(info.eop_flag)); %#ok
        end
    end
    for eo = {[0.123 1e-6 2e-6 1e-9 -2e-9 37], [-0.4 -5e-7 1.5e-6 0 0 37], [0 0 0 0 0 32]}
        for i = [1 9 12]
            o = struct('data_dir', eopdir, 'verbose', false, 'eop_override', eo{1});
            [C, Ct, info] = frames.eci2ecef(ce(i,:), b{1}, o);
            cc{end+1} = jobj('utc', jn(ce(i,:)), 'build', ['"' b{1} '"'], 'tidal', jn(0), 'override', jn(eo{1}), ...
                'C', jm(C), 'Ct', jm(Ct), 'mjd_utc', jn(info.mjd_utc), 'dut1', jn(info.dUT1), 'xp', jn(info.xp), ...
                'yp', jn(info.yp), 'dx', jn(info.dX), 'dy', jn(info.dY), 'dat', jn(info.dAT), 'X', jn(info.X), ...
                'Y', jn(info.Y), 's', jn(info.s), 'era', jn(info.era), 'eop_flag', jn(info.eop_flag)); %#ok
        end
    end
end

% ============================================================ tidal models
tm = {};
for mjd = [47100 54335 44239.1 55227.4 60735.5 61406.25 58849.123 62500.9 64000]
    [ox,oy,ou] = tidal_eop_ocean(mjd); [px,py] = tidal_pm_libration(mjd); [uu,ul] = tidal_ut1_libration(mjd);
    [tx,ty,tu] = tidal_eop(mjd);
    tm{end+1} = jobj('mjd', jn(mjd), 'ocean', jn([ox oy ou]), 'pm', jn([px py]), 'ut1lib', jn([uu ul]), ...
                     'zonal', jn(tidal_ut1_zonal(mjd)), 'total', jn([tx ty tu])); %#ok
end

% ============================================================ geodetic
ge = {};
R = [6778137 0 0; 0 6778137 0; 4000000 3000000 4500000; -2500000.5 -4800000.25 3700000.125;
     1e3 2e3 6.4e6; 0 0 -6.4e6; 7000000 -1 -300000; 6378137 0 0; 26000000 12000000 -8000000;
     -6600000 100000 -1200000; 1500000 -6000000 -2800000];
for i = 1:size(R,1)
    [la, lo, al] = op.geodetic(R(i,:).');
    ge{end+1} = jobj('r', jn(R(i,:)), 'lat', jn(la), 'lon', jn(lo), 'alt', jn(al)); %#ok
end

% ============================================================ earth rate (buildWorld)
er = {};
fo = {struct('build','gmst','dUT1',0.1), struct('build','gmst'), ...
      struct('build','A','data_dir',eopdir,'verbose',false), ...
      struct('build','B','data_dir',eopdir,'verbose',false), ...
      struct('build','C','data_dir',eopdir,'verbose',false)};
for e = {[2027 1 1 6 0 0], [2021 7 1 0 0 0]}
    for j = 1:numel(fo)
        w = earthRateECI_copy(e{1}, fo{j}.build, fo{j});
        du = 0; if isfield(fo{j},'dUT1'), du = fo{j}.dUT1; end
        er{end+1} = jobj('epoch', jn(e{1}), 'build', ['"' fo{j}.build '"'], 'dut1', jn(du), 'w', jn(w)); %#ok
    end
end

J = jobj('convert_utc', jlist(cu), 'addsec', jlist(ad), 'gmst_build', jlist(gm), 'cio', jlist(cc), ...
         'tidal', jlist(tm), 'geodetic', jlist(ge), 'earth_rate', jlist(er));
fid = fopen(fullfile(outdir, 'time_frames.json'), 'w'); fprintf(fid, '%s\n', J); fclose(fid);
fprintf('wrote %s (%d convertUTC, %d addsec, %d gmst, %d cio, %d tidal, %d geodetic, %d earth-rate)\n', ...
        fullfile(outdir, 'time_frames.json'), numel(cu), numel(ad), numel(gm), numel(cc), numel(tm), numel(ge), numel(er));
