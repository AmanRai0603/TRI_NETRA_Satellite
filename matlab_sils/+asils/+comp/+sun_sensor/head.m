function p = head(v)
%ASILS.COMP.SUN_SENSOR.HEAD  Quadrant Sun-sensor geometry. head() gives the
%   baseline (in-house unit to be designed): 1 mm square aperture 0.6 mm above
%   the detector (a +/-40 deg linear range), 0.5 % current noise, valid above
%   5 % of full Sun. head(v) takes every value from v, the part's nominal values
%   as asils.product.load reads them for the 'chain' level (aperture_side_m,
%   aperture_height_m, current_noise_frac, current_min_frac).
    if nargin < 1
        v = struct('aperture_side_m', 1.0e-3, 'aperture_height_m', 0.6e-3, 'current_noise_frac', 0.005, 'current_min_frac', 0.05);
    end
    p = struct('a', v.aperture_side_m, 'h', v.aperture_height_m, 'noise', v.current_noise_frac, 'min_frac', v.current_min_frac);
end
