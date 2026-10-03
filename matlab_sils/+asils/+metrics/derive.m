function rec = derive(rec)
%ASILS.METRICS.DERIVE  Error channels every metric reads (ECSS-E-ST-60-10C terms).
%   The TRUE reference is rebuilt from the TRUTH orbit (precision OD, rec.r/v)
%   with the scenario's guidance law -- not from the FSW's own GNSS-based
%   reference -- so APE includes every onboard error.
%   ape_3ax  angle between true attitude and true reference            [deg]
%   ape_los  angle between true payload boresight (product field) and its true target direction [deg]
%   ake_3ax  angle between estimated and true attitude                  [deg]
%   ake_los  knowledge error of the payload boresight                    [deg]
%   rks      rate stability: change of the pointing-error vector over
%            1 s (an exposure) divided by 1 s [deg/s]
%   rate     |w| [deg/s];  rate_err |w - w_ref| [deg/s]
%   e_vec, e_ake  the small-angle performance and knowledge error vectors [rad, body] the ECSS
%            indices (asils.metrics.ecss) are taken on
%   sun_angle angle between the power face (product sun_axis_body, default
%            -Z_B) and the Sun, sunlit samples only [deg]; sun_angle_geo the
%            same at every sample (eclipse included) for time-to metrics
%   spin_z   body rate about +Z_B [deg/s]
    n = numel(rec.t);
    bs = rec.P.dev.boresight;
    gd = rec.P.fsw.guidance;
    MT = asils.fsw.modes(); kinds = MT.guidance;           % guidance law of each controller state ('' = none)
    gd.sun_axis = rec.P.dev.sun_axis; gd.roll_axis = bs;
    rec.q_ref_true = nan(4,n); rec.e_vec = nan(3,n); rec.e_ake = nan(3,n);
    rec.ape_3ax = nan(1,n); rec.ape_los = nan(1,n); rec.ake_3ax = nan(1,n); rec.ake_los = nan(1,n);
    gd.flip = false; flip_on = isfield(rec.P.fsw, 'gd_yaw_flip') && rec.P.fsw.gd_yaw_flip;
    for j = 1:n
        q = rec.q(:,j); qe = rec.q_est(:,j);
        if flip_on                                          % the flight software's yaw flip, same hysteresis
            gd.sun_eci = rec.sun_eci(:,j); gd = asils.fsw.yaw_flip(gd, rec.r(:,j), rec.v(:,j), 0.1);
        end
        b_true = asils.quat.dcm(q)'*bs;
        if ~isempty(kinds{rec.mode(j)})
            gd.sun_eci = rec.sun_eci(:,j);                  % TRUE Sun for the true reference
            [qr, ~] = asils.fsw.guidance(kinds{rec.mode(j)}, rec.r(:,j), rec.v(:,j), rec.t(j), gd);
            rec.q_ref_true(:,j) = qr;
            dq = asils.quat.mult(asils.quat.conj(qr), q); if dq(4) < 0, dq = -dq; end
            rec.e_vec(:,j) = 2*dq(1:3);
            rec.ape_3ax(j) = asils.quat.angle(q, qr)*180/pi;
            b_ref = asils.quat.dcm(qr)'*bs;
            rec.ape_los(j) = acosd(max(-1, min(1, b_true'*b_ref)));
        end
        if all(isfinite(qe))
            dk = asils.quat.mult(asils.quat.conj(qe), q); if dk(4) < 0, dk = -dk; end
            rec.e_ake(:,j) = 2*dk(1:3);
            rec.ake_3ax(j) = asils.quat.angle(q, qe)*180/pi;
            b_est = asils.quat.dcm(qe)'*bs;
            rec.ake_los(j) = acosd(max(-1, min(1, b_true'*b_est)));
        end
    end
    rec.rate = sqrt(sum(rec.w.^2, 1))*180/pi;
    rec.sun_angle_geo = acosd(max(-1, min(1, rec.P.dev.sun_axis'*rec.sun_body)));
    rec.sun_angle = rec.sun_angle_geo; rec.sun_angle(rec.nu < 0.5) = NaN;
    rec.spin_z = rec.w(3,:)*180/pi;
    rec.rate_err = sqrt(sum((rec.w - rec.w_ref).^2, 1))*180/pi;
    lag = max(1, round(1/max(rec.P.sim.record_dt, 1e-9)));
    % the power budget (= the engine's metrics::derive): arrays from the case's power system, the
    % battery taking what they give less the platform and the ADCS, up to full; empty is empty
    rec.P_gen = nan(1, n); rec.soc = nan(1, n);
    ps = power_system_(rec.P.case.v);
    if ~isempty(ps)
        N = [1 -1 0 0 0 0; 0 0 1 -1 0 0; 0 0 0 0 1 -1];
        e = ps.soc0*ps.batt_wh;
        for j = 1:n
            s = rec.sun_body(:, j); s = s/max(norm(s), 1e-30);
            rec.P_gen(j) = 1361*min(max(rec.nu(j), 0), 1)*ps.eff*(ps.area*max(N'*s, 0));
            if j > 1
                load = ps.load_w + rec.P_mtq(j) + rec.P_rw(j) + p_rcs_(rec, j);
                e = min(max(e + (rec.P_gen(j) - load)*(rec.t(j) - rec.t(j-1))/3600, 0), ps.batt_wh);
            end
            rec.soc(j) = e/ps.batt_wh;
        end
    end
    rec.rks = nan(1, n);
    if n > lag
        d = rec.e_vec(:, lag+1:end) - rec.e_vec(:, 1:end-lag);
        rec.rks(lag+1:end) = sqrt(sum(d.^2, 1))*180/pi/(lag*rec.P.sim.record_dt);
    end
end

function ps = power_system_(v)
% the case's power system (section power), all or none (= metrics::PowerSystem::from)
    k = {'power_area_px', 'power_area_mx', 'power_area_py', 'power_area_my', 'power_area_pz', 'power_area_mz', ...
         'power_eff', 'power_batt_wh', 'power_load_w', 'power_soc0'};
    ps = [];
    if ~all(cellfun(@(x) isfield(v, x) && isfinite(v.(x)), k)), return, end
    ps = struct('area', cellfun(@(x) v.(x), k(1:6)), 'eff', v.power_eff, 'batt_wh', v.power_batt_wh, ...
                'load_w', v.power_load_w, 'soc0', v.power_soc0);
end

function p = p_rcs_(rec, j)
    p = 0; if isfield(rec, 'P_rcs') && isfinite(rec.P_rcs(j)), p = rec.P_rcs(j); end
end
