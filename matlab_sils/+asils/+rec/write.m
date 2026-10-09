function dir_out = write(rec, dir_out)
%ASILS.REC.WRITE  Save a run as adcs-rec/1: channels.csv + manifest.json (+ rec.mat).
%   Channel names carry their units. Every other tool (the Python report,
%   a colleague's MATLAB) reads these files; nothing re-runs.
    if nargin < 2 || isempty(dir_out)
        dir_out = fullfile(asils.util.root(), 'store', 'results', rec.P.id);
    end
    if ~exist(dir_out, 'dir'), mkdir(dir_out); end
    C = {'t_s', rec.t; ...
         'q_x', rec.q(1,:); 'q_y', rec.q(2,:); 'q_z', rec.q(3,:); 'q_w', rec.q(4,:); ...
         'w_x_degps', rec.w(1,:)*180/pi; 'w_y_degps', rec.w(2,:)*180/pi; 'w_z_degps', rec.w(3,:)*180/pi; ...
         'rate_degps', rec.rate; 'ape_3ax_deg', rec.ape_3ax; 'ape_los_deg', rec.ape_los; ...
         'ake_3ax_deg', rec.ake_3ax; 'ake_los_deg', rec.ake_los; 'rks_degps', rec.rks; ...
         'mode', rec.mode; 'm_x_Am2', rec.m(1,:); 'm_y_Am2', rec.m(2,:); 'm_z_Am2', rec.m(3,:)};
    names = {'gg', 'aero', 'srp', 'mag'};
    ax = 'xyz';
    for i = 1:4
        for a = 1:3
            C(end+1, :) = {sprintf('tau_%s_%s_Nm', names{i}, ax(a)), rec.tau_dist(3*(i-1)+a, :)}; %#ok<AGROW>
        end
    end
    for a = 1:3
        C(end+1,:) = {sprintf('tau_mtq_%s_Nm', ax(a)), rec.tau_mtq(a,:)}; %#ok<AGROW>
        C(end+1,:) = {sprintf('tau_rw_%s_Nm', ax(a)), rec.tau_rw(a,:)}; %#ok<AGROW>
        C(end+1,:) = {sprintf('B_%s_T', ax(a)), rec.B(a,:)}; %#ok<AGROW>
        C(end+1,:) = {sprintf('r_%s_m', ax(a)), rec.r(a,:)}; %#ok<AGROW>
        C(end+1,:) = {sprintf('sun_%s', ax(a)), rec.sun_eci(a,:)}; %#ok<AGROW>
    end
    for k = 1:size(rec.h_w, 1)
        C(end+1,:) = {sprintf('h_w%d_Nms', k), rec.h_w(k,:)}; %#ok<AGROW>
    end
    if isfield(rec, 'delta') && any(isfinite(rec.delta(:)))
        for k = 1:size(rec.delta, 1), C(end+1,:) = {sprintf('gimbal%d_rad', k), rec.delta(k,:)}; end %#ok<AGROW>
    end
    if isfield(rec, 'tau_rcs')
        for a = 1:3, C(end+1,:) = {sprintf('tau_rcs_%s_Nm', ax(a)), rec.tau_rcs(a,:)}; end %#ok<AGROW>
        C = [C; {'prop_kg', rec.prop_kg; 'P_rcs_W', rec.P_rcs; 'n_rotors_isolated', rec.n_failed}];
        for a = 1:3, C(end+1,:) = {sprintf('sun_body_%s', ax(a)), rec.sun_body(a,:)}; end %#ok<AGROW>
    end
    C = [C; {'P_mtq_W', rec.P_mtq; 'P_rw_W', rec.P_rw; 'rho_kgm3', rec.rho; 'shadow_nu', rec.nu; 'P_gen_W', rec.P_gen; 'soc', rec.soc; ...
             'sun_ok', rec.sun_ok; 'st_ok', rec.st_ok; 'ad_ok', rec.ad_ok}];
    fid = fopen(fullfile(dir_out, 'channels.csv'), 'w');
    fprintf(fid, '%s\n', strjoin(C(:,1)', ','));
    M = cell2mat(C(:,2));
    fmt = [repmat('%.9g,', 1, size(M,1)-1) '%.9g\n'];
    fprintf(fid, fmt, M);
    fclose(fid);
    man = struct('schema', 'adcs-rec/1', 'engine', asils.version(), 'owner', 'Agastya', ...
        'scenario', rec.P.id, 'case', rec.P.case.id, 'case_title', rec.P.case.title, ...
        'product', rec.P.dev.id, 'family', rec.P.dev.family, 'label', rec.P.scenario.label, ...
        'algorithms', rmfield(rec.P.fsw.alg, 'caps'), 'seed', rec.P.seed, 'epoch_utc', rec.P.epoch_utc, ...
        'duration_s', rec.P.sim.duration_s, 'dt_s', rec.P.sim.dt, 'wall_s', rec.wall_s, ...
        'orbit', struct('alt_km', rec.P.orbit.alt_km, 'inc_deg', rec.P.orbit.inc_deg, 'ltan_h', rec.P.orbit.ltan_h, ...
                        'raan_deg', rec.orbit.raan_rad*180/pi, 'period_s', rec.P.orbit.period_s, 'atmosphere', rec.P.orbit.atmos), ...
        'boresight_body', rec.P.dev.boresight', 'metrics', rec.metrics);
    % the flight software it flew: the twin's runtime over the design's algorithms (+asils/+alg), booted from the blob
    % the engine builds for the same run (asils.config)
    man.fsw = struct('impl', 'matlab (in-process: asils.fsw over +asils/+alg)', ...
                     'build_id', ['asils-fsw/1.0.0 (adcs-fswcfg/1) alg ' asils.alg.alg_id()]);
    man.mode_log = rec.mode_log;
    % what it was flown from, so `adcs results stale` can judge it (asils.util.fingerprint)
    R = asils.util.root();
    cf = rec.P.case.file;
    if ~exist(cf, 'file'), cf = fullfile(R, cf); end
    rel = strrep(cf, '\', '/');
    rr = [strrep(R, '\', '/') '/'];
    if strncmp(rel, rr, numel(rr)), rel = rel(numel(rr)+1:end); end
    man.engine_source = asils.util.fingerprint('source');
    ov = {};
    if isfield(rec.P, 'overrides')
        for i = 1:size(rec.P.overrides, 1), ov{end+1} = sprintf('%s=%s', rec.P.overrides{i, 1}, mat2str(rec.P.overrides{i, 2}, 17)); end %#ok<AGROW>
    end
    man.inputs = struct('case_file', rel, 'case_fingerprint', asils.util.fingerprint('file', cf), ...
                        'data_fingerprint', asils.util.fingerprint('data'), 'seed', rec.P.seed, 'overrides', {ov});
    fid = fopen(fullfile(dir_out, 'manifest.json'), 'w');
    fprintf(fid, '%s\n', jsonencode(man));
    fclose(fid);
end
