%% CHECK_DX  dX,dY source audit (pure MATLAB). Shows what each build used and the
%  RAW EOP 20 C04 dX (= what Orekit uses, FCN included). finals dX has FCN removed.
AS2R=4.848136811095359935899141e-6; r2mas=@(r) r/AS2R*1e3;
utc=[2025 3 1 12 0 0]; mjt=mjd_of(utc);
fprintf('==== dX SOURCE AUDIT @ %04d-%02d-%02d %02d:%02d UTC (MJD %.2f) ====\n\n', utc(1:5), mjt);

%% ---- what each build used ----
[~,~,iA]=eci2ecef_A(utc,struct('verbose',false));
[~,~,iB]=eci2ecef_B(utc,struct('verbose',false));
[~,~,iC]=eci2ecef_C(utc,struct('verbose',false));
fprintf('build A used  : dX=%+8.4f mas  dY=%+8.4f mas   (finals, FCN removed)\n', r2mas(iA.dX), r2mas(iA.dY));
fprintf('build B used  : dX=%+8.4f mas  dY=%+8.4f mas   (should = C04)\n', r2mas(iB.dX), r2mas(iB.dY));
fprintf('build C used  : dX=%+8.4f mas  dY=%+8.4f mas   (should = C04)\n', r2mas(iC.dX), r2mas(iC.dY));

%% ---- RAW EOP 20 C04 dX,dY for this MJD (exact file + positional fallback) ----
cfolder=fileparts(which('eci2ecef_C'));
cf=fullfile(cfolder,'eopc04_20.1962-now');
if ~isfile(cf), dd=dir(fullfile(cfolder,'eopc04*')); if ~isempty(dd), cf=fullfile(dd(1).folder,dd(1).name); end, end
if isfile(cf)
    Lc=regexp(fileread(cf),'\r?\n','split'); rows={};
    for i=1:numel(Lc), L=strtrim(Lc{i}); if isempty(L)||~isempty(regexp(L,'^[#A-Za-z]','once')), continue; end
        v=sscanf(L,'%f').'; if numel(v)>=7, rows{end+1}=v; end, end %#ok
    lens=cellfun(@numel,rows); R=vertcat(rows{lens==mode(lens)});
    mc=0; for c=1:size(R,2), if all(R(:,c)>=15000&R(:,c)<=99000)&&abs(median(diff(R(:,c)))-1)<0.6, mc=c; break; end, end
    ix=[]; for j=mc+4:size(R,2)-1, a=R(:,j); b=R(:,j+1);
        if median(abs(a))<5 && median(abs(b))<5 && min(a)<0 && min(b)<0, ix=j; iy=j+1; break; end, end
    if ~isempty(ix)
        [~,ri]=min(abs(R(:,mc)-floor(mjt))); sc=1; if median(abs(R(:,ix)))<5e-3, sc=1e3; end % arcsec->mas
        fprintf('\nEOP 20 C04    : dX=%+8.4f mas  dY=%+8.4f mas   (FCN INCLUDED = Orekit)  [cols %d,%d, MJD %.0f]\n', R(ri,ix)*sc, R(ri,iy)*sc, ix, iy, R(ri,mc));
        if R(ri,mc) < floor(mjt)-1, fprintf('  NOTE: C04 ends at MJD %.0f < target -> target is in the finals TAIL (update C04 file).\n', R(end,mc)); end
    else, fprintf('\nEOP 20 C04    : dX,dY columns not found positionally.\n'); end
else, fprintf('\nEOP 20 C04 file not found (looked for eopc04_20.1962-now next to eci2ecef_C).\n'); end

fprintf('\nExpected: build B,C dX == EOP 20 C04 dX (FCN included) == Orekit.\n');
fprintf('          build A dX == finals (FCN removed), smaller by the FCN.\n');

function mjd = mjd_of(u)
iy=u(1);im=u(2);id=u(3); my=fix((im-14)/12); k=iy+my;
mjd=fix((1461*(k+4800))/4)+fix((367*(im-2-12*my))/12)-fix((3*fix((k+4900)/100))/4)+id-2432076 + (u(4)+u(5)/60)/24;
end
