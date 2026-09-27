function txt = report(cfg, W, sol, outfile)
%CONFIG.REPORT  Print/save EVERY decision the engine made -- including the ones
%   you did not set. Nothing about a run should be invisible.
%
%   config.report(cfg)                 -> print what YOU asked for + the defaults
%   config.report(cfg, W)              -> also the resolved world (field, omega...)
%   config.report(cfg, W, sol)         -> also what the integrator actually DID
%   config.report(cfg, W, sol, file)   -> also write it next to your results
%   txt = config.report(...)           -> return the text
%
%   Read it as: "LEFT = the knob, RIGHT = the value in force, (default) = it was
%   not set by you". Anything marked (default) is a decision made on your behalf.
    if nargin<2, W=[]; end
    if nargin<3, sol=[]; end
    L = {};
    add = @(varargin) []; %#ok<NASGU>
    L{end+1} = sprintf('================ RUN DECISIONS ================');
    L{end+1} = sprintf('epoch (UTC)        : %04d-%02d-%02d %02d:%02d:%06.3f', cfg.epoch);
    if isscalar(cfg.tspan), L{end+1}=sprintf('tspan              : 0 .. %g s (%.1f min)', cfg.tspan, cfg.tspan/60);
    else, L{end+1}=sprintf('tspan              : %g .. %g s', cfg.tspan(1), cfg.tspan(2)); end
    L{end+1} = sprintf('|r0|, |v0|         : %.3f km, %.4f km/s', norm(cfg.r0)/1000, norm(cfg.v0)/1000);

    % ---- spacecraft ----
    sc = getf(cfg,'spacecraft',struct());
    L{end+1} = sprintf('-- spacecraft --');
    L{end+1} = sprintf('  mass / Aref / Cd / Cr : %s kg / %s m^2 / %s / %s', ...
        num(getf(sc,'mass',[])), num(getf(sc,'Aref',[])), num(getf(sc,'Cd',[])), num(getf(sc,'Cr',[])));

    % ---- forces ----
    L{end+1} = sprintf('-- forces (on/off + model) --');
    F = getf(cfg,'forces',struct()); fn = fieldnames(F);
    for i=1:numel(fn)
        f = F.(fn{i}); on = isfield(f,'on') && f.on;
        extra = '';
        if isfield(f,'model'), extra=[' model=' f.model]; end
        if isfield(f,'degree'), extra=[extra sprintf(' degree=%d order=%d', f.degree, getf(f,'order',f.degree))]; end
        if isfield(f,'atmos'),  extra=[extra ' atmos=' f.atmos]; end
        if isfield(f,'Cr'),     extra=[extra sprintf(' Cr=%g', f.Cr)]; end
        L{end+1} = sprintf('  %-11s %-3s%s', fn{i}, onoff(on), extra); %#ok
    end
    gf = getf(cfg,'gravityField',struct());
    L{end+1} = sprintf('  gravityField field=%s requested degree=%s', ...
        getf(gf,'field','default'), num(getf(gf,'degree',[])));
    if ~isempty(W) && isfield(W,'grav')
        L{end+1} = sprintf('    -> LOADED field max degree = %s  %s', num(getf(W.grav,'nmax',[])), ...
            truncNote(gf, W.grav));
    end

    % ---- integrator: the defaults people never see ----
    ig = getf(cfg,'integrator',struct()); m = lower(getf(ig,'method','rk78'));
    L{end+1} = sprintf('-- integrator --');
    L{end+1} = sprintf('  method             : %s (%s)', m, kindOf(m));
    if any(strcmp(kindOf(m),{'adaptive'}))
        L{end+1} = sprintf('  rtol / atol        : %s / %s', dflt(ig,'rtol',1e-9), dflt(ig,'atol',1e-12));
        L{end+1} = sprintf('  h0 / hmin / hmax   : %s / %s / %s', dflt(ig,'h0','min(span/100,10)'), ...
                            dflt(ig,'hmin',1e-6), dflt(ig,'hmax','capped at output spacing'));
        L{end+1} = sprintf('  facmin/facmax/maxsteps : %s / %s / %s', dflt(ig,'facmin',0.2), ...
                            dflt(ig,'facmax',5), dflt(ig,'maxsteps',2e6));
    elseif strcmp(kindOf(m),'fixed')
        L{end+1} = sprintf('  h (fixed step)     : %s', dflt(ig,'h','min(30, span/1000)'));
        L{end+1} = sprintf('    NOTE: keep h <= the output spacing or the dense output degrades.');
    else
        L{end+1} = sprintf('  rtol / atol        : %s / %s   (MATLAB ODE suite)', dflt(ig,'rtol',1e-9), dflt(ig,'atol',1e-12));
        L{end+1} = sprintf('    reported AT the output times via the solver''s own interpolant.');
    end

    % ---- frame / time ----
    fr = getf(cfg,'frame',struct());
    L{end+1} = sprintf('-- frame / time --');
    L{end+1} = sprintf('  build              : %s (%s)', getf(fr,'build','gmst'), frameNote(getf(fr,'build','gmst')));
    if ~isempty(W) && isfield(W,'omega_eci')
        w = W.omega_eci; tilt = 0;
        if norm(w)>0, tilt = asin(min(1,norm(w(1:2))/norm(w)))*206265; end
        L{end+1} = sprintf('  Earth rate (ECI)   : |w|=%.9e rad/s, tilt off z = %.1f arcsec', norm(w), tilt);
        L{end+1} = sprintf('    (taken from dCt/dt -- NOT assumed along z; tilt is the CIP offset)');
    end

    % ---- output ----
    ot = getf(cfg,'output',struct());
    if isfield(ot,'times') && ~isempty(ot.times)
        d = unique(round(diff(ot.times(:))*1e6)/1e6);
        L{end+1} = sprintf('-- output --');
        L{end+1} = sprintf('  %d times, spacing %s s', numel(ot.times), mat2str(d(1:min(3,end))));
    elseif isfield(ot,'dt')
        L{end+1} = sprintf('-- output --');
        L{end+1} = sprintf('  fixed dt = %g s', ot.dt);
    end

    % ---- what actually happened ----
    if ~isempty(sol)
        L{end+1} = sprintf('-- what the integrator actually did --');
        L{end+1} = sprintf('  nodes used         : %s', num(getf(sol,'nodes',[])));
        L{end+1} = sprintf('  wall time          : %.3f s', getf(sol,'walltime',NaN));
    end
    L{end+1} = sprintf('==============================================');

    txt = strjoin(L, sprintf('\n'));
    if nargout==0 || nargin>=4, fprintf('%s\n', txt); end
    if nargin>=4 && ~isempty(outfile)
        fid=fopen(outfile,'w'); if fid>0, fprintf(fid,'%s\n',txt); fclose(fid); end
    end
end

function s=onoff(b), if b, s='ON'; else, s='off'; end, end
function s=num(v)
    if isempty(v), s='(unset)'; elseif ischar(v), s=v; else, s=sprintf('%g',v); end
end
function s=dflt(st,f,d)
    if isfield(st,f) && ~isempty(st.(f)), s=sprintf('%g  [set]', st.(f));
    elseif ischar(d), s=sprintf('%s  (default)', d);
    else, s=sprintf('%g  (default)', d); end
end
function k=kindOf(m)
    if any(strcmp(m,{'rk4','nystrom4','rk6luther','gaussjackson8'})), k='fixed';
    elseif any(strncmp(m,{'ode'},3)), k='MATLAB ODE suite';
    else, k='adaptive'; end
end
function s=frameNote(b)
    switch lower(b)
        case 'gmst', s='fast, offline, no EOP';
        otherwise,   s='IAU 2006/2000A, downloads IERS EOP';
    end
end
function s=truncNote(gf, grav)
    req = []; if isfield(gf,'degree'), req = gf.degree; end
    nm = []; if isfield(grav,'nmax'), nm = grav.nmax; end
    if ~isempty(req) && ~isempty(nm) && req > nm
        s = sprintf('<-- TRUNCATED from %d! load EGM2008 for higher degree', req);
    else, s = ''; end
end
function val=getf(s,f,d), if isfield(s,f)&&~isempty(s.(f)), val=s.(f); else, val=d; end, end
