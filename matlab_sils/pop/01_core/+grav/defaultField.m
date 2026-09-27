function F = defaultField()
%GRAV.DEFAULTFIELD  Built-in zonal geopotential (J2..J6), works fully offline.
%
%   F = grav.defaultField()
%
%   Returns a field struct compatible with grav.sphericalHarmonic so the
%   propagator runs out-of-the-box with NO external gravity file:
%     F.mu    GM  [m^3/s^2]
%     F.Re    reference radius [m]
%     F.Cbar  (nmax+1)x(nmax+1) normalised cosine coeffs
%     F.Sbar  (nmax+1)x(nmax+1) normalised sine coeffs (all zero: zonal only)
%     F.nmax  6
%     F.name  'zonal J2-J6 (EGM-consistent)'
%
%   The zonal Jn are the standard EGM values; sectorials/tesserals are zero, so
%   this default captures oblateness and the dominant secular LEO/VLEO effects
%   (nodal & apsidal precession) but not longitude-dependent terms.  For a full
%   field, drop an ICGEM .gfc into gravity_data/ and load it with grav.loadGFC.
%
%   Sign convention: Jn = -C_{n,0} (unnormalised) -> \bar C_{n,0}=-Jn/sqrt(2n+1).
%
%   See also GRAV.LOADGFC, GRAV.SPHERICALHARMONIC.

    % EGM2008 defining constants (also used by the .gfc loader for consistency).
    F.mu = 3.986004415e14;      % m^3/s^2
    F.Re = 6378136.3;           % m

    % Unnormalised zonal harmonics (EGM96/EGM2008 agree to these digits).
    J = [ 1.08262668355e-3;     % J2
         -2.53265648533e-6;     % J3
         -1.61962159137e-6;     % J4
         -2.27296082869e-7;     % J5
          5.40681239107e-7 ];   % J6

    nmax = numel(J) + 1;        % degrees 2..6
    F.nmax = nmax;
    F.Cbar = zeros(nmax+1);
    F.Sbar = zeros(nmax+1);
    F.Cbar(1,1) = 1;            % central term
    for k = 1:numel(J)
        n = k + 1;             % k=1 -> J2 -> degree 2
        F.Cbar(n+1,1) = -J(k) / sqrt(2*n+1);
    end
    F.name = 'zonal J2-J6 (EGM-consistent)';
end
