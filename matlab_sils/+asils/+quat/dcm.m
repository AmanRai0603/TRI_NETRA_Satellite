function R = dcm(q)
%ASILS.QUAT.DCM  Passive DCM R_A->B from q_B/A (scalar-last). Ported from quat.toDcm.
%   v_B = dcm(q_BwrtA) * v_A
    q1 = q(1); q2 = q(2); q3 = q(3); q4 = q(4);
    R = [1-2*(q2^2+q3^2),  2*(q1*q2+q3*q4),  2*(q1*q3-q2*q4);
         2*(q2*q1-q3*q4),  1-2*(q1^2+q3^2),  2*(q2*q3+q1*q4);
         2*(q3*q1+q2*q4),  2*(q3*q2-q1*q4),  1-2*(q1^2+q2^2)];
end
