function S = centroid(img, cam)
%ASILS.COMP.STAR_TRACKER.CENTROID  Pixel frame -> star spots (BASELINE method,
%   in-house algorithm to come): robust background (median, MAD), threshold at
%   k_sigma, brightest pixels first with a 4-pixel exclusion, centre of gravity
%   of the background-subtracted 5 x 5 window.
%   S: [x; y; flux] per spot (flux in e-), brightest first, at most max_spots.
    bg = median(img(:)); sg = 1.4826*median(abs(img(:) - bg));
    thr = bg + cam.k_sigma*sg;
    idx = find(img > thr);
    [~, o] = sort(img(idx), 'descend'); idx = idx(o);
    [ry, rx] = ind2sub(size(img), idx);
    S = zeros(3, 0); n = size(img, 1);
    for k = 1:numel(idx)
        x = rx(k); y = ry(k);
        if x < 3 || y < 3 || x > n - 2 || y > n - 2, continue, end
        if ~isempty(S) && any(abs(S(1,:) - x) < 4 & abs(S(2,:) - y) < 4), continue, end
        W = img(y-2:y+2, x-2:x+2) - bg; W(W < 0) = 0;
        [gx, gy] = meshgrid(-2:2, -2:2); tot = sum(W(:));
        S(:, end+1) = [x + sum(gx(:).*W(:))/tot; y + sum(gy(:).*W(:))/tot; tot]; %#ok<AGROW>
        if size(S, 2) >= cam.max_spots, break, end
    end
end
