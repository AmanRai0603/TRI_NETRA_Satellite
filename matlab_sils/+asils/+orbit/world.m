function W = world(W, env)
%ASILS.ORBIT.WORLD  What the in-loop orbit's force model is handed once per run (the engine's adcs-pop World::new): W
%   holds the force set (W.fs), the epoch (W.epoch, UTC [Y Mo D H Mi S]) and the spacecraft (W.sc); env the run's
%   space weather (F10.7, its mean, Kp, ap). Added: the indices as the density takes them (swindex.from_manual), the
%   default field and its spherical-harmonic workspace (gravity.default_field, sph_setup), the Earth's rate in the
%   inertial frame (earthframes.earth_rate_from over the GMST frames 15 s either side of the epoch, op.buildWorld's
%   earthRateECI) and whether a force needs the ephemeris.
%   Owner: Agastya. Copyright (c) 2026 Agastya. All rights reserved.
    fs = W.fs;
    if fs.erp_on || fs.relativity_on || fs.solidtides || fs.oceantides
        error('asils:orbit', 'the force set names Earth radiation, relativity or tides: the twin''s orbit flies none of them');
    end
    W.need_ephem = fs.tb_on || fs.srp_on;

    % the run's space weather: Kp one value (kind 1, as the four-slot [Kp 0 Kp 0]) and ap, ap3 not given
    [st, W.sw] = asils.models.swindex.from_manual(env.F107, true, env.F107a, 1, [env.Kp; 0; env.Kp; 0], true, env.ap, false, NaN);
    if st ~= 0, error('asils:orbit', 'the run''s space weather is refused (status %d)', st); end

    % the field and its workspace: degrees 0..nf of the default field, evaluated to the force set's degree and order
    [W.gm, W.re, cbar] = asils.models.gravity.default_field();
    nf = round(sqrt(numel(cbar))) - 1;
    W.g_nws = nf;
    W.g_n = min(fs.grav_degree, nf);
    if fs.has_grav_order, W.g_m = min(fs.grav_order, W.g_n); else, W.g_m = W.g_n; end
    nc = (nf + 1)^2; nv = (nf + 3)^2;
    [~, ~, ~, W.g_c, W.g_s, W.g_f1, W.g_f2] = asils.models.gravity.sph_setup(nf, cbar, zeros(nc, 1), nf + 1, ...
        zeros(nc, 1), zeros(nc, 1), zeros(nv, 1), zeros(nv, 1));
    W.g_v = zeros(nv, 1); W.g_w = zeros(nv, 1);

    % the Earth's rate in the inertial frame (the GMST build, no polar motion)
    d = 15;
    up = asils.models.timescales.addsec(W.epoch, d); um = asils.models.timescales.addsec(W.epoch, -d);
    tp = asils.models.timescales.convert_utc(up(1), up(2), up(3), up(4), up(5), up(6), 0);
    tm = asils.models.timescales.convert_utc(um(1), um(2), um(3), um(4), um(5), um(6), 0);
    t0 = asils.models.timescales.convert_utc(W.epoch(1), W.epoch(2), W.epoch(3), W.epoch(4), W.epoch(5), W.epoch(6), 0);
    [~, ctp] = asils.models.earthframes.eci2ecef_gmst(tp.gmst_rad, 0, 0);
    [~, ctm] = asils.models.earthframes.eci2ecef_gmst(tm.gmst_rad, 0, 0);
    c0 = asils.models.earthframes.eci2ecef_gmst(t0.gmst_rad, 0, 0);
    W.omega_eci = asils.models.earthframes.earth_rate_from(ctp, ctm, c0, d);
end
