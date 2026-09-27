function a = direct(rSat, rBody, GM)
%THIRDBODY.DIRECT  Textbook point-mass third-body acceleration (Earth-centred).
%   a = GM*[ (rBody-rSat)/|rBody-rSat|^3 - rBody/|rBody|^3 ]   [m/s^2]
%   First term: body pulls the satellite. Second (indirect): body pulls Earth;
%   subtracted because the frame is Earth-centred.
%   SIMPLE and intuitive, but the two O(1/|rBody|^2) terms nearly cancel, so it
%   loses precision at LEO (~2 digits for Moon, ~5 for Sun in double; severe in
%   single precision). Prefer thirdbody.battin for production.
    s = rSat(:); b = rBody(:); d = b - s;
    a = GM*( d/norm(d)^3 - b/norm(b)^3 );
end
