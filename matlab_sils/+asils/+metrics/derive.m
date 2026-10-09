function rec = derive(rec)
%ASILS.METRICS.DERIVE  The derived channels every metric reads, sample by sample (the engine's metrics.rs derive, in
%   MATLAB): kpi_metric_channels's (asils.models.kpichannels: the reference the flight software's guidance gives in each
%   sample's mode, with its yaw flip carried; the errors and their small-angle vectors; the rate, the Sun's angle, the spin
%   and the rate stability) and the power system's (design_power_system: asils.models.powersys). This file walks the record.
%   ape_3ax, ape_los, ake_3ax, ake_los [deg]; rks [deg/s]; rate, spin_z, rate_err [deg/s]; sun_angle, sun_angle_geo [deg];
%   e_vec, e_ake: the small-angle performance and knowledge error vectors [rad, body]; P_gen [W], soc.
%   Owner: Agastya. Copyright (c) 2026 Agastya. All rights reserved.
    n = numel(rec.t);
    P = rec.P; p = P.fsw; bs = P.dev.boresight; sa = P.dev.sun_axis;
    MT = asils.fsw.modes();
    rec.e_vec = nan(3, n); rec.e_ake = nan(3, n);
    rec.ape_3ax = nan(1, n); rec.ape_los = nan(1, n); rec.ake_3ax = nan(1, n); rec.ake_los = nan(1, n);
    rec.rate = nan(1, n); rec.spin_z = nan(1, n); rec.sun_angle = nan(1, n); rec.sun_angle_geo = nan(1, n);
    flip = false;
    for j = 1:n
        r = rec.r(:, j); v = rec.v(:, j); se = rec.sun_eci(:, j); q = rec.q(:, j);
        flip = asils.models.kpichannels.kpi_yaw_flip(p.gd_yaw_flip ~= 0, r, v, p.gd_q_off, sa, bs, se, flip, p.gd_flip_hyst);
        m = rec.mode(j);
        kind = -1; if m >= 1 && m <= numel(MT.guid), kind = MT.guid(m); end
        [has, qr] = asils.models.kpichannels.kpi_reference(kind, r, v, rec.t(j), p.gd_q_off, p.gd_roll_deg, p.gd_t0, p.gd_T, p.gd_axis, ...
                                                           p.gd_q_inertial, sa, bs, se, flip);
        if has
            [rec.e_vec(:, j), rec.ape_los(j)] = asils.models.kpichannels.kpi_error(q, qr, bs);
            rec.ape_3ax(j) = asils.models.kpichannels.kpi_three_axis(q, qr);
        end
        qe = rec.q_est(:, j);
        if all(isfinite(qe))
            [rec.e_ake(:, j), rec.ake_los(j)] = asils.models.kpichannels.kpi_error(q, qe, bs);
            rec.ake_3ax(j) = asils.models.kpichannels.kpi_three_axis(q, qe);
        end
        [rec.rate(j), rec.spin_z(j)] = asils.models.kpichannels.kpi_rates(rec.w(:, j));
        [rec.sun_angle_geo(j), rec.sun_angle(j)] = asils.models.kpichannels.kpi_sun_angle(sa, rec.sun_body(:, j), rec.nu(j));
    end
    rec.rate_err = nan(1, n);
    rec.rks = nan(1, n);
    lag = asils.models.kpichannels.kpi_rks_lag(P.sim.record_dt);
    for j = lag + 1:n
        rec.rks(j) = asils.models.kpichannels.kpi_rks(rec.e_vec(:, j), rec.e_vec(:, j - lag), lag, P.sim.record_dt);
    end
    % the power budget: design_power_system's arrays, and the battery taking what they give less the load
    rec.P_gen = nan(1, n); rec.soc = nan(1, n);
    ps = power_system(P.case.v);
    if ~isempty(ps)
        e = asils.models.powersys.battery_start(ps.soc0, ps.batt_wh);
        for j = 1:n
            rec.P_gen(j) = asils.models.powersys.array_power(ps.area, ps.eff, asils.models.powersys.power_sun_unit(rec.sun_body(:, j)), rec.nu(j));
            if j > 1
                load = asils.models.powersys.power_load(ps.load_w, rec.P_mtq(j), rec.P_rw(j), rec.P_rcs(j));
                e = asils.models.powersys.battery_step(e, rec.P_gen(j), load, rec.t(j - 1), rec.t(j), ps.batt_wh);
            end
            rec.soc(j) = asils.models.powersys.battery_soc(e, ps.batt_wh);
        end
    end
end

function ps = power_system(v)
% the case's power system (section power), all or none (the engine's metrics::PowerSystem::from)
    k = {'power_area_px', 'power_area_mx', 'power_area_py', 'power_area_my', 'power_area_pz', 'power_area_mz', ...
         'power_eff', 'power_batt_wh', 'power_load_w', 'power_soc0'};
    ps = [];
    if ~all(cellfun(@(x) isfield(v, x) && isfinite(v.(x)), k)), return, end
    ps = struct('area', cellfun(@(x) v.(x), k(1:6)).', 'eff', v.power_eff, 'batt_wh', v.power_batt_wh, ...
                'load_w', v.power_load_w, 'soc0', v.power_soc0);
end
