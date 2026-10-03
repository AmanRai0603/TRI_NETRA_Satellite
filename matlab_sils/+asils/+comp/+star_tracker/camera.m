function cam = camera(fov_half_rad, v)
%ASILS.COMP.STAR_TRACKER.CAMERA  Detector and optics of the in-house star
%   tracker. camera(fov) gives the baseline values (until the unit is designed):
%   1024 x 1024 pixels, Gaussian PSF (1.2 px), a magnitude-6 star gives 3000 e-,
%   background 50 e- with 8 e- read noise, and the onboard chain's settings.
%   camera(fov, v) takes every value from v, the part's nominal values as
%   asils.product.load reads them for the 'image' model (detector_px,
%   psf_sigma_px, flux_mag6_e, background_e, read_noise_e, centroid_k_sigma,
%   max_spots, id_tol_rad, id_mag_tol, fit_tol_rad). The focal length follows
%   from the field of view.
    if nargin < 1, fov_half_rad = 0.17; end
    if nargin < 2
        v = struct('detector_px', 1024, 'psf_sigma_px', 1.2, 'flux_mag6_e', 3000, 'background_e', 50, ...
                   'read_noise_e', 8, 'centroid_k_sigma', 5, 'max_spots', 20, 'id_tol_rad', 2e-4, ...
                   'id_mag_tol', 0.25, 'fit_tol_rad', 1e-4);
    end
    cam = struct('n', v.detector_px, 'fov', fov_half_rad, 'psf_px', v.psf_sigma_px, 'flux0', v.flux_mag6_e, ...
                 'bg', v.background_e, 'read_noise', v.read_noise_e, 'k_sigma', v.centroid_k_sigma, ...
                 'max_spots', v.max_spots, 'id_tol_rad', v.id_tol_rad, 'mag_tol', v.id_mag_tol, ...
                 'fit_tol_rad', v.fit_tol_rad, 'noise', true);
    cam.f = (cam.n/2)/tan(fov_half_rad);             % focal length [px]
    cam.c = (cam.n + 1)/2;                           % principal point
end
