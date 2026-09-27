function [j, e] = elem2vectors(a, ecc, inc, Om, w)
%THIRDBODY.SECULAR.ELEM2VECTORS  Classical elements -> (j, e) vectors.
%   Frame: same inertial frame in which inc/Om are measured (z = reference pole).
    ci=cos(inc); si=sin(inc); cO=cos(Om); sO=sin(Om);
    jhat = [sO*si; -cO*si; ci];                 % orbit normal
    node = [cO; sO; 0];  ip = cross(jhat,node); % node & in-plane perp
    ehat = cos(w)*node + sin(w)*ip;             % pericentre direction
    j = sqrt(1-ecc^2)*jhat;
    e = ecc*ehat;
end
