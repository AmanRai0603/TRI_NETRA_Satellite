function [q, loss] = quest(b, r, w)
%ASILS.FSW.QUEST  Optimal attitude q_B/ECI from matched unit-vector pairs
%   (star-tracker algorithm): Davenport's q-method, the exact solution QUEST
%   approximates (Shuster & Oh 1981; Markley & Crassidis 2014, 5.3).
%   b (3 x n) body/sensor vectors, r (3 x n) catalogue vectors, w weights.
%   Wahba loss = sum w (1 - b'A r); A = asils.quat.dcm(q).
    if nargin < 3, w = ones(1, size(b,2)); end
    B = zeros(3); for i = 1:size(b,2), B = B + w(i)*b(:,i)*r(:,i)'; end
    S = B + B'; sg = trace(B);
    zv = [B(2,3) - B(3,2); B(3,1) - B(1,3); B(1,2) - B(2,1)];
    K = [S - sg*eye(3), zv; zv', sg];
    [V, L] = eig(K);
    [lmax, i] = max(diag(L));
    q = asils.quat.norm(V(:, i));
    loss = sum(w) - lmax;
end
