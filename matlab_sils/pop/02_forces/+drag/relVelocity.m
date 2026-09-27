function [vrel, uhat, Vmag] = relVelocity(r, v, wind, omega)
%DRAG.RELVELOCITY  Velocity relative to the co-rotating atmosphere + wind.
%   v_rel = v - (omega x r) - wind.  r,v,wind ECI [m,m/s];
%   omega: scalar [rad/s] about +z, OR a 3x1 ECI rate vector.
    if nargin<4||isempty(omega), omega=7.2921150e-5; end   % matches de440.constants().omega_earth
    if nargin<3||isempty(wind),  wind=[0;0;0];        end
    r=r(:); v=v(:); wind=wind(:);
    % omega: scalar = the naive z-axis rate; 3-vector = the true Earth rate in ECI,
    % which points along the CIP (~168 arcsec off z by 2008, worth ~0.4 m/s here).
    % Accepting both is what lets forces/drag.m give the panel models and the
    % cannonball the SAME co-rotation, instead of one each.
    om = omega(:); if isscalar(om), om = [0;0;om]; end
    vatm = cross(om, r) + wind;
    vrel = v - vatm; Vmag = norm(vrel); uhat = vrel/Vmag;
end
