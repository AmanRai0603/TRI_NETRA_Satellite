function u = itsg_urls(satDir, pdir, stem, dateStr)
%DATA.ITSG_URLS  Public view of the candidate URLs for a product/date.
%   u = data.itsg_urls('CHAMP','neutralDensity_1.0','neutralDensity','2003-01-01')
%
%   Exposed for the same reason as itsg_productinfo: the URL construction was wrong
%   for versioned directories for several rounds and no test could see it, because
%   the only way to exercise it was to make a real request and watch it 404.
    u = data.itsg('__urls__', dateStr, {satDir, pdir, stem});
end
