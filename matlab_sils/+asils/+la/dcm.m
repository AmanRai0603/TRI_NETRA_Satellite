function R = dcm(q)
%ASILS.LA.DCM  The passive ECI -> body matrix of q = [x y z w] (the toolbox's, adcs-sim-core la.rs dcm, term for term).
%   Owner: Agastya. Copyright (c) 2026 Agastya. All rights reserved.
    x = q(1); y = q(2); z = q(3); w = q(4);
    R = [1.0 - 2.0*(y*y + z*z), 2.0*(x*y + z*w), 2.0*(x*z - y*w);
         2.0*(y*x - z*w), 1.0 - 2.0*(x*x + z*z), 2.0*(y*z + x*w);
         2.0*(z*x + y*w), 2.0*(z*y - x*w), 1.0 - 2.0*(x*x + y*y)];
end
