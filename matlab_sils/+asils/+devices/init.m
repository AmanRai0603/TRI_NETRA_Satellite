function U = init(dev, seed)
%ASILS.DEVICES.INIT  The units a product carries, each drawn once per run (its dispersion), in the engine's fixed order
%   (adcs-sim run.rs Units::new), so a seed gives the twin and the engine the same units: sens's and act's methods,
%   generated from the design into +asils/+models (gyro, mag, finesun, sttracker, coilset, earthsensor, css, rotorset,
%   thrusters, starcat), their draws from the run's named streams (asils.devices.stream). What is left here is the
%   order of the draws and the units' state between ticks (the runner's).
%   Owner: Agastya. Copyright (c) 2026 Agastya. All rights reserved.
    % the generated modules' capacities (sttracker's ST_HIST, starcat's N_STARS, gnss's GPS_HIST): the MATLAB translator
    % writes a constant into the functions that use it and gives no accessor of its own (a translator gap, S7.17)
    ST_HIST = 64; N_STARS = 4000; GPS_HIST = 256;
    rs = @(name) asils.devices.stream(seed, name);
    disp_ = rs('dispersion');
    U = struct();
    U.gyro = []; U.sun = []; U.st = []; U.es = []; U.css = []; U.rcs = [];
    if dev.gyro.fitted, [U.gyro, disp_] = asils.models.gyro.gyro_new(dev.gyro.rec, disp_, rs('gyro')); end
    [U.mag, disp_] = asils.models.mag.mag_new(dev.mag.rec, disp_, rs('mag'));
    if dev.sun.fitted, [U.sun, disp_] = asils.models.finesun.sun_new(dev.sun.rec, disp_, rs('sun')); end
    if dev.st.fitted
        [U.st, disp_] = asils.models.sttracker.st_new(dev.st.rec, disp_, rs('st'));
        n = N_STARS;
        U.st_ht = zeros(ST_HIST, 1); U.st_hq = zeros(ST_HIST, 4); U.st_hn = 0;
        U.st_cr = zeros(n, 3); U.st_cm = zeros(n, 1);
        if dev.st.model >= 1, [~, U.st_cr, U.st_cm] = asils.models.starcat.star_catalogue(U.st_cr, U.st_cm); end
    end
    [U.mtq, disp_] = asils.models.coilset.coilset_new(dev.mtq.rec, disp_);
    if dev.es.fitted, [U.es, disp_] = asils.models.earthsensor.es_new(dev.es.rec, disp_, rs('es')); end
    if dev.css.fitted, [U.css, disp_] = asils.models.css.css_new(dev.css.rec, disp_, rs('css')); end
    [U.mex, disp_] = asils.models.rotorset.rotorset_new(dev.mex.rec, disp_, rs('mex'));
    if dev.rcs.fitted, [U.rcs, disp_] = asils.models.thrusters.thrusters_new(dev.rcs.rec, disp_); end %#ok<NASGU>
    if dev.st.fitted && dev.st.model == 2
        % the image model's frame buffers and the onboard pair table of the unit's catalogue, built once per run
        cam = dev.st.cam; nn = cam.n*cam.n; ns = N_STARS;
        [np, U.st_cr] = asils.models.stidentify.st_pair_count(U.st_cr, ns, cam.fov);
        [~, U.st_cr, pi_, pj, pa] = asils.models.stidentify.st_pairs(U.st_cr, ns, cam.fov, zeros(np, 1), zeros(np, 1), ...
            zeros(np, 1), zeros(np, 1), zeros(np, 1), zeros(np, 1));
        U.st_frame = struct('img', zeros(nn, 1), 'work', zeros(nn, 1), 'ia', zeros(nn, 1), 'ib', zeros(nn, 1), 'pi', pi_, 'pj', pj, 'pa', pa);
    end
    U.gps = struct('dead', false, 'g', rs('gps'), 'ht', zeros(GPS_HIST, 1), ...
                   'hr', zeros(GPS_HIST, 3), 'hv', zeros(GPS_HIST, 3), 'hn', 0);
    U.tlm = rs('telemetry');
end
