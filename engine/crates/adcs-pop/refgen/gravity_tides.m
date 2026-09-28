% refgen/gravity_tides.m -- reference vectors for the Rust port of the POP gravity
% (01_core/+grav, 01_core/+op/gravLoad.m, 02_forces/+forces/gravity.m).
%
% Run from matlab_sils/pop:
%   octave-cli --no-gui -q --eval "setup_paths; run('<abs>/refgen/gravity_tides.m')"
%
% Writes
%   ../tests/data/gravity_tides_synth.gfc  a SYNTHETIC ICGEM file (made up
%        coefficients with a Kaula-like 1e-5/n^2 envelope; NOT a gravity model) used
%        only to exercise grav.loadGFC and the full tesseral recursion to degree 70.
%   ../tests/data/gravity_tides_grav.json  the MATLAB outputs, printed with %.17g.
% Owner: Agastya. Copyright (c) 2026 Agastya. All rights reserved.
1;

function s = jnum(x)
    if x == 0 && 1/x < 0, s = '-0'; else, s = sprintf('%.17g', x); end
end
function s = jvec(v)
    v = v(:).';
    c = cell(1, numel(v));
    for i = 1:numel(v), c{i} = jnum(v(i)); end
    s = ['[' strjoin(c, ',') ']'];
end
function s = jmat(M)      % row-major flattening
    s = jvec(reshape(M.', 1, []));
end

here = fileparts(mfilename('fullpath'));
if isempty(here), here = pwd; end
dataDir = fullfile(here, '..', 'tests', 'data');
if ~exist(dataDir, 'dir'), mkdir(dataDir); end

% ---------------------------------------------------------------- synthetic .gfc
gfc = fullfile(dataDir, 'gravity_tides_synth.gfc');
NS = 72;
fid = fopen(gfc, 'w');
fprintf(fid, 'generating_institute   adcs-pop refgen (synthetic test data, NOT a gravity model)\n');
fprintf(fid, 'product_type           gravity_field\n');
fprintf(fid, 'modelname              SYNTH_TEST_72\n');
fprintf(fid, 'earth_gravity_constant 0.3986004415E+15\n');
fprintf(fid, 'radius                 0.63781363E+07\n');
fprintf(fid, 'max_degree             %d\n', NS);
fprintf(fid, 'errors                 formal\n');
fprintf(fid, 'norm                   fully_normalized\n');
fprintf(fid, 'tide_system            tide_free\n');
fprintf(fid, '\n');
fprintf(fid, 'key   L    M         C                       S                  sigma C    sigma S\n');
fprintf(fid, 'end_of_head ===================================================================\n');
for n = 0:NS
    for m = 0:n
        if n == 0, c = 1; s = 0;
        elseif n == 1, c = 0; s = 0;
        elseif n == 2 && m == 0, c = -4.84165143790815e-4; s = 0;
        else
            env = 1e-5 / n^2;
            c = env * sin(12.9898*n + 78.233*m + 0.5);
            s = env * cos(39.3468*n + 11.135*m + 0.25);
            if m == 0, s = 0; end
        end
        ln = sprintf('gfc %4d %4d %24.15e %24.15e %11.4e %11.4e', n, m, c, s, 1e-12, 1e-12);
        if mod(n + m, 3) == 0, ln = strrep(ln, 'e', 'D'); end   % Fortran exponents
        if mod(n + m, 7) == 0, ln = strrep(ln, ' ', sprintf('\t')); end % tab separated
        fprintf(fid, '%s\n', ln);
        if n >= 2 && n <= 4    % time-variable rows: must be ignored by the loader
            fprintf(fid, 'gfct %4d %4d %24.15e %24.15e %11.4e %11.4e 20050101.0000\n', n, m, 1.0, 1.0, 0, 0);
        end
    end
    if n == 10, fprintf(fid, '\n'); end
end
fclose(fid);

% ---------------------------------------------------------------- fields
Fd = op.gravLoad(struct('field', 'default', 'degree', 20));
Fs = op.gravLoad(struct('field', gfc, 'degree', 70));
Fs20 = grav.loadGFC(gfc, 20);
Fsall = grav.loadGFC(gfc);

% ---------------------------------------------------------------- positions
Rw = 6378137.0;
alts = [300 450 600 800 1000] * 1e3;
lats = [-90 -89.9999 -75 -45 -20 0 13 37 60 88 89.99 90];
P = [];
k = 0;
for ia = 1:numel(alts)
    for il = 1:numel(lats)
        k = k + 1;
        lon = mod(37.1 * k, 360) - 180;
        la = lats(il) * pi/180; lo = lon * pi/180;
        rr = Rw + alts(ia);
        P(:, end+1) = rr * [cos(la)*cos(lo); cos(la)*sin(lo); sin(la)];
    end
end
P(:, end+1) = [0; 0; Rw + 500e3];          % exact geographic poles
P(:, end+1) = [0; 0; -(Rw + 700e3)];
P(:, end+1) = [Rw + 400e3; 0; 0];
np = size(P, 2);

out = {};
out{end+1} = ['"positions":' jmat(P.')];
out{end+1} = sprintf('"default":{"mu":%s,"re":%s,"nmax":%d,"name":"%s","cbar":%s,"sbar":%s,"j":%s}', ...
    jnum(Fd.mu), jnum(Fd.Re), Fd.nmax, Fd.name, jmat(Fd.Cbar), jmat(Fd.Sbar), jvec(Fd.J));
out{end+1} = sprintf(['"synth":{"file":"gravity_tides_synth.gfc","mu":%s,"re":%s,"nmax":%d,"name":"%s","j":%s,' ...
    '"nmax20":%d,"cbar20":%s,"sbar20":%s,"nmax_all":%d,"c_all_72_5":%s,"s_all_72_71":%s,"c70_33":%s}'], ...
    jnum(Fs.mu), jnum(Fs.Re), Fs.nmax, Fs.name, jvec(Fs.J), Fs20.nmax, jmat(Fs20.Cbar), jmat(Fs20.Sbar), ...
    Fsall.nmax, jnum(Fsall.Cbar(73, 6)), jnum(Fsall.Sbar(73, 72)), jnum(Fs.Cbar(71, 34)));

% spherical harmonics
cases = { 'default', 2, 2; 'default', 6, 6; 'default', 6, 0; 'default', 4, 2; ...
          'synth', 2, 2; 'synth', 6, 6; 'synth', 20, 20; 'synth', 20, 0; 'synth', 70, 70; 'synth', 70, 30 };
sc = {};
for c = 1:size(cases, 1)
    if strcmp(cases{c, 1}, 'default'), F = Fd; else, F = Fs; end
    A = zeros(np, 3);
    for i = 1:np
        A(i, :) = grav.sphericalHarmonic(P(:, i), F.mu, F.Re, F.Cbar, F.Sbar, cases{c, 2}, cases{c, 3}).';
    end
    sc{end+1} = sprintf('{"field":"%s","deg":%d,"ord":%d,"acc":%s}', cases{c, 1}, cases{c, 2}, cases{c, 3}, jmat(A));
    printf('sphharm %s %d/%d done\n', cases{c, 1}, cases{c, 2}, cases{c, 3});
end
out{end+1} = ['"sph":[' strjoin(sc, ',') ']'];

% potential
pc = {};
pcases = { 'default', 6; 'synth', 20; 'synth', 70 };
for c = 1:size(pcases, 1)
    if strcmp(pcases{c, 1}, 'default'), F = Fd; else, F = Fs; end
    U = zeros(np, 1);
    for i = 1:np
        U(i) = grav.potential(P(:, i), F.mu, F.Re, F.Cbar, F.Sbar, pcases{c, 2});
    end
    pc{end+1} = sprintf('{"field":"%s","nmax":%d,"u":%s}', pcases{c, 1}, pcases{c, 2}, jvec(U));
end
out{end+1} = ['"pot":[' strjoin(pc, ',') ']'];

% j2accel (default J) and twoBody
jc = {};
for nj = 1:5
    A = zeros(np, 3);
    for i = 1:np, A(i, :) = grav.j2accel(P(:, i), Fd.mu, Fd.Re, Fd.J(1:nj)).'; end
    jc{end+1} = sprintf('{"nj":%d,"acc":%s}', nj, jmat(A));
end
out{end+1} = ['"j2accel":[' strjoin(jc, ',') ']'];
A = zeros(np, 3);
for i = 1:np, A(i, :) = grav.twoBody(P(:, i), Fd.mu).'; end
out{end+1} = ['"twobody":' jmat(A)];

% forces.gravity through a ctx (ECI in, ECI out) with a real GMST+polar-motion C
fc = {};
fcases = { 'default', 'twobody', 20, []; 'default', 'j2', 20, []; 'default', 'j3', 20, []; ...
           'default', 'j6', 20, []; 'default', 'sphharm', 20, []; 'default', 'sphharm', 4, 3; ...
           'synth', 'sphharm', 70, []; 'synth', 'sphharm', 20, 10; 'synth', 'j5', 70, [] };
epochs = [2020 1 1 0 0 0; 2027 1 1 6 0 0; 2031 7 19 13 47 12.5];
for c = 1:size(fcases, 1)
    if strcmp(fcases{c, 1}, 'default'), F = Fd; else, F = Fs; end
    g = struct('on', true, 'model', fcases{c, 2}, 'degree', fcases{c, 3});
    if ~isempty(fcases{c, 4}), g.order = fcases{c, 4}; end
    for e = 1:size(epochs, 1)
        T = timeconv.convertUTC(epochs(e,1), epochs(e,2), epochs(e,3), epochs(e,4), epochs(e,5), epochs(e,6), 0);
        [C, Ct] = frames.eci2ecefGMST(T.gmst_rad, 0.12/206264.806, 0.31/206264.806);
        for i = 1:7:np
            ctx = struct();
            ctx.cfg.forces.gravity = g; ctx.grav = F; ctx.C = C; ctx.Ct = Ct;
            ctx.r_eci = P(:, i); ctx.r_ecef = C * P(:, i);
            a = forces.gravity(ctx);
            ord = -1; if ~isempty(fcases{c, 4}), ord = fcases{c, 4}; end
            fc{end+1} = sprintf('{"field":"%s","model":"%s","degree":%d,"order":%d,"c":%s,"r_eci":%s,"a":%s}', ...
                fcases{c, 1}, fcases{c, 2}, fcases{c, 3}, ord, jmat(C), jvec(P(:, i)), jvec(a));
        end
    end
end
out{end+1} = ['"forces":[' strjoin(fc, ',') ']'];

fid = fopen(fullfile(dataDir, 'gravity_tides_grav.json'), 'w');
fprintf(fid, '{%s}\n', strjoin(out, ',\n'));
fclose(fid);
printf('wrote %s\n', fullfile(dataDir, 'gravity_tides_grav.json'));
