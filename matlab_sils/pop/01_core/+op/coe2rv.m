function [r, v] = coe2rv(a, e, inc, RAAN, argp, nu, mu)
%OP.COE2RV  Classical orbital elements -> inertial state. The inverse of op.rv2coe.
%
%   [r, v] = op.coe2rv(a, e, inc, RAAN, argp, nu, mu)
%
%   a    semi-major axis [m]
%   e    eccentricity [-]
%   inc  inclination [rad]
%   RAAN right ascension of the ascending node [rad]
%   argp argument of perigee [rad]
%   nu   true anomaly [rad]
%   mu   gravitational parameter [m^3/s^2]
%
%   Returns r, v as 3x1 column vectors in the same inertial frame the elements are
%   expressed in. Angles are RADIANS (op.rv2coe returns radians, so the pair is
%   consistent -- deg2rad at the call site, not here).
%
%   Method: build the state in the perifocal frame (p,q,w), where the maths is
%   trivial, then rotate by Rz(-RAAN)*Rx(-inc)*Rz(-argp). This is the standard
%   construction and is exact for e < 1.
    if nargin < 7, error('op:coe2rv:args','need (a,e,inc,RAAN,argp,nu,mu)'); end
    if e >= 1, error('op:coe2rv:ecc','this form is for closed orbits (e < 1), got e = %g', e); end

    p = a*(1 - e^2);                       % semi-latus rectum
    rmag = p / (1 + e*cos(nu));

    % --- perifocal frame: x toward perigee, y 90 deg along the motion -----------
    r_pf = [rmag*cos(nu); rmag*sin(nu); 0];
    v_pf = sqrt(mu/p) * [-sin(nu); e + cos(nu); 0];

    % --- perifocal -> inertial --------------------------------------------------
    cO = cos(RAAN); sO = sin(RAAN);
    ci = cos(inc);  si = sin(inc);
    cw = cos(argp); sw = sin(argp);
    Q = [ cO*cw - sO*sw*ci,  -cO*sw - sO*cw*ci,   sO*si; ...
          sO*cw + cO*sw*ci,  -sO*sw + cO*cw*ci,  -cO*si; ...
          sw*si,              cw*si,              ci ];
    r = Q * r_pf;
    v = Q * v_pf;
end
