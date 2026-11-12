function W = world(W, env)
%ASILS.ORBIT.WORLD  What the in-loop orbit's force model is handed once per run (the engine's adcs-pop World::new): W
%   holds the force set (W.fs), the epoch (W.epoch, UTC [Y Mo D H Mi S]) and the spacecraft (W.sc); env the run's
%   space weather (F10.7, its mean, Kp, ap). Added: the indices as the density takes them (swindex.from_manual), the
%   default field and its spherical-harmonic workspace (gravity.default_field, sph_setup), the Earth's rate in the
%   inertial frame (earthframes.earth_rate_from over the frames of the build 15 s either side of the epoch, op.buildWorld's
%   earthRateECI), the Earth frame build (env_earth_frame_build) and its EOP and whether a force needs the ephemeris.
%   Owner: Agastya. Copyright (c) 2026 Agastya. All rights reserved.
    fs = W.fs;
    if fs.erp_on || fs.relativity_on || fs.solidtides || fs.oceantides
        error('asils:orbit', 'the force set names Earth radiation, relativity or tides: the twin''s orbit flies none of them');
    end
    W.build = 'Gmst';                                       % the Earth-frame build (the engine's frames::Build)
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

    % the Earth frame (env_earth_frame_build: 0 Gmst, 1 build A with the EOP of env_eop_snapshot) and the Earth's rate in the
    % inertial frame, from the frames 15 s either side of the epoch (op.buildWorld's earthRateECI): a frame the build cannot make
    % is an error, never [0; 0; omega] (P8c)
    if env.frame_build == 0, W.build = 'Gmst';
    elseif env.frame_build == 1, W.build = 'A'; W.eop = asils.orbit.eop_snapshot();
    else, error('asils:orbit', 'env_earth_frame_build = %d: build B and C need the EOP 20 C04 series, which the snapshot does not hold', env.frame_build);
    end
    d = 15;
    up = asils.models.timescales.addsec(W.epoch, d); um = asils.models.timescales.addsec(W.epoch, -d);
    ctp = asils.orbit.eci2ecef(W, up)'; ctm = asils.orbit.eci2ecef(W, um)'; c0 = asils.orbit.eci2ecef(W, W.epoch);
    W.omega_eci = asils.models.earthframes.earth_rate_from(ctp, ctm, c0, d);
end
