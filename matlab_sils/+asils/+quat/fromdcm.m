function q = fromdcm(R)
%ASILS.QUAT.FROMDCM  q_B/A from passive DCM R_A->B (Shepperd). Ported from quat.fromDcm.
    tr = trace(R);
    [~, idx] = max([tr, R(1,1), R(2,2), R(3,3)]);
    switch idx
        case 1, V = [R(2,3)-R(3,2); R(3,1)-R(1,3); R(1,2)-R(2,1); 1+tr];
        case 2, V = [1+2*R(1,1)-tr; R(1,2)+R(2,1); R(1,3)+R(3,1); R(2,3)-R(3,2)];
        case 3, V = [R(2,1)+R(1,2); 1+2*R(2,2)-tr; R(2,3)+R(3,2); R(3,1)-R(1,3)];
        otherwise, V = [R(3,1)+R(1,3); R(3,2)+R(2,3); 1+2*R(3,3)-tr; R(1,2)-R(2,1)];
    end
    q = asils.quat.norm(V);
end
