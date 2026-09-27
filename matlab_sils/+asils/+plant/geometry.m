function M = geometry(A0, G, gi)
%ASILS.PLANT.GEOMETRY  Momentum-device geometry for asils.plant.deriv.
%   A0 (3 x nr) rotor spin axes at zero gimbal angle; G (3 x ng) gimbal axes;
%   gi (1 x nr) gimbal index per rotor (0 = fixed-axis rotor: wheel or ring).
    if nargin < 2, G = zeros(3,0); end
    if nargin < 3, gi = zeros(1, size(A0,2)); end
    M.A0 = A0; M.G = G; M.gi = gi; M.nr = size(A0,2); M.ng = size(G,2);
    M.T0 = zeros(size(A0));
    for i = 1:M.nr
        if gi(i) > 0, M.T0(:,i) = asils.util.cross3(G(:,gi(i)), A0(:,i)); end
    end
end
