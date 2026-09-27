function [B_eci_T, B_ecef_T] = field(r_ecef, C_eci2ecef, gh, nmax)
%ASILS.ENV.FIELD  Geomagnetic field [T] at an ECEF position, returned in ECI and ECEF.
%   r_ecef       3x1 [m] truth position (from the in-loop precision orbit)
%   C_eci2ecef   3x3 ECI->ECEF rotation at this epoch (from the POP frame)
%   gh           IGRF coefficients at this epoch (asils.env.igrf_gh)
    x = r_ecef(1); y = r_ecef(2); z = r_ecef(3);
    [lat, lon, alt] = op.geodetic(r_ecef);            % POP WGS84 geodetic
    Bned = asils.env.igrf_ned(gh, lat, lon, alt/1000, nmax) * 1e-9;
    sl = sin(lat); cl = cos(lat); so = sin(lon); co = cos(lon);
    Rn2e = [-sl*co, -so, -cl*co;
            -sl*so,  co, -cl*so;
             cl,     0,  -sl];
    B_ecef_T = Rn2e * Bned;
    B_eci_T = C_eci2ecef' * B_ecef_T;
end
