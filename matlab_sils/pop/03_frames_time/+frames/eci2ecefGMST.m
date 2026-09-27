function [C, Ct] = eci2ecefGMST(gmst_rad, xp, yp)
%FRAMES.ECI2ECEFGMST  Approximate ECI->ECEF rotation from GMST only (no EOP fetch).
%   [C, Ct] = frames.eci2ecefGMST(gmst_rad[, xp, yp])
%     C  : 3x3, r_ecef = C * r_eci
%     Ct : 3x3, r_eci  = Ct * r_ecef  (= C')
%   Uses only Earth rotation about the pole (GMST), optionally polar motion
%   (xp,yp in rad).  It OMITS precession/nutation, so it is accurate only to the
%   ~arcsecond-to-tens-of-arcsecond level (tens of metres at LEO radius).  It is
%   provided so the propagator, examples, and tests RUN OFFLINE with zero data
%   fetch.  For precise work switch cfg.frame.build to 'A' | 'B' | 'C' (full IAU
%   2006/2000A CIO chain with real IERS EOP) -- see frames.eci2ecef.
    g = gmst_rad;
    Rz = [ cos(g)  sin(g) 0;      % rotate ECI -> Earth-fixed about +z by GMST
          -sin(g)  cos(g) 0;
             0       0    1];
    if nargin>=3 && (xp~=0 || yp~=0)
        W = [1 0 xp; 0 1 -yp; -xp yp 1];    % small-angle polar motion
        C = W*Rz;
    else
        C = Rz;
    end
    Ct = C.';
end
