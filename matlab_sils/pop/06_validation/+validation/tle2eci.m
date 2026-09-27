function [r,v] = tle2eci(tle, mu, targetEpoch)
%VALIDATION.TLE2ECI  Osculating ECI state from TLE mean elements.
%   [r,v] = validation.tle2eci(tle[, mu[, targetEpoch]])
%   Converts the Keplerian mean elements to a Cartesian state (TEME frame, here
%   treated as ECI to within the ~arcsec TEME<->J2000 rotation).  This is an
%   APPROXIMATE seed: it ignores the SGP4 short-period corrections.
%   If targetEpoch ([Y Mo D H Mi S] UTC) is given, the mean elements are advanced
%   from the TLE epoch to that time with J2 secular rates (RAAN, argp, M) so the
%   state is at the SAME instant you compare against -- otherwise a TLE evaluated
%   at its own epoch differs from a GPS state at another time mostly by orbital
%   PHASE (hundreds of km), which is not a real accuracy difference.
    if nargin<2||isempty(mu), K=de440.constants(); mu=K.mu_earth; end
    n = tle.n*2*pi/86400;                             % rad/s
    a = (mu/n^2)^(1/3);
    e = tle.ecc; inc = tle.inc;
    M = tle.M; raan = tle.raan; argp = tle.argp;
    if nargin>=3 && ~isempty(targetEpoch)
        dt = (datenum(targetEpoch) - datenum(tle.epoch))*86400;   % s from TLE epoch
        Kc = de440.constants(); Re = Kc.Re_earth; J2 = 1.0826269e-3;
        p = a*(1-e^2); f = 1.5*J2*(Re/p)^2*n;
        raan = raan - f*cos(inc)*dt;                              % nodal regression
        argp = argp + f*(2 - 2.5*sin(inc)^2)*dt;                  % apsidal precession
        M    = M    + (n + f*sqrt(1-e^2)*(1 - 1.5*sin(inc)^2))*dt;% mean-anomaly advance
    end
    % solve Kepler M -> E
    E=M;
    for it=1:100, dE=(E-e*sin(E)-M)/(1-e*cos(E)); E=E-dE; if abs(dE)<1e-13, break; end, end
    nu = 2*atan2(sqrt(1+e)*sin(E/2), sqrt(1-e)*cos(E/2));
    p = a*(1-e^2); rmag = p/(1+e*cos(nu));
    r_pf = [rmag*cos(nu); rmag*sin(nu); 0];
    v_pf = sqrt(mu/p)*[-sin(nu); e+cos(nu); 0];
    Rz=@(t)[cos(t) -sin(t) 0; sin(t) cos(t) 0; 0 0 1];
    Rx=@(t)[1 0 0; 0 cos(t) -sin(t); 0 sin(t) cos(t)];
    Q = Rz(raan)*Rx(inc)*Rz(argp);
    r = Q*r_pf;  v = Q*v_pf;
end
