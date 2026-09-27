%% ============================================================================
%  compare_density.m  —  DENSITY vs REALITY  (SCRIPT: edit top, run, get results)
%  Compares model thermospheric density against TU Delft MEASURED density along
%  the real track, for the density models you select. Pure comparison (reports
%  the gap; no bias correction). Repeatable drawing is in functions.
%% ============================================================================
setup_paths;

% ------------------------------- SETTINGS -----------------------------------
SAT     = 'GOCE';                    % has_tudelft satellite (GOCE/CHAMP/GRACE-FO/SWARM)
START   = '2013-06-01';
STOP    = '2013-06-01';
THIN    = 60;                        % keep every Nth 10 s sample (60 -> 10 min)
MODELS  = {'exponential','nrlmsise','jb2008','dtm2020'};  % density models to compare
SAVE    = true;
% ----------------------------------------------------------------------------

fprintf('\n===== DENSITY vs REALITY: %s %s..%s =====\n', SAT, START, STOP);
cat = sat.catalog(SAT); assert(cat.has_tudelft, sprintf('%s has no TU Delft density',SAT));

% (1) measured density + geodetic track (no login)
T = data.tudelft_density(cat.tudelft_name, START, STOP);
idx = 1:THIN:height(T);
dt=T.DateTime(idx); alt=T.Altitude_m(idx); lat=T.Latitude_deg(idx); lon=T.Longitude_deg(idx); rho_meas=T.Density_kgm3(idx);
n=numel(idx);
fprintf('measured: %d points, alt %.1f-%.1f km\n', n, min(alt)/1000, max(alt)/1000);

% (2) real space weather for the window
SW = data.spaceweather(START, datestr(datenum(STOP)+1,'yyyy-mm-dd'));

% (3) evaluate each model at every measured point
models=struct('name',{},'rho',{},'ratio_med',{},'gap_pct',{},'rms_logerr',{});
for m=1:numel(MODELS)
    md=MODELS{m}; rho=nan(n,1); ok=true;
    for k=1:n
        u=datevec(dt(k)); u=u(1:6);
        geo=struct('alt_km',alt(k)/1000,'lat_deg',lat(k),'lon_deg',lon(k), ...
                   'lst_h',mod(u(4)+u(5)/60+lon(k)/15,24),'doy',floor(datenum(u)-datenum(u(1),1,0)),'utc',u);
        try
            if strcmp(md,'exponential'), sw=struct(); else, sw=atmos.spaceweather(u,struct('table',SW)); end
            atm=atmos.provider(md,geo,sw); rho(k)=atm.rho;
        catch, ok=false; break; end
    end
    if ~ok, fprintf('  %-12s: not available - skipped\n',md); continue; end
    r=rho./rho_meas; r=r(isfinite(r)); le=log(rho)-log(rho_meas); le=le(isfinite(le));
    models(end+1)=struct('name',md,'rho',rho,'ratio_med',median(r),'gap_pct',100*(median(r)-1),'rms_logerr',sqrt(mean(le.^2)));
    fprintf('  %-12s: median(model/meas)=%.3f  gap=%+.1f%%  RMS(log-err)=%.3f\n', md, models(end).ratio_med, models(end).gap_pct, models(end).rms_logerr);
end

% (4) figures + save
if SAVE && ~isempty(models)
    outdir=save_results(sprintf('density_%s_%s',SAT,START));
    if isdatetime(dt), tm=minutes(dt-dt(1)); else, tm=(datenum(dt)-datenum(dt(1)))*1440; end
    fd=figure('Color','w','Name','density'); semilogy(tm,rho_meas,'k','LineWidth',1.6); hold on
    for i=1:numel(models), semilogy(tm,models(i).rho,'LineWidth',1.1); end
    grid on; box on; xlabel('time [min]'); ylabel('\\rho [kg/m^3]'); legend(['measured',{models.name}],'Interpreter','none');
    title(sprintf('%s density: measured vs models',SAT)); print(fd,'-dpng','-r150',fullfile(outdir,'density_timeseries.png'));
    fr=figure('Color','w','Name','ratio'); hold on
    for i=1:numel(models), plot(tm,models(i).rho./rho_meas,'LineWidth',1.1); end
    yline(1,'k--'); grid on; box on; xlabel('time [min]'); ylabel('model / measured'); legend({models.name},'Interpreter','none');
    title('Density ratio (1.0 = perfect)'); print(fr,'-dpng','-r150',fullfile(outdir,'density_ratio.png'));
    % inputs_report DOES NOT EXIST ANYWHERE IN THIS TREE, and the bare `catch, end`
    % around it meant nobody ever found out: the call threw "Undefined function"
    % on every single run and the error was swallowed. A swallowing catch around a
    % function that does not exist is indistinguishable from a feature that works.
    % The provenance report is what this wanted, and that one is real:
    if exist('validation.provenance','file') || exist('+validation/provenance.m','file')
        try
            PRV = validation.provenance(config.defaultConfig(), struct(), struct(), SAT);
            fid_ = fopen(fullfile(outdir,'provenance.txt'),'w');
            if fid_ > 0
                fprintf(fid_, 'provenance for %s  %s..%s\n', SAT, START, STOP);
                fprintf(fid_, 'measured %d | fetched %d | ASSUMED %d | MISSING %d\n\n', ...
                        PRV.n.measured, PRV.n.fetched, PRV.n.assumed, PRV.n.missing);
                for q_ = 1:numel(PRV.items)
                    fprintf(fid_, '  %-34s %-12s %s\n', PRV.items(q_).name, ...
                            PRV.items(q_).class, PRV.items(q_).source);
                end
                fclose(fid_);
            end
        catch err_
            % report it, do not swallow it -- that is how the old bug hid
            fprintf('  [provenance report skipped: %s]\n', err_.message);
        end
    end
    fid=fopen(fullfile(outdir,'density_stats.csv'),'w'); fprintf(fid,'model,median_ratio,gap_pct,rms_log_error\n');
    for i=1:numel(models), fprintf(fid,'%s,%.4f,%.2f,%.4f\n',models(i).name,models(i).ratio_med,models(i).gap_pct,models(i).rms_logerr); end
    fclose(fid);
    fprintf('\nsaved to: %s\n', outdir);
end
fprintf('\nRead it: ratio near 1.0 / small gap%% = model matches reality (pure comparison).\n');
