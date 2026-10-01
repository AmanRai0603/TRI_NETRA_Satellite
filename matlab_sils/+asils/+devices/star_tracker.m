function [q_meas, valid, D] = star_tracker(q_true, t, w_true, sun_B, moon_B, nadir_B, earth_ang, D, s)
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
    smear = 1 + sqrt(w_true'*w_true)/s.noise_rate_ref;    % rate-dependent noise (star smear)
    Rold = asils.quat.dcm(q_old);
    for h = 1:nh
        bs = s.boresight(:, h);
        % the Sun or the Moon in its exclusion cone blinds the head, and it
        % stays blind for blind_s after the body leaves the cone
        dazzled = acos(max(-1,min(1, bs'*sun_B))) <= s.sun_excl || acos(max(-1,min(1, bs'*moon_B))) <= s.moon_excl;
        if dazzled, D.blind_until(h) = t + s.blind_s; end
        valid(h) = slow && ~D.dead(h) && ~dazzled && t >= D.blind_until(h) ...
            && acos(max(-1,min(1, bs'*nadir_B))) > earth_ang + s.earth_excl;
        dq = asils.quat.mult(D.q_mis(:,h), D.q_bias(:,h));          % mount error of this head
        if strcmp(s.model, 'image')
            % COMPONENT LEVEL: the in-house chain on a rendered frame
            % (asils.comp.star_tracker: render -> centroid -> identify -> QUEST)
            Rbh = head_(bs); Rtrue = Rbh*asils.quat.dcm(dq)';       % true mount of this head
            K = D.K; K.R_head_nominal = Rbh;
            [q_meas(:,h), okh] = asils.comp.star_tracker.chain(q_old, Rtrue, D.cat, K, D.cam);
            valid(h) = valid(h) && okh;
        elseif strcmp(s.model, 'quest')
            % STAR-FIELD MODEL + ALGORITHM: stars inside the FOV (true attitude
            % t - latency), the brightest 12 centroided with noise, identified,
            % and the attitude solved by the q-method (asils.fsw.quest).
            bs_eci = Rold'*bs;
            c = bs_eci'*D.cat.r;
            in = find(c > cos(s.fov));
            [~, o] = sort(D.cat.mag(in)); in = in(o(1:min(12, numel(in))));
            if numel(in) < 3, valid(h) = false; q_meas(:,h) = q_old; continue, end
            Rm = asils.quat.dcm(dq)'*Rold;              % what the mis-mounted head believes
            sc = s.noise_cross*smear*sqrt(8);                 % per-star centroid noise (8 stars ~ spec accuracy)
            bm = zeros(3, numel(in));
            for k = 1:numel(in)
                v = Rm*D.cat.r(:, in(k));
                e = sc*randn(3,1); e = e - (v'*e)*v;
                bm(:,k) = (v + e)/norm(v + e);
            end
            q_meas(:,h) = asils.fsw.quest(bm, D.cat.r(:, in));
        else
            % noise in the sensor frame: cross axes and roll about the boresight
            e = s.noise_cross*smear*randn(3,1);
            e = e - (bs'*e)*bs + s.noise_roll*smear*randn*bs;
            q_meas(:, h) = asils.quat.norm(asils.quat.mult(q_old, asils.quat.mult(dq, asils.quat.fromrotvec(e))));
        end
    end
end

function R = head_(bs)
%HEAD_  Nominal body -> head rotation: head z on the boresight.
    z = bs/norm(bs); x = cross(z, [0; 0; 1]); if norm(x) < 1e-6, x = cross(z, [1; 0; 0]); end
    x = x/norm(x); y = cross(z, x);
    R = [x'; y'; z'];
end
