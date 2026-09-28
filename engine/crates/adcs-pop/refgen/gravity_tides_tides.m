% refgen/gravity_tides_tides.m -- reference vectors for the Rust port of the POP tides
% (02_forces/+solidtides, +oceantides, +tideutil, +forces/solidtides.m, oceantides.m).
%
% Run from matlab_sils/pop:
%   octave-cli --no-gui -q --eval "setup_paths; run('<abs>/refgen/gravity_tides_tides.m')"
%
% Inputs are built exactly as op.accel builds ctx: T = timeconv.convertUTC(utc),
% E = ephemInputs(T.tdb_jd) (DE440), C = frames.eci2ecefGMST(T.gmst_rad, xp, yp),
% ctx.r_ecef = C*r_eci, ctx.grav = op.gravLoad(default field).
% Writes ../tests/data/gravity_tides_tides.json (all numbers %.17g).
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
function s = jmat(M)
    s = jvec(reshape(M.', 1, []));
end
function s = jdcs(d)
    s = sprintf('{"n":%d,"dc":%s,"ds":%s}', size(d.dC, 1), jmat(d.dC), jmat(d.dS));
end

here = fileparts(mfilename('fullpath'));
if isempty(here), here = pwd; end
dataDir = fullfile(here, '..', 'tests', 'data');

Fd = op.gravLoad(struct('field', 'default', 'degree', 20));
pp = strsplit(path, pathsep);          % the POP root = parent of 01_core on the path
popRoot = fileparts(pp{find(~cellfun(@isempty, regexp(pp, '[/\\]01_core$')), 1)});
S = load(fullfile(popRoot, 'force_data', 'fes2004_deg10.mat'));
% oceantides.fromModel multiplies tbl.doodson (int64 in the .mat) by a double
% vector, which MATLAB/Octave refuse for integer matrices: convert to double.
tbl = S; tbl.doodson = double(S.doodson); tbl.n = double(S.n(:)); tbl.m = double(S.m(:));
tbl.Cp = S.Cp(:); tbl.Sp = S.Sp(:); tbl.Cm = S.Cm(:); tbl.Sm = S.Sm(:);

epochs = [2020 1 1 0 0 0;
          2021 9 23 14 12 7;
          2023 5 5 23 59 59;
          2025 3 20 9 1 0;
          2027 1 1 6 0 0;
          2029 11 11 11 11 11;
          2032 2 29 18 30 0;
          2035 12 31 23 0 0];
Rw = 6378137.0;
ep = {};
for e = 1:size(epochs, 1)
    u = epochs(e, :);
    T = timeconv.convertUTC(u(1), u(2), u(3), u(4), u(5), u(6), 0);
    E = ephemInputs(T.tdb_jd);
    xp = (0.05 + 0.02*e) / 206264.806247; yp = (0.35 - 0.03*e) / 206264.806247;
    [C, Ct] = frames.eci2ecefGMST(T.gmst_rad, xp, yp);
    rs = C * E.sun_eci; rm = C * E.moon_eci;
    d_iers = solidtides.iers2010(rm, rs, Fd.mu, Fd.Re);
    d_iers_def = solidtides.iers2010(rm, rs);
    d_el = solidtides.elastic2(rm, rs);
    d_el2 = solidtides.elastic2(rm, rs, 0.3, Fd.mu, Fd.Re);
    d_fd = solidtides.freqDependent(T.gmst_rad, d_iers);
    beta = oceantides.doodson(T.tt_jd);
    d_ml = oceantides.mainLines(T.tt_jd);
    d_fm = oceantides.fromModel(T.tt_jd, tbl);
    PL = solidtides.normLegendre(4, sin(0.3 + 0.1*e));
    pos = {};
    for i = 1:6
        alt = (300 + 140*(i-1)) * 1e3;
        la = (-88 + 35.3*(i-1) + 3*e) * pi/180; lo = (17*e + 61*i) * pi/180;
        r = (Rw + alt) * [cos(la)*cos(lo); cos(la)*sin(lo); sin(la)];
        ctx = struct('C', C, 'Ct', Ct, 'r_eci', r, 'r_ecef', C*r, 'grav', Fd, 'T', T, 'E', E);
        a_sol_ecef = tideutil.accelFromDeg2(ctx.r_ecef, d_iers, Fd.mu, Fd.Re);
        a_sol = forces.solidtides(ctx);
        a_oc_ecef = tideutil.accelFromDeg2(ctx.r_ecef, d_ml, Fd.mu, Fd.Re);
        a_oc = forces.oceantides(ctx);
        a_fm_ecef = tideutil.accelFromDeg2(ctx.r_ecef, d_fm, Fd.mu, Fd.Re);
        a_def_ecef = tideutil.accelFromDeg2(ctx.r_ecef, d_iers);
        pos{end+1} = sprintf(['{"r_eci":%s,"r_ecef":%s,"solid_ecef":%s,"solid_eci":%s,' ...
            '"ocean_ecef":%s,"ocean_eci":%s,"fes_ecef":%s,"solid_ecef_defaults":%s}'], ...
            jvec(r), jvec(ctx.r_ecef), jvec(a_sol_ecef), jvec(a_sol), jvec(a_oc_ecef), jvec(a_oc), ...
            jvec(a_fm_ecef), jvec(a_def_ecef));
    end
    ep{end+1} = sprintf(['{"utc":%s,"tt_jd":%s,"gmst_rad":%s,"c":%s,"sun_eci":%s,"moon_eci":%s,' ...
        '"sun_ecef":%s,"moon_ecef":%s,"iers2010":%s,"iers2010_defaults":%s,"elastic2":%s,' ...
        '"elastic2_fld":%s,"freqdep":%s,"doodson":%s,"mainlines":%s,"frommodel":%s,' ...
        '"legendre_x":%s,"legendre4":%s,"pos":[%s]}'], ...
        jvec(u), jnum(T.tt_jd), jnum(T.gmst_rad), jmat(C), jvec(E.sun_eci), jvec(E.moon_eci), ...
        jvec(rs), jvec(rm), jdcs(d_iers), jdcs(d_iers_def), jdcs(d_el), jdcs(d_el2), jdcs(d_fd), ...
        jvec(beta), jdcs(d_ml), jdcs(d_fm), jnum(sin(0.3 + 0.1*e)), jmat(PL), strjoin(pos, ','));
    printf('epoch %d done\n', e);
end
fid = fopen(fullfile(dataDir, 'gravity_tides_tides.json'), 'w');
fprintf(fid, '{"mu":%s,"re":%s,"epochs":[\n%s]}\n', jnum(Fd.mu), jnum(Fd.Re), strjoin(ep, ',\n'));
fclose(fid);
printf('wrote %s\n', fullfile(dataDir, 'gravity_tides_tides.json'));
