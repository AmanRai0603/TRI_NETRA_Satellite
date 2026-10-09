function [U, bus, z] = sense(U, dev, bus, x, sky, m_coil, nu, t, k, rt, r, v, jd0, dt, scale)
%ASILS.DEVICES.SENSE  Every sensor sampled and written onto the bus as its device's bytes (registers, UART frames, CAN
%   telemetry), in the order the units draw their noise: the engine's run.rs sense, in MATLAB. The models are sens's and
%   act's methods (+asils/+models: gyro_sample, mag_sample, sun_sample, css_sample, st_history, st_sample, st_image,
%   es_sample, gps_history, gps_delayed, gnss_ecef, gps_sample, rotor_telemetry), the counts the emulators' scaling
%   (asils.models.emucodec); the framing the rig's (asils.hal). z: what the sensors measured (the recorder's view).
%   Owner: Agastya. Copyright (c) 2026 Agastya. All rights reserved.
    bus.now_ns = round(t*1e9);
    if ~isempty(U.gyro)
        [w_meas, U.gyro] = asils.models.gyro.gyro_sample(U.gyro, dev.gyro.rec, x.w, dt);
        bus.gyro = asils.hal.gyro_resp(true, w_meas, scale);
    else
        w_meas = x.w; bus.gyro = [];
    end
    [b_meas, U.mag] = asils.models.mag.mag_sample(U.mag, dev.mag.rec, sky.b_b, m_coil);
    if dev.mag.fitted, bus.mag = asils.hal.regs(true, asils.models.emucodec.mag_counts(b_meas, scale)); else, bus.mag = []; end
    sun_ok = false; s = zeros(3, 1);
    if ~isempty(U.sun)
        [sun_ok, s, U.sun] = asils.models.finesun.sun_sample(U.sun, dev.sun.rec, sky.sb, nu);
    elseif ~isempty(U.css)
        [sun_ok, s, U.css] = asils.models.css.css_sample(U.css, dev.css.rec, sky.sb, nu, sky.nb, sky.earth_ang);
    end
    if ~sun_ok, s = zeros(3, 1); end
    if dev.sun.fitted || dev.css.fitted, bus.sun = asils.hal.regs(sun_ok, asils.models.emucodec.unit_counts(s, scale)); else, bus.sun = []; end
    st_ok = false;
    if ~isempty(U.st)
        [U.st_hn, U.st_ht, U.st_hq] = asils.models.sttracker.st_history(U.st_ht, U.st_hq, U.st_hn, t, x.q);
        if mod(k, rt.st) == 0
            [ok, q, valid, dq, q_old, U.st, U.st_ht, U.st_hq, U.st_cr, U.st_cm] = asils.models.sttracker.st_sample(U.st, dev.st.rec, ...
                U.st_ht, U.st_hq, U.st_hn, U.st_cr, U.st_cm, x.q, t, x.w, sky.sb, sky.mb, sky.nb, sky.earth_ang);
            nh = dev.st.nh;
            if dev.st.model == 2 && any(valid(1:nh))
                % COMPONENT LEVEL: each valid head's chain on a rendered frame (sens_star_image's st_image)
                W = U.st_frame;
                [okc, qc, U.st, U.st_cr, U.st_cm, W.img, W.work, W.ia, W.ib, W.pi, W.pj, W.pa] = asils.models.stimage.st_image(U.st, ...
                    dev.st.rec, dev.st.cam, valid, dq, q_old, U.st_cr, U.st_cm, 4000, W.img, W.work, W.ia, W.ib, W.pi, W.pj, W.pa);
                U.st_frame = W;
                for h = 1:nh, if okc(h), ok(h) = true; q(h, :) = qc(h, :); end, end
            end
            hv = false(nh, 1); hq = repmat([0 0 0 1], nh, 1);
            for h = 1:nh, if ok(h), hv(h) = true; hq(h, :) = q(h, :); st_ok = true; end, end
            bus.uart{2} = [bus.uart{2}, asils.hal.st_frame(hv, hq, nh, scale).'];
        end
    end
    if ~isempty(U.es)
        if mod(k, rt.es) == 0
            [eok, e, U.es] = asils.models.earthsensor.es_sample(U.es, dev.es.rec, sky.nb);
        else
            eok = false; e = zeros(3, 1);
        end
        if ~eok, e = zeros(3, 1); end
        bus.es = asils.hal.regs(eok, asils.models.emucodec.unit_counts(e, scale));
    end
    G = U.gps;
    if dev.gps.fitted
        [G.hn, G.ht, G.hr, G.hv] = asils.models.gnss.gps_history(G.ht, G.hr, G.hv, G.hn, dev.gps.latency, t, r, v);
    end
    if dev.gps.fitted && mod(k, rt.gps) == 0 && ~G.dead
        % a receiver fixes in ECEF (WGS-84); the fix it reports now solves for the epoch `latency` ago
        [te, rd, vd] = asils.models.gnss.gps_delayed(G.ht, G.hr, G.hv, G.hn, dev.gps.latency, t);
        [re, ve] = asils.models.gnss.gnss_ecef(jd0, te, rd, vd);
        [rg, vg, G.g] = asils.models.gnss.gps_sample(re, ve, dev.gps.pos_sigma, dev.gps.vel_sigma, G.g);
        bus.uart{3} = [bus.uart{3}, asils.hal.gps_frame(true, rg, vg, scale).'];
    end
    U.gps = G;
    % the rotors' telemetry: act_rotor_telemetry's rotor_telemetry, act's stated noises, from the telemetry stream
    [hm, dm, U.tlm] = asils.models.rotortlm.rotor_telemetry(dev.mex.rec.n, x.h, x.d, dev.mex.rec.gi, dev.rotor_tlm_noise, ...
                                                            dev.gimbal_tlm_noise, U.tlm);
    nr = dev.mex.rec.n;
    if nr > 0
        f = zeros(nr, 10);
        for i = 1:nr, f(i, :) = asils.hal.rotor_tm(i - 1, hm(i), dm(i), scale); end
        bus.can_rx = [bus.can_rx; f];
    end
    z = struct('w_meas', w_meas, 'b_meas', b_meas, 'sun_ok', sun_ok, 'st_ok', st_ok);
end
