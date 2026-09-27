function K = lqr_gain(A, B, Q, R)
%ASILS.FSW.LQR_GAIN  Continuous LQR gain K = R^-1 B' X, X the stabilising
%   solution of A'X + XA - XBR^-1B'X + Q = 0, from the stable invariant
%   subspace of the Hamiltonian (no toolbox needed, MATLAB and Octave).
    n = size(A, 1);
    Hm = [A, -B/R*B'; -Q, -A'];
    [V, L] = eig(Hm);
    [~, idx] = sort(real(diag(L)));
    V = V(:, idx(1:n));
    X = real(V(n+1:end, :)/V(1:n, :));
    K = R\(B'*X);
end
