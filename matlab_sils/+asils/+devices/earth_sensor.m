function [n_meas, valid] = earth_sensor(nadir_B, D, s)
%ASILS.DEVICES.EARTH_SENSOR  Static IR Earth sensor MODEL (SYN-ES-1): the nadir
%   unit vector in the body frame with a mount bias and white noise, valid
%   while nadir lies within the detector field (the limb crossings stay on the
%   thermopile array). The unit's own processing -- limb points to a horizon
%   fit -- is the component chain asils.comp.earth_sensor (in-house, to come);
%   this model stands in for the unit's output.
    valid = acos(max(-1, min(1, s.boresight'*nadir_B))) < s.fov_rad;
    if ~valid, n_meas = [0;0;0]; return, end
    e = s.noise*randn(3,1);
    n_meas = asils.quat.dcm(asils.quat.mult(D.bias, asils.quat.fromrotvec(e)))*nadir_B;
    n_meas = n_meas/norm(n_meas);
end
