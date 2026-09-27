function raw_eop_audit(orekit_data_dir)
%% RAW_EOP_AUDIT  Unbiased ground-truth: read the RAW dX,dY bytes for MJD 60735
%  directly from each file, independent of the builds and Orekit.
%    raw_eop_audit                      % audits MATLAB's finals + C04 files
%    raw_eop_audit('C:\path\orekit-data') % ALSO reads Orekit's own EOP file
%  IERS column spec (finals2000A.all): Bull A dX 98-106, dY 117-125 (mas);
%                                      Bull B dX 166-175, dY 176-185 (mas).
mjt=60735;  here=fileparts(which('eci2ecef_A'));
fprintf('==== RAW dX,dY at MJD %d (2025-03-01), read straight from the files ====\n\n', mjt);

%% (1) MATLAB's finals2000A.all
ff=fullfile(here,'finals2000A.all');
fprintf('--- finals2000A.all (%s) ---\n', tf(isfile(ff)));
if isfile(ff)
    s=find_line(ff, @(L) num(L,8,15), mjt);
    if ~isempty(s)
        fprintf('  raw line (chars 90..185):\n   |%s|\n', s(min(90,end):min(185,end)));
        fprintf('  Bull A: dX(98-106)=%s mas   dY(117-125)=%s mas\n', fld(s,98,106), fld(s,117,125));
        fprintf('  Bull B: dX(166-175)=%s mas  dY(176-185)=%s mas\n', fld(s,166,175), fld(s,176,185));
    else, fprintf('  MJD %d not found.\n', mjt); end
end

%% (2) MATLAB's EOP 20 C04
cf=fullfile(here,'eopc04_20.1962-now'); if ~isfile(cf), d=dir(fullfile(here,'eopc04*')); if ~isempty(d), cf=fullfile(d(1).folder,d(1).name); end, end
fprintf('\n--- EOP 20 C04 (%s) ---\n', tf(isfile(cf)));
dump_c04(cf, mjt);

%% (3) Orekit's OWN EOP file (to prove stale-vs-wrong-read)
if nargin>=1 && ~isempty(orekit_data_dir)
    fprintf('\n--- Orekit orekit-data EOP files ---\n');
    for pat={'finals2000A.all','finals2000A.daily','eopc04*','EOP_20_C04*'}
        d=dir(fullfile(orekit_data_dir,'**',pat{1}));
        for k=1:numel(d)
            fp=fullfile(d(k).folder,d(k).name);
            fprintf('  file: %s\n', fp);
            if ~isempty(regexpi(d(k).name,'finals','once'))
                s=find_line(fp,@(L)num(L,8,15),mjt);
                if ~isempty(s), fprintf('    finals Bull A dX(98-106)=%s  dY(117-125)=%s ; Bull B dX(166-175)=%s\n', fld(s,98,106),fld(s,117,125),fld(s,166,175)); end
            else, dump_c04(fp, mjt); end
        end
    end
end

fprintf('\n==== READ THE NUMBERS ABOVE: whichever value the FILE contains is the truth. ====\n');
fprintf('If MATLAB''s files say ~0.38 and Orekit''s file also says ~0.38 -> Orekit is mis-reading.\n');
fprintf('If Orekit''s file says ~0.98 -> Orekit''s data is stale; update orekit-data or point it\n');
fprintf('at MATLAB''s current file. If MATLAB''s files say ~0.98 -> then MATLAB is mis-reading.\n');
end

%% helpers
function dump_c04(cf, mjt)
if ~isfile(cf), fprintf('  (file not found)\n'); return; end
Lc=regexp(fileread(cf),'\r?\n','split');
for i=1:numel(Lc), L=strtrim(Lc{i}); if isempty(L)||~isempty(regexp(L,'^[#A-Za-z]','once')), continue; end
    v=sscanf(L,'%f').'; if numel(v)<7, continue; end
    mc=find(v>=15000 & v<=99000,1); if isempty(mc)||abs(v(mc)-mjt)>0.5, continue; end
    fprintf('  numeric row: %s\n', num2str(v,'%.5g '));
    fprintf('  MJD=col%d(%.0f)  x=col%d(%.4f)  y=col%d(%.4f)  UT1=col%d(%.5f)\n', mc,v(mc),mc+1,v(mc+1),mc+2,v(mc+2),mc+3,v(mc+3));
    fprintf('  -> cols %d,%d (next small sign-changing pair) = dX=%.4f dY=%.4f (units as in file)\n', mc+4,mc+5,v(mc+4),v(mc+5));
    return
end
fprintf('  MJD %d not found in C04 (may be beyond its span -> finals tail).\n', mjt);
end
function s=find_line(f,mjfun,mjt), s=''; L=regexp(fileread(f),'\r?\n','split');
for i=1:numel(L), if numel(L{i})>=15, m=mjfun(L{i}); if ~isnan(m)&&abs(m-mjt)<0.5, s=L{i}; return; end, end, end, end
function v=num(L,a,b), if numel(L)>=b, v=str2double(L(a:b)); else, v=NaN; end, end
function t=fld(s,a,b), if numel(s)>=b, t=strtrim(s(a:b)); if isempty(t), t='<blank>'; end, else, t='<short>'; end, end
function t=tf(b), if b, t='found'; else, t='NOT FOUND'; end, end
