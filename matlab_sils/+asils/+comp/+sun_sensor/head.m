function p = head()
%ASILS.COMP.SUN_SENSOR.HEAD  Baseline quadrant Sun-sensor geometry (in-house
%   unit to be designed): 1 mm square aperture 0.6 mm above the detector (a
%   +/-40 deg linear range), 0.5 % current noise, valid above 5 % of full Sun.
    p = struct('a', 1.0e-3, 'h', 0.6e-3, 'noise', 0.005, 'min_frac', 0.05);
end
