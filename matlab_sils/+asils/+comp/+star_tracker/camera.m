function cam = camera(fov_half_rad)
%ASILS.COMP.STAR_TRACKER.CAMERA  Detector and optics of the in-house star
%   tracker (baseline values until the unit is designed): 1024 x 1024 pixels,
%   focal length from the field of view, Gaussian PSF (1.2 px), a magnitude-6
%   star gives 3000 e-, background 50 e- with 8 e- read noise.
    if nargin < 1, fov_half_rad = 0.17; end
    cam = struct('n', 1024, 'fov', fov_half_rad, 'psf_px', 1.2, 'flux0', 3000, 'bg', 50, 'read_noise', 8, ...
                 'k_sigma', 5, 'max_spots', 20, 'id_tol_rad', 2e-4, 'mag_tol', 0.25, 'fit_tol_rad', 1e-4);
    cam.f = (cam.n/2)/tan(fov_half_rad);             % focal length [px]
    cam.c = (cam.n + 1)/2;                           % principal point
end
