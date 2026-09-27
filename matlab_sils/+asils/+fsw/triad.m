function q = triad(b1, b2, r1, r2)
%ASILS.FSW.TRIAD  TRIAD attitude q_B/ECI from two body vectors and their references.
%   b1 (most accurate, e.g. Sun) is kept exactly. Ported from Standard Code ad.triad.
    t1b = b1/norm(b1); t2b = asils.util.cross3(b1, b2); t2b = t2b/norm(t2b); t3b = asils.util.cross3(t1b, t2b);
    t1r = r1/norm(r1); t2r = asils.util.cross3(r1, r2); t2r = t2r/norm(t2r); t3r = asils.util.cross3(t1r, t2r);
    A = [t1b t2b t3b] * [t1r t2r t3r]';      % ECI -> body
    q = asils.quat.fromdcm(A);
end
