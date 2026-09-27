function q = boresight_offset(bs)
%ASILS.FSW.BORESIGHT_OFFSET  Offset q_B/N that puts the payload boresight bs
%   (body axes) on the nadir axis (+Y) of the guidance frame N:
%   dcm(q) * [0;1;0] = bs, by the shortest rotation.
    ey = [0;1;0]; bs = bs(:)/norm(bs);
    ax = [ey(2)*bs(3)-ey(3)*bs(2); ey(3)*bs(1)-ey(1)*bs(3); ey(1)*bs(2)-ey(2)*bs(1)];
    c = ey'*bs; s = norm(ax);
    if s < 1e-12
        if c > 0, q = [0;0;0;1]; else, q = [1;0;0;0]; end
        return
    end
    k = ax/s; K = [0 -k(3) k(2); k(3) 0 -k(1); -k(2) k(1) 0];
    A = eye(3) + s*K + (1 - c)*(K*K);      % active rotation taking ey to bs
    q = asils.quat.fromdcm(A);
end
