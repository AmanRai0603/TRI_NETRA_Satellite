% EPHEM_FORCES  Reference vectors for adcs-pop spk/ephem/thirdbody/srp/erp/relativity.
%   Run from matlab_sils/pop:
%     octave-cli --no-gui -q --eval "setup_paths; run('<abs>/refgen/ephem_forces.m')"
%   Writes ../tests/data/ephem_forces.json (every double as %.17g).
%   Owner: Agastya. Copyright (c) 2026 Agastya. All rights reserved.

here = fileparts(mfilename('fullpath'));
if isempty(here), here = pwd; end
outf = fullfile(here, '..', 'tests', 'data', 'ephem_forces.json');

J  = @(v) ['[' strjoin(arrayfun(@(x) sprintf('%.17g', x), v(:).', 'UniformOutput', false), ',') ']'];
Jm = @(M) J(M.');                       % 3x3 -> row-major 9-vector
N  = @(x) sprintf('%.17g', x);
unitv = @(x) x / norm(x);

eph = de440.open();
K   = de440.constants();
fld = grav.defaultField();
mu_field = fld.mu;

epochs = [2020  1  1  0  0  0;
          2021  6 15 12 30  0;
          2023  3 20 21 24  0;
          2024 12 21  9  0  0;
          2027  1  1  6  0  0;
          2029  7  4  3  3  3;
          2031  9 23 18  0  0;
          2033  2 28 23 59 30;
          2035 12 31 12  0  0];

SC16 = sgeom.vleo16u();
sc = struct('mass', SC16.mass, 'Aref', 0.20*0.20, 'Cd', 2.2, 'Cr', 1.3, ...
            'facets', SC16.facets, 'R_bi', eye(3));

Rx = @(a) [1 0 0; 0 cos(a) -sin(a); 0 sin(a) cos(a)];
Ry = @(a) [cos(a) 0 sin(a); 0 1 0; -sin(a) 0 cos(a)];
Rz = @(a) [cos(a) -sin(a) 0; sin(a) cos(a) 0; 0 0 1];

fid = fopen(outf, 'w');
fprintf(fid, '{\n');

% ---------------- constants ----------------
fprintf(fid, '"constants": {"AU_m":%s,"c":%s,"GM_sun":%s,"GM_earth":%s,"GM_moon":%s,"EMRAT":%s,"TSI":%s,"P0":%s,"Re_earth":%s,"f_earth":%s,"Rp_earth":%s,"mu_earth":%s,"omega_earth":%s,"N_A":%s,"Rsun":%s},\n', ...
  N(K.AU_m),N(K.c),N(K.GM_sun),N(K.GM_earth),N(K.GM_moon),N(K.EMRAT),N(K.TSI),N(K.P0),N(K.Re_earth),N(K.f_earth),N(K.Rp_earth),N(K.mu_earth),N(K.omega_earth),N(K.N_A),N(K.Rsun));
fprintf(fid, '"mu_field": %s,\n', N(mu_field));

% ---------------- raw SPK states: every segment at every epoch (+ record edges) ----------------
fprintf(fid, '"spk": [\n');
first = true;
jds = [];
for ie = 1:size(epochs,1)
  e = epochs(ie,:);
  T = timeconv.convertUTC(e(1),e(2),e(3),e(4),e(5),e(6));
  jds(end+1) = T.tdb_jd; %#ok<SAGROW>
end
jspk = [jds, 2451545.0, 2451545.0 + 0.123456789, 2460000.5 + 1e-7];
for ij = 1:numel(jspk)
  for k = 1:numel(eph.keys)
    ct = sscanf(strrep(eph.keys{k}, '_', ' '), '%d');
    [p, v] = de440.state(ct(1), ct(2), jspk(ij), eph);
    if ~first, fprintf(fid, ',\n'); end
    first = false;
    fprintf(fid, '{"center":%d,"target":%d,"jd":%s,"pos":%s,"vel":%s}', ct(1), ct(2), N(jspk(ij)), J(p), J(v));
  end
end
fprintf(fid, '\n],\n');

% ---------------- de440.sun/moon/earth + ephemInputs ----------------
fprintf(fid, '"ephem": [\n');
for ie = 1:numel(jds)
  jd = jds(ie);
  [rs, vs] = de440.sun(jd, eph); [rm, vm] = de440.moon(jd, eph); [re, ve] = de440.earth(jd, eph);
  E = ephemInputs(jd, eph);
  if ie > 1, fprintf(fid, ',\n'); end
  fprintf(fid, ['{"jd":%s,"sun_r":%s,"sun_v":%s,"moon_r":%s,"moon_v":%s,"earth_r":%s,"earth_v":%s,' ...
                '"sun_unit":%s,"sun_dist":%s,"flux_scale":%s,"P_srp":%s,"sun_eci":%s,"sun_vel":%s,' ...
                '"moon_eci":%s,"moon_vel":%s,"GM_sun":%s,"GM_moon":%s,"tide_sun":%s,"tide_moon":%s,' ...
                '"albedo_sun_unit":%s,"earth_helio_pos":%s,"earth_helio_vel":%s,"sun_ra":%s,"sun_dec":%s}'], ...
    N(jd), J(rs), J(vs), J(rm), J(vm), J(re), J(ve), J(E.sun_unit), N(E.sun_dist), N(E.flux_scale), N(E.P_srp), ...
    J(E.sun_eci), J(E.sun_vel), J(E.moon_eci), J(E.moon_vel), N(E.GM_sun), N(E.GM_moon), J(E.tide_sun), J(E.tide_moon), ...
    J(E.albedo_sun_unit), J(E.earth_helio_pos), J(E.earth_helio_vel), N(E.sun_ra), N(E.sun_dec));
end
fprintf(fid, '\n],\n');

% ---------------- force cases ----------------
alts = [500 540 570 600];  incs = [0 28.5 51.6 97.4];
eclModels = {'cylindrical','conical','fine'};
fprintf(fid, '"cases": [\n');
ncase = 0;
for ie = 1:size(epochs,1)
  e = epochs(ie,:);
  T = timeconv.convertUTC(e(1),e(2),e(3),e(4),e(5),e(6));
  E = ephemInputs(T.tdb_jd, eph);
  s = E.sun_unit;
  p = unitv(cross(s, [0;0;1]));
  for kc = 1:7
    if kc <= 4                                   % inclined circular orbits, arbitrary phase
      a = K.Re_earth + 1000*alts(kc); inc = deg2rad(incs(kc));
      Om = 0.7*ie + 1.3*kc; u = 2.1*ie - 0.9*kc;
      rhat = [cos(u)*cos(Om) - sin(u)*cos(inc)*sin(Om); cos(u)*sin(Om) + sin(u)*cos(inc)*cos(Om); sin(u)*sin(inc)];
      that = [-sin(u)*cos(Om) - cos(u)*cos(inc)*sin(Om); -sin(u)*sin(Om) + cos(u)*cos(inc)*cos(Om); cos(u)*sin(inc)];
      r = a*rhat; v = sqrt(K.mu_earth/a)*that; tag = 'orbit';
    else
      a = K.Re_earth + 1000*(520 + 10*ie);
      if kc == 5
        th = 0.05; tag = 'umbra';
      else
        mdl = eclModels{kc - 4};                % 6 -> conical, 7 -> fine penumbra
        lo = 0; hi = pi/2;
        for it = 1:80
          th = 0.5*(lo + hi);
          nu = srp.eclipse(a*(cos(th)*(-s) + sin(th)*p), E.sun_eci, mdl);
          if nu < 0.5, lo = th; else, hi = th; end
        end
        tag = ['penumbra_' mdl];
      end
      r = a*(cos(th)*(-s) + sin(th)*p);
      v = sqrt(K.mu_earth/a)*unitv(cross([0.3;-0.5;0.8], r));
    end
    if mod(kc, 2) == 0
      x = unitv(v); zr = [0;0;1]; if abs(dot(x,zr)) > 0.98, zr = [0;1;0]; end
      y = unitv(cross(zr, x)); z = cross(x, y); R = [x y z];
    else
      R = Rz(0.3 + 0.7*kc + 0.2*ie) * Ry(1.1 - 0.2*kc) * Rx(2.0 + 0.5*ie);
    end
    ecl = eclModels{mod(kc + ie, 3) + 1};

    % third body
    tb_b_s = thirdbody.accel(r, E.sun_eci, E.GM_sun, 'battin');
    tb_b_m = thirdbody.accel(r, E.moon_eci, E.GM_moon, 'battin');
    tb_d_s = thirdbody.accel(r, E.sun_eci, E.GM_sun, 'direct');
    tb_d_m = thirdbody.accel(r, E.moon_eci, E.GM_moon, 'direct');
    tb_t_s = thirdbody.accel(r, E.sun_eci, E.GM_sun, 'tidal');
    tb_t_m = thirdbody.accel(r, E.moon_eci, E.GM_moon, 'tidal');
    tb_l_s = thirdbody.accel(r, E.sun_eci, E.GM_sun, 'legendre');
    tb_l_m = thirdbody.accel(r, E.moon_eci, E.GM_moon, 'legendre');
    tb_l8_m = thirdbody.legendre(r, E.moon_eci, E.GM_moon, 8);
    ctx = struct(); ctx.r_eci = r; ctx.v_eci = v; ctx.E = E; ctx.T = T; ctx.grav = fld;
    ctx.sc = sc; ctx.sc.R_bi = R;
    ctx.cfg.forces.thirdbody = struct('model', 'battin');
    f_tb = forces.thirdbody(ctx);
    ctx.cfg.forces.thirdbody = struct('model', 'legendre');
    f_tb_leg = forces.thirdbody(ctx);

    % eclipse
    nu_cyl = srp.eclipse(r, E.sun_eci, 'cylindrical');
    nu_con = srp.eclipse(r, E.sun_eci, 'conical');
    nu_fin = srp.eclipse(r, E.sun_eci, 'fine');
    nu_def = srp.eclipse(r, E.sun_eci);

    % srp
    ctx.cfg.forces.srp = struct('model', 'cannonball', 'eclipse', ecl);
    f_srp_cb = forces.srp(ctx);
    ctx.cfg.forces.srp = struct('model', 'cannonball', 'eclipse', ecl, 'Cr', 1.7);
    f_srp_cb17 = forces.srp(ctx);
    ctx.cfg.forces.srp = struct('model', 'boxwing', 'eclipse', ecl);
    f_srp_bw = forces.srp(ctx);
    shat = unitv(E.sun_eci - r);
    [bw_a, bw_F] = srp.boxwing(E.P_srp, 0.37, shat, R, sc);

    % erp
    CrAoM = 1.3*sc.Aref/sc.mass;
    [erp_k, ck] = erp.knocke(r, E.sun_eci, CrAoM, T.doy);
    erp_s = erp.simple(r, E.sun_eci, CrAoM, T.doy);
    erp_c = erp.ceres(r, E.sun_eci, CrAoM, []);
    [erp_bw, cbw] = erp.boxwing(r, E.sun_eci, R, sc, T.doy);
    ctx.cfg.forces.erp = struct('model', 'boxwing', 'nrings', 8, 'nseg', 16);
    f_erp_bw = forces.erp(ctx);
    ctx.cfg.forces.erp = struct('model', 'knocke', 'Cr', 1.1);
    f_erp_k = forces.erp(ctx);

    % relativity
    [rel_all, rp] = relativity.total(r, v, E, [], mu_field);
    ctx.cfg.forces.relativity = struct();
    f_rel = forces.relativity(ctx);
    rel_lt_def = relativity.lenseThirring(r, v);
    rel_sch_def = relativity.schwarzschild(r, v);

    ncase = ncase + 1;
    if ncase > 1, fprintf(fid, ',\n'); end
    fprintf(fid, '{"tag":"%s","jd":%s,"doy":%s,"r":%s,"v":%s,"R_bi":%s,"eclipse_model":"%s",', ...
      tag, N(T.tdb_jd), N(T.doy), J(r), J(v), Jm(R), ecl);
    fprintf(fid, '"tb_battin_sun":%s,"tb_battin_moon":%s,"tb_direct_sun":%s,"tb_direct_moon":%s,"tb_tidal_sun":%s,"tb_tidal_moon":%s,"tb_legendre_sun":%s,"tb_legendre_moon":%s,"tb_legendre8_moon":%s,"f_thirdbody":%s,"f_thirdbody_legendre":%s,', ...
      J(tb_b_s), J(tb_b_m), J(tb_d_s), J(tb_d_m), J(tb_t_s), J(tb_t_m), J(tb_l_s), J(tb_l_m), J(tb_l8_m), J(f_tb), J(f_tb_leg));
    fprintf(fid, '"nu_cylindrical":%s,"nu_conical":%s,"nu_fine":%s,"nu_default":%s,"f_srp_cannonball":%s,"f_srp_cannonball_cr17":%s,"f_srp_boxwing":%s,"srp_boxwing_a":%s,"srp_boxwing_F":%s,', ...
      N(nu_cyl), N(nu_con), N(nu_fin), N(nu_def), J(f_srp_cb), J(f_srp_cb17), J(f_srp_bw), J(bw_a), J(bw_F));
    fprintf(fid, '"erp_knocke":%s,"erp_knocke_sw":%s,"erp_knocke_lw":%s,"erp_simple":%s,"erp_ceres":%s,"erp_boxwing":%s,"erp_boxwing_sw":%s,"erp_boxwing_lw":%s,"f_erp_boxwing_8x16":%s,"f_erp_knocke_cr11":%s,', ...
      J(erp_k), J(ck.sw), J(ck.lw), J(erp_s), J(erp_c), J(erp_bw), J(cbw.sw), J(cbw.lw), J(f_erp_bw), J(f_erp_k));
    fprintf(fid, '"rel_total":%s,"rel_schwarzschild":%s,"rel_lensethirring":%s,"rel_desitter":%s,"f_relativity":%s,"rel_lt_default":%s,"rel_sch_default":%s}', ...
      J(rel_all), J(rp.schwarzschild), J(rp.lensethirring), J(rp.desitter), J(f_rel), J(rel_lt_def), J(rel_sch_def));
    fflush(stdout); printf('case %d %s nu=%.3f\n', ncase, tag, nu_con);
  end
end
fprintf(fid, '\n],\n');

% ---------------- secular (Lidov-Kozai) ----------------
Esec = ephemInputs(jds(5), eph);
phiS = thirdbody.secular.phiQuad(K.Re_earth + 550e3, K.GM_sun, norm(Esec.sun_eci), 0.0167);
phiM = thirdbody.secular.phiQuad(K.Re_earth + 550e3, K.GM_moon, norm(Esec.moon_eci));
[jv, ev] = thirdbody.secular.elem2vectors(7e6, 0.3, 1.1, 0.4, 0.9);
[ec, ic, Om, w] = thirdbody.secular.vectors2elem(jv, ev);
nS = unitv([0; -0.3977771559; 0.9174820621]);
[dj, de] = thirdbody.secular.kozaiRates(jv, ev, nS, phiS);
P = struct('nhat', {nS, [0.1; -0.35; 0.93]}, 'phiQ', {phiS*1e3, phiM*1e3});
out = thirdbody.secular.propagate(K.Re_earth + 550e3, 0.01, deg2rad(63), 0.2, 0.5, P, 86400*30, 200);
fprintf(fid, '"secular": {"phiS":%s,"phiM":%s,"j":%s,"e":%s,"ecc":%s,"inc":%s,"Om":%s,"w":%s,"dj":%s,"de":%s,', ...
  N(phiS), N(phiM), J(jv), J(ev), N(ec), N(ic), N(Om), N(w), J(dj), J(de));
fprintf(fid, '"prop_t":%s,"prop_e":%s,"prop_inc":%s,"prop_Om":%s,"prop_w":%s,"prop_kozai":%s}\n', ...
  J(out.t), J(out.e), J(out.inc), J(out.Om), J(out.w), J(out.kozai));
fprintf(fid, '}\n');
fclose(fid);
printf('wrote %s (%d cases)\n', outf, ncase);
