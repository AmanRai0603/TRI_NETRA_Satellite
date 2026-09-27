function [m, tau_ach] = torque2dipole(tau_cmd, B, m_max)
%ASILS.FSW.TORQUE2DIPOLE  Desired body torque -> magnetorquer dipole (minimum norm).
%   PORTED from Standard Code act.torque2dipole: m = (B x tau)/|B|^2, then the
%   achieved torque m x B (the along-B part is unreachable). CHANGE: uniform
%   (direction-preserving) scaling instead of per-axis clipping, so the
%   achieved torque stays perpendicular to B and parallel to the request.
    Bs = B'*B;
    if Bs < 1e-18, m = zeros(3,1); tau_ach = m; return, end
    m = asils.util.cross3(B, tau_cmd)/Bs;
    a = min(1, min(m_max ./ max(abs(m), 1e-30)));
    m = a*m;
    tau_ach = asils.util.cross3(m, B);
end
