function [id, ok] = identify(b, mag, K, cam)
%ASILS.COMP.STAR_TRACKER.IDENTIFY  Spots -> catalogue ids (BASELINE method:
%   pair-angle voting with a magnitude gate, Kosik 1991; in-house algorithm to
%   come). Every measured pair votes for the catalogue pairs of the same angle
%   (within id_tol) whose magnitudes both match (within mag_tol); each spot
%   takes its most-voted catalogue star; the assignment is then VERIFIED: a
%   star is kept only if its angles to at least two other kept stars match the
%   catalogue. ok when three or more stars survive.
%   b: 3 x n unit vectors (head frame), mag: 1 x n estimated magnitudes.
    n = size(b, 2); id = zeros(1, n); ok = false;
    if n < 3, return, end
    V = sparse(n, numel(K.mag));
    for p = 1:n-1
        for q = p+1:n
            th = acos(min(1, b(:,p)'*b(:,q)));
            lo = find(K.ang >= th - cam.id_tol_rad, 1); hi = find(K.ang <= th + cam.id_tol_rad, 1, 'last');
            if isempty(lo) || isempty(hi) || hi < lo, continue, end
            ci = K.i(lo:hi); cj = K.j(lo:hi);
            m1 = abs(K.mag(ci) - mag(p)) < cam.mag_tol & abs(K.mag(cj) - mag(q)) < cam.mag_tol;   % p->i, q->j
            m2 = abs(K.mag(cj) - mag(p)) < cam.mag_tol & abs(K.mag(ci) - mag(q)) < cam.mag_tol;   % p->j, q->i
            ci = ci(:)'; cj = cj(:)';
            a = [ci(m1), cj(m2)]; bq = [cj(m1), ci(m2)];
            if isempty(a), continue, end
            V = V + sparse([p*ones(1, numel(a)), q*ones(1, numel(bq))], [a, bq], 1, n, numel(K.mag));
        end
    end
    [v, cand] = max(V, [], 2); cand = full(cand)'; cand(full(v)' == 0) = 0;
    keep = cand > 0;
    for it = 1:3                                   % verification by mutual angles
        for p = find(keep)
            good = 0;
            for q = find(keep & (1:n) ~= p)
                th = acos(min(1, b(:,p)'*b(:,q))); tc = acos(min(1, K.r(:,cand(p))'*K.r(:,cand(q))));
                good = good + (abs(th - tc) < 3*cam.id_tol_rad);
            end
            if good < 2, keep(p) = false; end
        end
    end
    id(keep) = cand(keep); ok = nnz(keep) >= 3;
end
