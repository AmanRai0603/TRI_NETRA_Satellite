function [ecc, inc, Om, w] = vectors2elem(j, e)
%THIRDBODY.SECULAR.VECTORS2ELEM  (j, e) vectors -> ecc, inc, Om, w [rad].
    j=j(:); e=e(:); jn=norm(j); ecc=norm(e);
    inc = acos(max(min(j(3)/jn,1),-1));
    node = cross([0;0;1], j);
    if norm(node) < 1e-12
        Om = 0; nodeu = [1;0;0];
    else
        nodeu = node/norm(node); Om = atan2(node(2), node(1));
    end
    if ecc < 1e-12
        w = 0;
    else
        ehat = e/ecc; ip = cross(j/jn, nodeu);
        w = atan2(dot(ip,ehat), dot(nodeu,ehat));
    end
end
