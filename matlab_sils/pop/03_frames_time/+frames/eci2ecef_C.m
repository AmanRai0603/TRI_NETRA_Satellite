function [C, Ct, info] = eci2ecef_C(utc, opt)
%eci2ecef_C  ECI (GCRS/J2000) -> ECEF (ITRS) rotation via the full IAU 2006/2000A
%  CIO chain with real IERS Earth-Orientation Parameters. Self-contained: every
%  routine below is a local function; only dependency is xys06_tables.mat sitting
%  in the SAME folder as this file. Build "C" = MAXIMUM standardized accuracy:
%  build B (EOP 20 C04 + finals tail) PLUS cubic (4-pt Lagrange) EOP
%  interpolation, UT1 zonal-tide regularisation (RG_ZONT2), and the full IERS
%  sub-daily tidal model (ocean ORTHO_EOP + PM libration PMSDNUT2 + UT1
%  libration UTLIBR). Extra deps in this folder: tidal_eop.m (+ its 3 parts)
%  and tidal_ut1_zonal.m.
%
%  WHY C IS THE MAXIMUM-ACCURACY BUILD (and why it differs from Orekit ~1 mm):
%   * rotation theory  : IAU 2006/2000A CIO chain (the IAU/IERS standard - maximal)
%   * EOP product      : EOP 20 C04 (ITRF2020/ICRF3), dX,dY include Free Core Nutation
%   * interpolation    : N-point Lagrange (NPTS, default cubic) of the daily EOP
%   * UT1 regularise   : RG_ZONT2 zonal tides removed before interp, added back (opt.zonal)
%   * sub-daily        : full IERS ortho_eop + PM libration + UT1 libration
%   Against the TRUE pole this is as accurate as standardized models allow; the
%   remaining error is data-limited (IERS EOP uncertainty ~mm), not model-limited.
%   Against OREKIT it differs by ~1 mm because Orekit (a) does NOT apply RG_ZONT2
%   UT1 regularisation and (b) uses its own EOP interpolation scheme. Set
%   opt.zonal=false and match NPTS to Orekit to converge to Orekit; keep the
%   defaults (zonal=true, cubic) for maximum PHYSICAL accuracy.
%
%    C  = eci2ecef_C(utc)              % 3x3 ;  r_ecef = C * r_eci
%   [C,Ct,info] = eci2ecef_C(utc,opt)  % Ct = ECEF->ECI ; info = EOP used
%
%  utc : datetime (UTC) scalar/array  OR  [Y Mo D H Mi S] row (or Nx6).
%        For an array -> C is 3x3xN.
%  opt (optional struct, all fields have defaults):
%        .force_download  re-fetch EOP even if cache is fresh   (default false)
%        .force_reload    re-parse EOP even if session-cached   (default false)
%        .verbose         print fetch/parse progress            (default true)
%        .c04_url         cell array to override the C04 source URLs  (build B)
%        .c04_dxdy        [iX iY] force the dX,dY column indices (build B)
%
%  Build A = finals2000A.all (Bulletin A/B). Reproduces dcmeci2ecef and Orekit
%            getITRF(IERS_2010, simpleEOP=true).
%  Build B = EOP 20 C04 (ITRF2020 / ICRF3) spliced with the finals rapid tail.
%
%  EOP is downloaded ONCE into this folder and reused; re-fetched only if missing
%  or older than 7 days. Kernel validated vs SOFA/ERFA canonical vectors
%  (X,Y <3e-19 rad; composed matrix 2.85e-12 vs c2t06a).

if nargin < 2 || isempty(opt), opt = struct(); end
opt = defaults(opt);
tab = get_tables();
U   = utcmat(utc);
N   = size(U,1);
EOP = get_eop('B', opt);

C = zeros(3,3,N);
for k = 1:N
    mjd = mjd_of(U(k,:));
    if isfield(opt,'eop_override') && ~isempty(opt.eop_override)
        eo=opt.eop_override; dUT1=eo(1); xp=eo(2); yp=eo(3); dX=eo(4); dY=eo(5); dAT=eo(6); ef=2;
    else
        [dUT1, xp, yp, dX, dY, dAT, ef] = eop_interp(mjd, EOP);
    end
    Ts = time_scales(U(k,:), dUT1, dAT);
    fa = fund_args(Ts.t);
    [X, Y] = xy06(Ts.t, fa, tab);   X = X + dX;  Y = Y + dY;
    s   = s06(Ts.t, fa, X, Y, tab);
    Q   = c2ixys(X, Y, s);
    era = era00(Ts.jd_ut1(1), Ts.jd_ut1(2));
    W   = pom00(xp, yp, sp00(Ts.t));
    C(:,:,k) = W * rot(3, era) * Q;
end
if N == 1
    C = C(:,:,1);  Ct = C.';
    info = struct('mjd_utc',mjd,'dUT1',dUT1,'xp',xp,'yp',yp,'dX',dX,'dY',dY, ...
                  'dAT',dAT,'X',X,'Y',Y,'s',s,'era',era,'build','C','eop_flag',ef);
else
    Ct = permute(C,[2 1 3]);
    info = struct('build','C','N',N);
end
end

% ====================================================================== OPTIONS
function opt = defaults(o)
d = struct('force_download',false,'force_reload',false,'verbose',true,'zonal',true,'data_dir','');
f = fieldnames(d);
for i=1:numel(f), if ~isfield(o,f{i}), o.(f{i}) = d.(f{i}); end, end
opt = o;
end

% ================================================================ TABLE LOADING
function T = get_tables()
persistent TAB
if isempty(TAB)
    here = fileparts(mfilename('fullpath'));
    f = fullfile(here,'xys06_tables.mat');
    assert(isfile(f),'%s: xys06_tables.mat must sit next to this file', mfilename);
    TAB = load(f);
end
T = TAB;
end

% ============================================================ KERNEL: CIP / ROT
function fa = fund_args(t)
AS2R=4.848136811095359935899141e-6; TURNAS=1296000.0; D2PI=6.283185307179586476925287;
fa=zeros(14,1);
fa(1)=mod(485868.249036+t*(1717915923.2178+t*(31.8792+t*(0.051635+t*(-0.00024470)))),TURNAS)*AS2R;
fa(2)=mod(1287104.793048+t*(129596581.0481+t*(-0.5532+t*(0.000136+t*(-0.00001149)))),TURNAS)*AS2R;
fa(3)=mod(335779.526232+t*(1739527262.8478+t*(-12.7512+t*(-0.001037+t*(0.00000417)))),TURNAS)*AS2R;
fa(4)=mod(1072260.703692+t*(1602961601.2090+t*(-6.3706+t*(0.006593+t*(-0.00003169)))),TURNAS)*AS2R;
fa(5)=mod(450160.398036+t*(-6962890.5431+t*(7.4722+t*(0.007702+t*(-0.00005939)))),TURNAS)*AS2R;
fa(6)=mod(4.402608842+2608.7903141574*t,D2PI);
fa(7)=mod(3.176146697+1021.3285546211*t,D2PI);
fa(8)=mod(1.753470314+628.3075849991*t,D2PI);
fa(9)=mod(6.203480913+334.0612426700*t,D2PI);
fa(10)=mod(0.599546497+52.9690962641*t,D2PI);
fa(11)=mod(0.874016757+21.3299104960*t,D2PI);
fa(12)=mod(5.481293872+7.4781598567*t,D2PI);
fa(13)=mod(5.311886287+3.8133035638*t,D2PI);
fa(14)=(0.024381750+0.00000538691*t)*t;
end

function [X,Y] = xy06(t, fa, tab)
AS2R=4.848136811095359935899141e-6; fa=fa(:);
jxy=tab.xy_terms(:,1); p=tab.xy_terms(:,2); M=tab.xy_terms(:,3:16);
As=tab.xy_terms(:,17); Ac=tab.xy_terms(:,18);
arg=M*fa; comp=(t.^p).*(As.*sin(arg)+Ac.*cos(arg)); pt=(t.^(0:5)).';
X=AS2R*(tab.xyp(1,:)*pt + sum(comp(jxy==0))/1e6);
Y=AS2R*(tab.xyp(2,:)*pt + sum(comp(jxy==1))/1e6);
end

function s = s06(t, fa, x, y, tab)
AS2R=4.848136811095359935899141e-6; fa8=fa([1 2 3 4 5 7 8 14]); w=tab.s_poly(:);
Sb={tab.s0,tab.s1,tab.s2,tab.s3,tab.s4};
for k=1:5
    Sk=Sb{k}; if isempty(Sk), continue; end
    arg=Sk(:,1:8)*fa8; w(k)=w(k)+sum(Sk(:,9).*sin(arg)+Sk(:,10).*cos(arg));
end
poly=w(1)+(w(2)+(w(3)+(w(4)+(w(5)+w(6)*t)*t)*t)*t)*t;
s=poly*AS2R - x*y/2.0;
end

function sp = sp00(t), sp=-47e-6*t*4.848136811095359935899141e-6; end

function Q = c2ixys(x, y, s)
r2=x*x+y*y; if r2>0, e=atan2(y,x); else, e=0; end
d=atan(sqrt(r2/(1-r2)));
Q=rot(3,-(e+s))*rot(2,d)*rot(3,e);
end

function th = era00(dj1, dj2)
if dj1<dj2, d1=dj1; d2=dj2; else, d1=dj2; d2=dj1; end
D2PI=6.283185307179586476925287; t=d1+(d2-2451545.0); f=mod(d1,1.0)+mod(d2,1.0);
th=mod(D2PI*(f+0.7790572732640+0.00273781191135448*t),D2PI); if th<0, th=th+D2PI; end
end

function W = pom00(xp, yp, sp), W=rot(1,-yp)*rot(2,-xp)*rot(3,sp); end

function R = rot(ax, a)
c=cos(a); s=sin(a);
switch ax
    case 1, R=[1 0 0;0 c s;0 -s c];
    case 2, R=[c 0 -s;0 1 0;s 0 c];
    case 3, R=[c s 0;-s c 0;0 0 1];
end
end

% =================================================================== TIME
function [djm0,djm]=cal2jd(iy,im,id)
djm0=2400000.5; my=fix((im-14)/12); iypmy=iy+my;
djm=fix((1461*(iypmy+4800))/4)+fix((367*(im-2-12*my))/12)-fix((3*fix((iypmy+4900)/100))/4)+id-2432076;
end

function mjd = mjd_of(u)
[~,djm]=cal2jd(u(1),u(2),u(3)); mjd=djm+(u(4)*3600+u(5)*60+u(6))/86400;
end

function T = time_scales(u, dUT1, dAT)
DJ00=2451545.0; TTMTAI=32.184;
[djm0,djm]=cal2jd(u(1),u(2),u(3)); fd=(u(4)*3600+u(5)*60+u(6))/86400; utc2=djm+fd;
T.jd_ut1=[djm0, utc2+dUT1/86400];
T.jd_tt =[djm0, utc2+(dAT+TTMTAI)/86400];
T.t=((T.jd_tt(1)-DJ00)+T.jd_tt(2))/36525.0;
end

function U = utcmat(utc)
if isa(utc,'datetime')
    utc=utc(:); if isempty(utc.TimeZone), utc.TimeZone='UTC'; end, utc.TimeZone='UTC';
    U=[utc.Year utc.Month utc.Day utc.Hour utc.Minute utc.Second];
elseif isnumeric(utc)
    assert(size(utc,2)==6,'numeric UTC must be Nx6 = [Y Mo D H Mi S]'); U=utc;
else, error('utc must be datetime or Nx6 numeric'); end
end

% ============================================================ EOP: ORCHESTRATION
function EOP = get_eop(build, opt)
persistent CACHE
if ~isempty(CACHE) && strcmp(CACHE.key,build) && ~opt.force_reload, EOP=CACHE; return; end
P = eop_paths(opt);
% leap seconds (only if missing) + rapid finals (weekly)
fetch_one({'https://hpiers.obspm.fr/iers/bul/bulc/Leap_Second.dat', ...
           'https://data.iana.org/time-zones/data/leap-seconds.list'}, P.leap, Inf, opt);
fetch_one({'https://datacenter.iers.org/data/9/finals2000A.all', ...
           'https://datacenter.iers.org/products/eop/rapid/standard/finals2000A.all', ...
           'https://maia.usno.navy.mil/ser7/finals2000A.all'}, P.finals, 7, opt);
LS  = parse_leap(P.leap);
FIN = parse_finals(P.finals);
if strcmp(build,'A')
    E = FIN;
else
    urls = def_c04_urls(); if isfield(opt,'c04_url') && ~isempty(opt.c04_url), urls = opt.c04_url; end
    fetch_one(urls, P.c04, 7, opt);
    C04 = parse_c04(P.c04, opt);
    E = splice(C04, FIN, opt);
end
E = finalize_eop(E, LS, build);
CACHE = E; EOP = E;
if opt.verbose
    fprintf('[eop] build %s ready: %d rows, MJD %.0f..%.0f (%s)\n', build, numel(E.mjd), E.mjd(1), E.mjd(end), E.source);
end
end

function P = eop_paths(opt)
if nargin>=1 && isfield(opt,'data_dir') && ~isempty(opt.data_dir)
    here=opt.data_dir; if ~exist(here,'dir'), mkdir(here); end
else
    here=fileparts(mfilename('fullpath'));
end
P.leap  =fullfile(here,'Leap_Second.dat');
P.finals=fullfile(here,'finals2000A.all');
P.c04   =fullfile(here,'eopc04_20.1962-now');
end

function urls = def_c04_urls()
urls = {'https://hpiers.obspm.fr/iers/eop/eopc04/eopc04.1962-now', ...
        'https://hpiers.obspm.fr/iers/eop/eopc04/eopc04_IAU2000.1962-now', ...
        'https://datacenter.iers.org/data/latestVersion/EOP_20_C04_one_file_1962-now.txt'};
end

function E = finalize_eop(E, LS, build)
E.leap = LS;
E.dut1_tai = E.dut1 - arrayfun(@(m)leap_at(LS,m), E.mjd);   % leap-continuous UT1
E.key = build;
end

% ================================================================ EOP: FETCH
function fetch_one(urls, localfile, max_age_days, opt)
need = opt.force_download || ~isfile(localfile);
if ~need && isfinite(max_age_days), dd=dir(localfile); if (now-dd.datenum)>max_age_days, need=true; end, end
if ~need, if opt.verbose, fprintf('[eop] cache OK : %s\n', localfile); end, return; end
wo=weboptions('Timeout',60,'ContentType','text'); ok=false;
for i=1:numel(urls)
    try
        tmp=[localfile '.tmp']; websave(tmp,urls{i},wo);
        if isfile(localfile), delete(localfile); end, movefile(tmp,localfile,'f');
        if opt.verbose, fprintf('[eop] downloaded: %s <- %s\n', localfile, urls{i}); end
        ok=true; break;
    catch ME
        if opt.verbose, fprintf('[eop] FAIL %s (%s)\n', urls{i}, ME.message); end
    end
end
if ~ok
    if isfile(localfile), warning('%s: all sources failed; using existing file', mfilename);
    else, error('%s: could not obtain %s', mfilename, localfile); end
end
end

% ============================================================ EOP: LEAP PARSER
function LS = parse_leap(file)
txt=fileread(file); lines=regexp(txt,'\r?\n','split'); mjd=[]; dat=[];
isIANA = ~isempty(regexp(txt,'#[@$]','once')) || ~isempty(strfind(file,'leap-seconds.list')); %#ok<STREMP>
for i=1:numel(lines)
    L=strtrim(lines{i}); if isempty(L)||L(1)=='#', continue; end
    v=sscanf(L,'%f');
    if isIANA
        if numel(v)>=2, mjd(end+1,1)=v(1)/86400+15020; dat(end+1,1)=v(2); end %#ok
    else
        if numel(v)>=5, mjd(end+1,1)=v(1); dat(end+1,1)=v(5); end %#ok
    end
end
[mjd,ix]=sort(mjd); dat=dat(ix); LS=[mjd dat];
if isempty(LS), warning('leap parse empty; default'); LS=[41317 10;57754 37]; end
end

% ============================================================ EOP: FINALS PARSER
function E = parse_finals(file)
fid=fopen(file,'r'); assert(fid>0); raw=textscan(fid,'%s','Delimiter','\n','Whitespace',''); fclose(fid);
L=raw{1}; n=numel(L); mjd=nan(n,1); xp=mjd; yp=mjd; du=mjd; dX=mjd; dY=mjd;
for i=1:n
    s=L{i}; if numel(s)<68, continue; end
    mjd(i)=g(s,8,15); xa=g(s,19,27); ya=g(s,38,46); ua=g(s,59,68); dxa=g(s,98,106); dya=g(s,117,125);
    xb=g(s,135,144); yb=g(s,145,154); ub=g(s,155,165); dxb=g(s,166,175); dyb=g(s,176,185);
    if ~isnan(xb)&&~isnan(ub), xp(i)=xb; yp(i)=yb; du(i)=ub; dX(i)=pk(dxb,dxa); dY(i)=pk(dyb,dya);   %% x/y/UT1 and dX,dY: Bulletin B (final) if present, else Bulletin A
    else, xp(i)=xa; yp(i)=ya; du(i)=ua; dX(i)=dxa; dY(i)=dya; end
end
k=~isnan(mjd)&~isnan(xp)&~isnan(du);
E.mjd=mjd(k); E.xp=xp(k); E.yp=yp(k); E.dut1=du(k);
dXk=dX(k); dXk(isnan(dXk))=0; E.dX=dXk/1000; dYk=dY(k); dYk(isnan(dYk))=0; E.dY=dYk/1000;
E.source='finals2000A.all';
end
function v=g(s,a,b), if numel(s)>=b, v=str2double(s(a:b)); else, v=NaN; end, end
function v=pk(a,b), if ~isnan(a), v=a; else, v=b; end, end

% ============================================================ EOP: C04 PARSER
%  Format-adaptive. Handles the 2023 EOP 20 C04 layout (which carries an HOUR
%  column before MJD -> "YR MM DD HH MJD x y UT1-UTC dX dY ...") as well as the
%  older "YR MM DD MJD ..." layout. x, y, UT1-UTC are positionally fixed right
%  after MJD (true for every C04 variant); dX,dY are found from the header
%  labels, else a magnitude-guarded positional guess, else set 0 with a warning.
function E = parse_c04(file, opt)
%  x, y, UT1-UTC are read positionally (right after MJD, fixed in every C04 variant).
%  dX,dY are ALSO read from C04 (they INCLUDE Free Core Nutation, so they match
%  Orekit; finals dX,dY have FCN removed). The dX,dY columns are found by NAME in
%  the header, mapped to data columns via the MJD offset, with mas/arcsec auto-scale.
%  Override with opt.c04_native_dxdy=[iX iY] if a nonstandard header defeats it.
[R, hdr] = read_c04_numeric(file);
mc = locate_mjd_col(R);
assert(mc>0, '%s: could not locate an MJD column in C04 file %s', mfilename, file);
assert(size(R,2) >= mc+3, '%s: C04 rows too short (need x,y,UT1 after MJD)', mfilename);
E.mjd = R(:,mc); E.xp = R(:,mc+1); E.yp = R(:,mc+2); E.dut1 = R(:,mc+3);
% dX,dY: build C reads them from C04 (which INCLUDES Free Core Nutation, matching
% Orekit) instead of finals (where FCN is removed). Columns auto-detected from the
% header by name; a magnitude sanity-check guards against a wrong pick.
ix=[]; iy=[];
if isfield(opt,'c04_native_dxdy') && numel(opt.c04_native_dxdy)==2
    ix=opt.c04_native_dxdy(1); iy=opt.c04_native_dxdy(2);
elseif ~isempty(hdr)
    tok=regexp(regexprep(hdr,'^[#\s]+',''),'\s+','split');
    ht=lower(regexprep(tok,'[^a-z0-9]',''));
    hM=find(strcmp(ht,'mjd'),1);
    hX=find(ismember(ht,{'dx','dxcip','xcip','dx2000a'}),1);
    hY=find(ismember(ht,{'dy','dycip','ycip','dy2000a'}),1);
    if ~isempty(hM)&&~isempty(hX)&&~isempty(hY)
        off=mc-hM; ix=hX+off; iy=hY+off;   % map header token positions to DATA columns
    end
end
if isempty(ix)                % positional fallback (no header/name needed):
    % dX,dY = first adjacent column-pair after UT1 that is sub-mas AND changes sign
    % (excludes the always-positive LOD; xrt,yrt come after dX,dY so are not hit first)
    for j=mc+4:size(R,2)-1
        a=R(:,j); b=R(:,j+1); a=a(~isnan(a)); b=b(~isnan(b));
        if ~isempty(a)&&~isempty(b) && median(abs(a))<5 && median(abs(b))<5 && min(a)<0 && min(b)<0
            ix=j; iy=j+1; break
        end
    end
end
E.has_dxdy=false;
if ~isempty(ix)&&~isempty(iy) && ix>=1&&iy>=1 && size(R,2)>=max(ix,iy)
    vx=R(:,ix); vy=R(:,iy); mad=median(abs(vx(~isnan(vx))));
    if mad<5e-3            % already arcsec (dX ~1e-4..1e-3)
        E.dX=vx;       E.dY=vy;       E.has_dxdy=true;
    elseif mad<5          % mas -> arcsec
        E.dX=vx/1000;  E.dY=vy/1000;  E.has_dxdy=true;
    end
end
if E.has_dxdy
    if opt.verbose, fprintf('[eop] C04 dX,dY read from cols %d,%d (FCN included, matches Orekit)\n', ix, iy); end
else
    warning('%s: C04 dX,dY not resolved; using finals dX,dY (FCN removed). Set opt.c04_native_dxdy=[iX iY].', mfilename);
end
if opt.verbose
    ds='from finals (tail only)'; if E.has_dxdy, ds='from C04 (FCN incl)'; end
    fprintf('[eop] C04: %d rows, MJD=col%d, x/y/UT1=col%d-%d; dX,dY %s\n', ...
            numel(E.mjd), mc, mc+1, mc+3, ds);
end
E.source='eopc04';
end

function [R, hdr] = read_c04_numeric(file)
txt=fileread(file); lines=regexp(txt,'\r?\n','split'); V={}; hdr='';
for i=1:numel(lines)
    L=strtrim(lines{i}); if isempty(L), continue; end
    if ~isempty(regexp(L,'^[#A-Za-z]','once'))            % header/comment line
        if ~isempty(regexpi(L,'MJD|UT1|dX|Xcip','once')), hdr=L; end
        continue;
    end
    v=sscanf(L,'%f').'; if numel(v)>=7, V{end+1}=v; end %#ok
end
assert(~isempty(V),'%s: no numeric rows in %s', mfilename, file);
lens=cellfun(@numel,V); nm=mode(lens); V=V(lens==nm);
R=vertcat(V{:});
end

function mc = locate_mjd_col(R)
mc=0; best=inf;
for c=1:size(R,2)
    col=R(:,c);
    if all(col>=15000 & col<=99000)
        sc=abs(median(diff(col))-1);              % MJD steps by ~1 day
        if sc<best && sc<0.6, best=sc; mc=c; end
    end
end
if mc==0
    for c=1:size(R,2), if all(R(:,c)>=15000 & R(:,c)<=99000), mc=c; return; end, end
end
end

% ================================================================ EOP: SPLICE
function E = splice(C04, FIN, opt)
% x, y, UT1-UTC: C04 (final, ITRF2020) over its span, then finals rapid+predicted
% tail beyond C04's last epoch.  dX, dY: from finals across the whole grid
% (identical IAU2006/2000A pole offsets, sub-mas), unless C04 carried its own.
mmax=max(C04.mjd); t=FIN.mjd>mmax;
E.mjd =[C04.mjd; FIN.mjd(t)];
E.xp  =[C04.xp;  FIN.xp(t)];
E.yp  =[C04.yp;  FIN.yp(t)];
E.dut1=[C04.dut1;FIN.dut1(t)];
if isfield(C04,'has_dxdy') && C04.has_dxdy
    E.dX=[C04.dX; FIN.dX(t)]; E.dY=[C04.dY; FIN.dY(t)]; dsrc='C04-native';
else
    E.dX = interp1(FIN.mjd, FIN.dX, E.mjd, 'linear', 0);   % finals dX,dY on merged grid
    E.dY = interp1(FIN.mjd, FIN.dY, E.mjd, 'linear', 0);   % 0 where finals has no data (pre-1992)
    dsrc='finals';
end
[E.mjd,ix]=sort(E.mjd); E.xp=E.xp(ix); E.yp=E.yp(ix); E.dut1=E.dut1(ix); E.dX=E.dX(ix); E.dY=E.dY(ix);
E.source=sprintf('C04(<=%.0f)+finals tail; dX,dY=%s', mmax, dsrc);
if opt.verbose, fprintf('[eop] spliced: C04 x/y/UT1 to MJD %.0f, +%d finals rows; dX,dY from %s\n', mmax, sum(t), dsrc); end
end

% ================================================================ EOP: INTERP
function [dUT1,xp,yp,dX,dY,dAT,flag] = eop_interp(mjd, E)
% Build C: N-point Lagrange EOP interpolation + UT1 zonal-tide regularisation
% (RG_ZONT2) + full sub-daily tidal (ocean + PM + UT1 libration).
% NPTS sets the interpolation order: 2=linear, 4=cubic (default), 6=quintic.
% Match NPTS to Orekit's EOP interpolation to minimise the off-grid difference.
NPTS = 4;
AS2R=4.848136811095359935899141e-6; m=E.mjd; flag=0;
if mjd<m(1)||mjd>m(end), flag=1; warning('MJD %.3f outside EOP table [%.1f %.1f]; extrapolating',mjd,m(1),m(end)); end
j=find(m<=mjd,1,'last'); if isempty(j), j=1; end
half=floor((NPTS-1)/2); i0=min(max(j-half,1),numel(m)-NPTS+1); idx=i0:i0+NPTS-1;  mw=m(idx);
[dxp_t,dyp_t,dut1_t]=tidal_eop(mjd);           % sub-daily: uas,uas,us
xp=(lagr(mw,E.xp(idx),mjd)+dxp_t*1e-6)*AS2R;    % arcsec(+uas) -> rad
yp=(lagr(mw,E.yp(idx),mjd)+dyp_t*1e-6)*AS2R;
dX= lagr(mw,E.dX(idx),mjd)*AS2R;
dY= lagr(mw,E.dY(idx),mjd)*AS2R;
dAT=leap_at(E.leap,mjd);
if ~isfield(E,'zonal') || E.zonal               % IERS UT1 zonal-tide regularisation (RG_ZONT2)
    zg=arrayfun(@tidal_ut1_zonal, mw);
    ut1r=lagr(mw, E.dut1_tai(idx)-zg, mjd);
    dUT1=ut1r + tidal_ut1_zonal(mjd) + dut1_t*1e-6 + dAT;
else                                            % raw interpolation (match a tool that does not regularise, e.g. Orekit)
    ut1r=lagr(mw, E.dut1_tai(idx), mjd);
    dUT1=ut1r + dut1_t*1e-6 + dAT;
end
end

function y=lagr(x,f,xi)                          % N-point Lagrange
n=numel(x); y=0;
for a=1:n
    L=1; for b=1:n, if b~=a, L=L*(xi-x(b))/(x(a)-x(b)); end, end
    y=y+f(a)*L;
end
end

function d = leap_at(LS, mjd)
j=find(LS(:,1)<=mjd,1,'last'); if isempty(j), d=LS(1,2); else, d=LS(j,2); end
end
