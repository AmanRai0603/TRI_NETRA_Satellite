function R_b2i = R_lvlh(r, v)
%R_LVLH  Body->ECI rotation for a nadir-pointing (LVLH) attitude.
%   z_body = nadir (-r_hat), y_body = -orbit normal, x_body ~ +velocity.
%   Replace with your ADCS quaternion/DCM when you have real attitude.
    zb = -r(:)/norm(r);
    h  = cross(r(:), v(:)); yb = -h/norm(h);
    xb = cross(yb, zb); xb = xb/norm(xb);
    R_b2i = [xb, yb, zb];
end
