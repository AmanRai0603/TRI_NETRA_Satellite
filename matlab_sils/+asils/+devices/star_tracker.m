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
    for h = 1:nh
        bs = s.boresight(:, h);
        valid(h) = slow && acos(max(-1,min(1, bs'*sun_B))) > s.sun_excl ...
            && acos(max(-1,min(1, bs'*nadir_B))) > earth_ang + s.earth_excl;
        % noise in the sensor frame: cross axes and roll about the boresight
        e = s.noise_cross*randn(3,1);
        e = e - (bs'*e)*bs + s.noise_roll*randn*bs;
        dq = asils.quat.mult(D.q_mis(:,h), asils.quat.mult(D.q_bias(:,h), asils.quat.fromrotvec(e)));
        q_meas(:, h) = asils.quat.norm(asils.quat.mult(q_old, dq));
    end
end
