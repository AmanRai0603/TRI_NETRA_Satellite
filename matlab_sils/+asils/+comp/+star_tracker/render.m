function [img, truth] = render(R_eci2head, cat, cam)
%ASILS.COMP.STAR_TRACKER.RENDER  MODEL SIDE of the star-tracker chain: the
%   frame the detector would read at the head attitude R_eci2head (DCM): every
%   catalogue star in the field projected through a pinhole (head z = boresight),
%   spread by a Gaussian PSF, on a background with shot and read noise.
%   truth: [x; y; id] of the rendered stars (for tests only).
    n = cam.n; img = cam.bg*ones(n);
    v = R_eci2head*cat.r;
    in = find(v(3,:) > cos(cam.fov*sqrt(2)));
    truth = zeros(3, 0);
    [gx, gy] = meshgrid(-4:4, -4:4);
    for k = in
        x = cam.f*v(1,k)/v(3,k) + cam.c; y = cam.f*v(2,k)/v(3,k) + cam.c;
        if x < 6 || y < 6 || x > n - 5 || y > n - 5, continue, end
        fl = cam.flux0*10^(-0.4*(cat.mag(k) - 6));
        ix = round(x); iy = round(y);
        w = exp(-((ix + gx - x).^2 + (iy + gy - y).^2)/(2*cam.psf_px^2)); w = w/sum(w(:));
        img(iy-4:iy+4, ix-4:ix+4) = img(iy-4:iy+4, ix-4:ix+4) + fl*w;
        truth(:, end+1) = [x; y; k]; %#ok<AGROW>
    end
    img = img + sqrt(max(img, 0)).*randn(n) + cam.read_noise*randn(n);     % shot + read noise
end
