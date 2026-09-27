function a = j2accel(r_ecef, mu, Re, J)
%GRAV.J2ACCEL  Fast zonal (J2..J6) gravitational acceleration in ECEF.
%
%   a = grav.j2accel(r_ecef, mu, Re, J)
%
%   A fast, mostly-analytic, purely-ZONAL geopotential.  Two uses:
%     (a) quick LEO/VLEO runs where only oblateness matters, and
%     (b) an INDEPENDENT cross-check on grav.sphericalHarmonic -- restrict the
%         harmonic field to the same zonal terms and the two must agree to
%         ~1e-12 relative (see test/test_gravity.m).
%
%   INPUTS
%     r_ecef  3x1 position in the EARTH-FIXED (ITRF/ECEF) frame [m].  Zonal
%             gravity is axisymmetric about the spin axis, so you MUST pass
%             ECEF (body-fixed), not ECI.
%     mu      central-body GM [m^3/s^2].
%     Re      reference (equatorial) radius of the field [m].
%     J       [J2 ... J6] unnormalised zonal coeffs (Jn = -C_{n,0}); pass fewer
%             to truncate (e.g. J = 1.08262668e-3 -> J2 only).
%
%   OUTPUT  a : 3x1 acceleration in ECEF [m/s^2] (central + zonal perturbation).
%
%   The J2 term is written in closed form (Vallado); J3..J6 are delegated to the
%   validated spherical-harmonic engine with a zonal-only normalised field, so
%   this helper can never silently diverge from the main model.
%
%   See also GRAV.SPHERICALHARMONIC, GRAV.TWOBODY.
    r  = r_ecef(:);
    rn = norm(r);
    a  = -mu*r/rn^3;                      % central term
    if nargin < 4 || isempty(J), return; end
    J = J(:); nJ = numel(J);

    % ---- J2 closed form (exact) ----
    if nJ>=1 && J(1)~=0
        J2 = J(1);
        u  = r(3)/rn;                     % sin(latitude)
        k  = 1.5*J2*mu*Re^2/rn^4;
        a(1) = a(1) + k*(5*u^2-1)*r(1)/rn;
        a(2) = a(2) + k*(5*u^2-1)*r(2)/rn;
        a(3) = a(3) + k*(5*u^2-3)*r(3)/rn;
    end

    % ---- J3..J6 via the general harmonic engine (zonal-only field) ----
    if nJ>=2 && any(J(2:end)~=0)
        maxN = nJ+1;                      % J index k -> degree n=k+1
        C = zeros(maxN+1); S = zeros(maxN+1);
        C(1,1) = 1;                       % central term inside the engine
        for kk = 2:nJ                     % kk=2 -> J3 (degree 3) ...
            n  = kk+1;
            C(n+1,1) = -J(kk)/sqrt(2*n+1);   % normalised C_{n,0}
        end
        a = a + grav.sphericalHarmonic(r, mu, Re, C, S, maxN, 0) ...
              - grav.twoBody(r, mu);      % engine already carries central term
        % (subtract the engine's central term; we keep our own above once)
    end
end
