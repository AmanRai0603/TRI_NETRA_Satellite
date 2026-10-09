function q = qnorm(q)
%ASILS.LA.QNORM  q over its norm, [0 0 0 1] for none (the toolbox's, adcs-sim-core la.rs qnorm).
%   Owner: Agastya. Copyright (c) 2026 Agastya. All rights reserved.
    n = sqrt(q(1)*q(1) + q(2)*q(2) + q(3)*q(3) + q(4)*q(4));
    if n < 1e-300, q = [0; 0; 0; 1]; else, q = [q(1)/n; q(2)/n; q(3)/n; q(4)/n]; end
end
