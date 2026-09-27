function make_OD_fixtures(dateStr, tspan_s)
%MAKE_OD_FIXTURES  Synthetic, format-correct ITSG files -> a closed-loop test of validate_OD.
%
%   make_OD_fixtures()                       % CHAMP 2007-01-01, 4 h
%   make_OD_fixtures('2007-01-01', 4*3600)
%
%   WHY. validate_OD can only be exercised where the ITSG cache and the network
%   are. This writes CHAMP fixtures into that same cache in the real GROOPS
%   layout -- including the two bare-number header lines that a naive parser eats
%   as data -- so validate_OD runs offline, through the REAL data.itsg parser.
%
%   The point is that the fixtures are built FROM THE ENGINE'S OWN OUTPUT, so
%   every answer is known in advance and any deviation is a plumbing bug:
%       [1] vs reducedDynamicOrbit  ~0.03 m   (interpolation floor only)
%       [2] vs kinematicOrbit       ~0.05 m   (the 2 cm noise injected below)
%           reference spread        ~0.03 m
%       [3] ACC ratio model/meas     1.000    (proves the quaternion sense and
%                                              that parts.drag+srp+erp is the
%                                              right non-conservative sum)
%       [4] density ratio            1.000
%   That is how the 72 m pchip artifact was caught: [2] read 72.64 m against a
%   reference that is truth + 2 cm.
%
%   WHAT IT DOES NOT TEST. Real densities, real Cd, real GROOPS quirks, real
%   attitude sign flips, leap seconds. Those need the real cached files. This
%   tests PLUMBING -- field names, frames, units, product names, shadowing --
%   which is where the bugs actually were.
%
%   WARNING  These files sit in the same cache as real downloads and are named
%   identically. Point data.root() at a scratch dir first, or delete them after:
%       data.root('set', fullfile(tempdir,'itsg_fixtures'));
%
%   Requires an offline-capable atmosphere ('exponential') and gravity ('default').

    if nargin<1 || isempty(dateStr), dateStr = '2007-01-01'; end
    if nargin<2 || isempty(tspan_s), tspan_s = 4*3600; end
SAT='CHAMP'; DATE=dateStr;
dv0 = datevec(datenum(dateStr,'yyyy-mm-dd'));
MJD0 = datenum(dv0(1),dv0(2),dv0(3)) - 678942;
DUR = tspan_s;  DT = 10;

K=de440.constants(); mu=K.mu_earth;
a=6378137+450e3; inc=87.3*pi/180; vc=sqrt(mu/a);
cfg = config.defaultConfig();
cfg.epoch=[dv0(1) dv0(2) dv0(3) 0 0 0];
cfg.r0=[a;0;0]; cfg.v0=[0;vc*cos(inc);vc*sin(inc)];
cfg.tspan=DUR;
cfg.spacecraft.mass=522; cfg.spacecraft.Aref=0.767;
cfg.spacecraft.Cd=3.0;   cfg.spacecraft.Cr=1.3;
cfg.spaceweather.manual=struct('F107',90,'F107a',90,'ap',8);
cfg.forces.drag.atmos='exponential';      % only offline-capable model in Octave
cfg.forces.drag.corotate=true;
cfg.forces.srp.Cr=1.3;
cfg.forces.relativity=struct('on',true,'terms',{{'schwarzschild'}});
cfg.gravityField=struct('field','default','degree',6);
cfg.forces.gravity.degree=6; cfg.forces.gravity.order=6;
cfg.output=struct('times',(0:DT:DUR).');
cfg.integrator=struct('method','rk78','rtol',1e-11,'atol',1e-6);

W = op.buildWorld(cfg);
sol = op.propagate(cfg);
n = numel(sol.t);
mjd = MJD0 + sol.t/86400;
fprintf('truth: %d epochs, |r0|=%.3f km\n', n, norm(sol.r(1,:))/1000);

outdir = fullfile(data.root(),'itsg','CHAMP');
if ~exist(outdir,'dir'), mkdir(outdir); end

% ---------- 1. reducedDynamicOrbit : MJD + pos + vel + acc  (9 data cols) ----
acc = zeros(n,3);
for k=1:n
    acc(k,:) = op.accel(sol.t(k), sol.r(k,:).', sol.v(k,:).', W).';
end
labs = {'pos x [m]','pos y [m]','pos z [m]','vel x [m/s]','vel y [m/s]','vel z [m/s]', ...
        'acc x [m/s^2]','acc y [m/s^2]','acc z [m/s^2]'};
writeGroops(fullfile(outdir,sprintf('CHAMP_reducedDynamicOrbit_%s.txt.gz',DATE)), ...
            'ORBIT', -6, labs, [mjd, sol.r, sol.v, acc]);

% ---------- 2. kinematicOrbit : MJD + pos, non-equidistant, 2 cm noise -------
rand('seed',42); randn('seed',42);
keep = sort(randperm(n, round(0.85*n)));           % irregular epochs, like the real thing
rk = sol.r(keep,:) + 0.02*randn(numel(keep),3);    % 2 cm -> the reference floor
writeGroops(fullfile(outdir,sprintf('CHAMP_kinematicOrbit_%s.txt.gz',DATE)), ...
            'ORBIT', -6, {'pos x [m]','pos y [m]','pos z [m]'}, [mjd(keep), rk]);

% ---------- 3. attitude : MJD + q0 qx qy qz  (satellite -> celestial) --------
% ram-pointing: x_body = along v, z_body = nadir, y = z cross x
q = zeros(n,4);
for k=1:n
    r=sol.r(k,:).'; v=sol.v(k,:).';
    xb = v/norm(v);
    zb = -r/norm(r); zb = zb - (zb.'*xb)*xb; zb = zb/norm(zb);
    yb = cross(zb,xb);
    A  = [xb yb zb];              % columns = body axes in celestial => A = R_cel<-sat
    q(k,:) = dcm2quat_local(A);
end
writeGroops(fullfile(outdir,sprintf('CHAMP_attitude_%s.txt.gz',DATE)), ...
            'STARCAMERA', -8, {'quaternion q0 []','quaternion q1 []','quaternion q2 []','quaternion q3 []'}, ...
            [mjd, q]);

% ---------- 4. nonConservativeForces : MJD + ax ay az, SATELLITE frame -------
% Truth = the model's OWN drag+srp, rotated celestial -> satellite. So a perfect
% validate_OD must report ratio modelled/measured = 1.000 exactly.
a_sat = zeros(n,3);
for k=1:n
    [~,parts] = op.accel(sol.t(k), sol.r(k,:).', sol.v(k,:).', W);
    anc = zeros(3,1);
    for f={'drag','srp','erp'}
        if isfield(parts,f{1}), anc = anc + parts.(f{1})(:); end
    end
    A = validation.quat2dcm(q(k,:));       % sat -> celestial (the lifted helper)
    a_sat(k,:) = (A.'*anc).';              % celestial -> sat
end
writeGroops(fullfile(outdir,sprintf('CHAMP_nonConservativeForces_%s.txt.gz',DATE)), ...
            'ACCELEROMETER', -9, {'acceleration x [m/s^2]','acceleration y [m/s^2]','acceleration z [m/s^2]'}, ...
            [mjd, a_sat]);

% ---------- 5. neutralDensity_ACC : MJD + rho  (release 1.0, MISCVALUE) ------
% NOTE: NO dataN: labels in this file -> readGroops must infer ncol. That is the
% real r1.0 layout for CHAMP, and it is what validate_OD does NOT ask for.
sel = 1:6:n;                                       % 60 s, like the real product
rho = zeros(numel(sel),1);
for i=1:numel(sel)
    k=sel(i);
    [C,~] = frames.eci2ecef(utcAt(cfg.epoch,sol.t(k)), 'gmst', struct('dUT1',0));
    re = C*sol.r(k,:).';
    [~,~,alt] = op.geodetic(re);
    atm = atmos.exponential(alt/1000);
    rho(i) = atm.rho;
end
writeGroops(fullfile(outdir,sprintf('CHAMP_neutralDensity_ACC_%s.txt.gz',DATE)), ...
            'MISCVALUE', -5, {}, [mjd(sel), rho]);

fprintf('\nfixtures written to %s\n', outdir);
ls(outdir)
end

% ============================================================== helpers =======
function writeGroops(gzpath, type, code, labels, M)
    txt = strrep(gzpath,'.gz','');
    fid = fopen(txt,'w');
    fprintf(fid,'groops instrument version=20200123\n');
    fprintf(fid,'# %s\n', type);
    fprintf(fid,'%11d %10d\n', code, 1);                 % BARE numbers -- the trap
    if isempty(labels)
        fprintf(fid,'# Time [MJD]\n');                   % MISCVALUE: no dataN labels
    else
        s = '# Time [MJD]';
        for i=1:numel(labels), s=[s sprintf('  data%d: %s',i-1,labels{i})]; end
        fprintf(fid,'%s\n', s);
    end
    fprintf(fid,'# ==================================================\n');
    fprintf(fid,'%11d\n', size(M,1));                    % BARE number -- the other trap
    fmt = ['%.10f', repmat(' %.12e',1,size(M,2)-1), '\n'];
    fprintf(fid, fmt, M.');
    fclose(fid);
    if exist(gzpath,'file'), delete(gzpath); end
    gzip(txt); delete(txt);
end

function q = dcm2quat_local(A)
    tr = trace(A);
    if tr > 0
        s = sqrt(tr+1)*2; q0 = 0.25*s;
        q1 = (A(3,2)-A(2,3))/s; q2 = (A(1,3)-A(3,1))/s; q3 = (A(2,1)-A(1,2))/s;
    elseif A(1,1)>A(2,2) && A(1,1)>A(3,3)
        s = sqrt(1+A(1,1)-A(2,2)-A(3,3))*2;
        q0 = (A(3,2)-A(2,3))/s; q1 = 0.25*s;
        q2 = (A(1,2)+A(2,1))/s; q3 = (A(1,3)+A(3,1))/s;
    elseif A(2,2)>A(3,3)
        s = sqrt(1+A(2,2)-A(1,1)-A(3,3))*2;
        q0 = (A(1,3)-A(3,1))/s; q1 = (A(1,2)+A(2,1))/s;
        q2 = 0.25*s;            q3 = (A(2,3)+A(3,2))/s;
    else
        s = sqrt(1+A(3,3)-A(1,1)-A(2,2))*2;
        q0 = (A(2,1)-A(1,2))/s; q1 = (A(1,3)+A(3,1))/s;
        q2 = (A(2,3)+A(3,2))/s; q3 = 0.25*s;
    end
    q = [q0 q1 q2 q3]; q = q/norm(q);
end

function u = utcAt(epoch, dt)
    u = op.addsec(epoch, dt);
end
