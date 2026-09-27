function fld = loadGFC(fname, maxdeg)
%GRAV.LOADGFC  Read an ICGEM ".gfc" spherical-harmonic gravity field.
%   fld = grav.loadGFC(fname[, maxdeg])
%   Parses the standard ICGEM ASCII format (EIGEN, GOCO, GGM, ITU_GRACE, EGM2008
%   .gfc downloads).  Returns the field in the form the native engine expects:
%     fld.mu    GM [m^3/s^2]           (from 'earth_gravity_constant')
%     fld.Re    reference radius [m]   (from 'radius')
%     fld.nmax  maximum degree kept
%     fld.Cbar  (nmax+1 x nmax+1) fully-normalised cosine coeffs, Cbar(n+1,m+1)
%     fld.Sbar  (nmax+1 x nmax+1) fully-normalised sine coeffs
%     fld.name  the model name
%   Coefficients are read from the 'gfc' (static) lines; time-variable 'gfct'
%   trend/periodic lines are IGNORED (static field only).  Zonal C(n,0) are kept
%   with their file sign (ICGEM stores the negative, unnormalised-consistent
%   convention already baked into the normalised value).
%
%   EXAMPLE
%     fld = grav.loadGFC('gravity_data/EGM2008.gfc', 70);
%     cfg.gravityField.field = 'gravity_data/EGM2008.gfc';
%
%   See also GRAV.DEFAULTFIELD, GRAV.SPHERICALHARMONIC.
    if nargin<2||isempty(maxdeg), maxdeg=Inf; end
    fid=fopen(fname,'r');
    if fid<0, error('grav:loadGFC','cannot open "%s"',fname); end
    mu=NaN; Re=NaN; name='(gfc)'; inHeader=true; nmaxHdr=Inf;
    C=[]; S=[];
    while true
        ln=fgetl(fid); if ~ischar(ln), break; end
        t=strtrim(ln); if isempty(t), continue; end
        tok=strsplit(t);
        key=lower(tok{1});
        if inHeader
            switch key
                case 'earth_gravity_constant', mu=str2double(tok{2});
                case 'radius',                 Re=str2double(tok{2});
                case 'max_degree',             nmaxHdr=str2double(tok{2});
                case 'modelname',              if numel(tok)>=2, name=tok{2}; end
                case 'end_of_head',            inHeader=false;
                                               N=min(maxdeg,nmaxHdr);
                                               if isinf(N), N=360; end
                                               C=zeros(N+1); S=zeros(N+1);
            end
            continue;
        end
        % coefficient lines: 'gfc'|'gfct'  n  m  Cnm  Snm ...
        if strcmp(key,'gfc') || strcmp(key,'gfct')
            n=str2double(tok{2}); m=str2double(tok{3});
            if n>maxdeg, continue; end
            if isempty(C)                        % no end_of_head seen: lazy alloc
                N=min(maxdeg,360); C=zeros(N+1); S=zeros(N+1);
            end
            if n+1>size(C,1)                     % grow if needed
                C(n+1,n+1)=0; S(n+1,n+1)=0;
            end
            cv=str2num_safe(tok{4}); sv=str2num_safe(tok{5}); %#ok
            if strcmp(key,'gfc')                 % static only (skip gfct trend rows)
                C(n+1,m+1)=cv; S(n+1,m+1)=sv;
            end
        end
    end
    fclose(fid);
    nmax=size(C,1)-1;
    fld.mu=mu; fld.Re=Re; fld.nmax=nmax; fld.Cbar=C; fld.Sbar=S; fld.name=name;
end

function x=str2num_safe(s)
    s=strrep(s,'D','E'); s=strrep(s,'d','E');   % Fortran exponent -> MATLAB
    x=str2double(s);
end
