function fld = gravLoad(gcfg)
%OP.GRAVLOAD  Resolve the gravity field struct from config.
%   gcfg.field : one of
%       'default'                    embedded zonal J2..J6 (offline, no file)
%       'EGM2008' | 'EIGEN-6C4' | .. a KNOWN model name -> data.gravity fetches &
%                                    caches the ICGEM .gfc once, then loadGFC
%       '/path/to/model.gfc'         an explicit .gfc file (offline)
%   gcfg.degree : max degree to keep from the file (default 60)
%   Returns fld with .mu .Re .Cbar .Sbar .nmax .J .name (+ .matfile for toolbox).
%
%   So switching between EGM2008 and EIGEN-6C4 is one line:
%       cfg.gravityField = struct('field','EGM2008','degree',70);
%       cfg.gravityField = struct('field','EIGEN-6C4','degree',70);
    if nargin<1||isempty(gcfg), gcfg=struct(); end
    src = getf(gcfg,'field','default');
    % Degree is a PHYSICS decision, not a formatting detail: 60 vs 20 vs 4 changes
    % the answer and the runtime. Defaulting to 60 in silence means a caller who
    % forgot to set it gets a degree-60 field and never knows. Say it out loud.
    if isfield(gcfg,'degree') && ~isempty(gcfg.degree)
        deg = gcfg.degree;
    else
        deg = 60;
        warning('op:gravLoad:degreeDefault', ...
          ['no cfg.gravityField.degree given -- defaulting to %d. That is a physics ' ...
           'choice being made for you: set it explicitly (20 is plenty for a short ' ...
           'drag study; 70 needs field=''EGM2008'').'], deg);
    end
    if strcmpi(src,'default')
        fld = grav.defaultField();
    elseif exist(src,'file')==2
        fld = grav.loadGFC(src, deg);                 % explicit path
    else
        gfc = data.gravity(src);                      % model name -> fetch/cache .gfc
        fld = grav.loadGFC(gfc, deg);
    end
    % expose zonal Jn for the fast j2..j6 path
    nz = min(fld.nmax,6); J=zeros(max(nz-1,0),1);
    for n=2:nz, J(n-1) = -fld.Cbar(n+1,1)*sqrt(2*n+1); end
    fld.J = J;
    if isfield(gcfg,'matfile'), fld.matfile=gcfg.matfile; end
end
function val=getf(s,f,d), if isfield(s,f)&&~isempty(s.(f)), val=s.(f); else, val=d; end, end
