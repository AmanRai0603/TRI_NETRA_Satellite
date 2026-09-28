function p = head()
%ASILS.COMP.EARTH_SENSOR.HEAD  Baseline static IR Earth-sensor array (in-house
%   unit to be designed): 16 limb crossings around the cone, a +/-80 deg field,
%   0.1 deg horizon-profile noise per crossing.
    p = struct('n', 16, 'fov', 80*pi/180, 'noise', 0.1*pi/180);
end
