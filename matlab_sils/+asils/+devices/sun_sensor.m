function [s_meas, valid] = sun_sensor(s_true_B, nu, D, s)
%ASILS.DEVICES.SUN_SENSOR  Set of fine sun-sensor heads -> one Sun unit vector (body).
%   The head with the Sun closest to its boresight answers if the Sun is
%   inside its FOV and the spacecraft is sunlit (nu > 0.5). Noise and head bias
%   as small rotations (SYN-SUN-1). Standard Code sens.sunSensor modelled one
%   head's alpha/beta angles; this returns the vector the FSW actually uses.
    valid = false; s_meas = [0;0;0];
    if nu < 0.5, return, end
    c = s_true_B'*D.n;
    [cm, j] = max(c);
    if cm < cos(s.fov_rad), return, end
    if isfield(s, 'level') && strcmp(s.level, 'chain')
        % COMPONENT LEVEL: quadrant currents -> angles (asils.comp.sun_sensor)
        nz = D.n(:,j); x = cross(nz, [0;0;1]); if norm(x) < 1e-6, x = cross(nz, [1;0;0]); end
        x = x/norm(x); Rh = [x'; cross(nz, x)'; nz'];
        sb = asils.quat.dcm(asils.quat.fromrotvec(D.bias(:,j)))*s_true_B;
        p = s.head;                                  % the part's aperture, height, noise, threshold
        [sh, valid] = asils.comp.sun_sensor.angles(asils.comp.sun_sensor.currents(Rh*sb, p), p);
        s_meas = Rh'*sh; return
    end
    e = D.bias(:,j) + s.noise*randn(3,1);
    s_meas = asils.quat.dcm(asils.quat.fromrotvec(e)) * s_true_B;
    s_meas = s_meas/sqrt(s_meas'*s_meas);
    valid = true;
end
