function v = ecss(kind, rec, idx, delta_s, separation_s)
%ASILS.METRICS.ECSS  The ECSS-E-ST-60-10C error indices over a metric window (= the engine's
%   metrics::ecss). On the performance error (rpe, mpe, pde) or the knowledge error (rke, mke,
%   kde), three-axis or across the boresight ('_los'). The window is cut into consecutive blocks
%   of delta_s from its first sample:
%     rpe / rke   |e(t) - mean of e over t's block|, every sample          (relative error)
%     mpe / mke   |mean of e over a block|, every block                    (mean error)
%     pde / kde   |mean over block k+s - mean over block k|, s = separation_s / delta_s   (drift)
%   Values in degrees; a block with no finite sample is skipped; a drift needs both blocks.
    los = numel(kind) > 4 && strcmp(kind(end-3:end), '_los');
    base = kind(1:3);
    if any(strcmp(base, {'rke', 'mke', 'kde'})), E = rec.e_ake; else, E = rec.e_vec; end
    v = [];
    if isempty(idx) || ~(delta_s > 0), return, end
    E = E(:, idx);
    if los
        bs = rec.P.dev.boresight(:); bs = bs/norm(bs);
        E = E - bs*(bs'*E);
    end
    t = rec.t(idx);
    blk = floor((t - t(1))/delta_s + 1e-9) + 1;
    nb = blk(end);
    ok = all(isfinite(E), 1);
    M = nan(3, nb);
    for b = 1:nb
        k = ok & blk == b;
        if any(k), M(:, b) = mean(E(:, k), 2); end
    end
    switch base
        case {'rpe', 'rke'}
            k = find(ok & all(isfinite(M(:, blk)), 1));
            v = sqrt(sum((E(:, k) - M(:, blk(k))).^2, 1))*180/pi;
        case {'mpe', 'mke'}
            m = M(:, all(isfinite(M), 1));
            v = sqrt(sum(m.^2, 1))*180/pi;
        case {'pde', 'kde'}
            s = max(1, round(separation_s/delta_s));
            if nb > s
                D = M(:, 1+s:nb) - M(:, 1:nb-s);
                D = D(:, all(isfinite(D), 1));
                v = sqrt(sum(D.^2, 1))*180/pi;
            end
    end
end
