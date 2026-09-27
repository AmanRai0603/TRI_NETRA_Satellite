function a = tidal(rSat, rBody, GM)
%THIRDBODY.TIDAL  Leading-order (degree-2) tidal/gradient approximation.
%   a = -GM/|rBody|^3 * ( rSat - 3 (rSat.rHat) rHat ),  rHat = rBody/|rBody|
%   The differential ("tidal") acceleration: stretch along the body direction,
%   compress perpendicular. Relative error ~ |rSat|/|rBody| (2% Moon, 6e-5 Sun
%   at LEO). Great for insight and quick estimates; not for precise OD.
    s = rSat(:); b = rBody(:); rb = norm(b); rh = b/rb;
    a = -GM/rb^3 * ( s - 3*dot(s,rh)*rh );
end
