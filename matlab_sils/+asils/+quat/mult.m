function q = mult(a, b)
%ASILS.QUAT.MULT  Hamilton product a (x) b, scalar-LAST [x;y;z;w].
%   Ported from TRI-NETRa Standard Code quat.multiply (same convention):
%   dcm(mult(a,b)) = dcm(b)*dcm(a), so q_C/A = mult(q_B/A, q_C/B).
    q = [a(4)*b(1) + a(1)*b(4) + a(2)*b(3) - a(3)*b(2);
         a(4)*b(2) - a(1)*b(3) + a(2)*b(4) + a(3)*b(1);
         a(4)*b(3) + a(1)*b(2) - a(2)*b(1) + a(3)*b(4);
         a(4)*b(4) - a(1)*b(1) - a(2)*b(2) - a(3)*b(3)];
end
