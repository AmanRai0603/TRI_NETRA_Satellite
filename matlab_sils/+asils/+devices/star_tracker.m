function [q_meas, valid, D] = star_tracker(q_true, t, w_true, sun_B, nadir_B, earth_ang, D, s)
%ASILS.DEVICES.STAR_TRACKER  Star-tracker heads -> attitude q_B/ECI per head (4 x nh)
%   and validity (1 x nh), with latency, noise and exclusion angles.
%   Noise: cross-boresight and roll about the boresight (SYN-ST-1), plus a
%   constant bias and mount misalignment. Validity: body rate below max_rate,
%   Sun and Earth-limb exclusion angles. LATENCY: the output is the attitude
%   latency_s ago (history kept here); the FSW compensates with the gyro,
%   as Standard Code sdp.starTrackerLatency does.
    % D.hist_t / D.hist_q: true attitude recorded EVERY plant tick by the
    % loop (asils.devices.st_history); interpolate exactly at t - latency.
    tl = t - s.latency;
    k = find(D.hist_t <= tl + 1e-9, 1, 'last');
    if isempty(k)
        q_old = q_true;
    elseif k == numel(D.hist_t)
        q_old = D.hist_q(:, k);
    else
        a = (tl - D.hist_t(k))/(D.hist_t(k+1) - D.hist_t(k));
        q_old = asils.quat.slerp(D.hist_q(:, k), D.hist_q(:, k+1), a);
    end
    nh = size(s.boresight, 2); q_meas = zeros(4, nh); valid = false(1, nh);
    slow = sqrt(w_true'*w_true) < s.max_rate;
    Rold = asils.quat.dcm(q_old);
    for h = 1:nh
        bs = s.boresight(:, h);
        valid(h) = slow && ~D.dead(h) && acos(max(-1,min(1, bs'*sun_B))) > s.sun_excl ...
            && acos(max(-1,min(1, bs'*nadir_B))) > earth_ang + s.earth_excl;
        dq = asils.quat.mult(D.q_mis(:,h), D.q_bias(:,h));          % mount error of this head
        if strcmp(s.model, 'quest')
            % STAR-FIELD MODEL + ALGORITHM: stars inside the FOV (true attitude
            % t - latency), the brightest 12 centroided with noise, identified,
            % and the attitude solved by the q-method (asils.fsw.quest).
            bs_eci = Rold'*bs;
            c = bs_eci'*D.cat.r;
            in = find(c > cos(s.fov));
            [~, o] = sort(D.cat.mag(in)); in = in(o(1:min(12, numel(in))));
            if numel(in) < 3, valid(h) = false; q_meas(:,h) = q_old; continue, end
            Rm = asils.quat.dcm(dq)'*Rold;              % what the mis-mounted head believes
            sc = s.noise_cross*sqrt(8);                 % per-star centroid noise (8 stars ~ spec accuracy)
            bm = zeros(3, numel(in));
            for k = 1:numel(in)
                v = Rm*D.cat.r(:, in(k));
                e = sc*randn(3,1); e = e - (v'*e)*v;
                bm(:,k) = (v + e)/norm(v + e);
            end
            q_meas(:,h) = asils.fsw.quest(bm, D.cat.r(:, in));
        else
            % noise in the sensor frame: cross axes and roll about the boresight
            e = s.noise_cross*randn(3,1);
            e = e - (bs'*e)*bs + s.noise_roll*randn*bs;
            q_meas(:, h) = asils.quat.norm(asils.quat.mult(q_old, asils.quat.mult(dq, asils.quat.fromrotvec(e))));
        end
    end
end
