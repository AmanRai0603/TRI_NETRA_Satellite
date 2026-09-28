function [q, ok, id] = attitude(b, id, K, tol)
%ASILS.COMP.STAR_TRACKER.ATTITUDE  Identified stars -> ECI-to-head attitude
%   quaternion (scalar last) by the q-method / QUEST (asils.fsw.quest), with a
%   residual check: the star whose measured direction misses its catalogue
%   star by more than tol after the fit is dropped and the fit repeated (an
%   identification error cannot survive it). ok with three or more left.
    if nargin < 4, tol = 1e-4; end
    q = [0; 0; 0; 1];
    for it = 1:numel(id)
        k = find(id > 0); ok = numel(k) >= 3;
        if ~ok, return, end
        q = asils.fsw.quest(b(:, k), K.r(:, id(k)));
        res = acos(min(1, sum(b(:, k).*(asils.quat.dcm(q)*K.r(:, id(k))), 1)));
        [rm, j] = max(res);
        if rm < tol, return, end
        id(k(j)) = 0;
    end
    ok = false;
end
