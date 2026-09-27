function gh = igrf_gh(decyear, coefs)
%ASILS.ENV.IGRF_GH  Gauss coefficients at a decimal year.
%   Ported from Standard Code env.igrfCoefsAt (linear interpolation between
%   5-year epochs). CHANGE: after the last epoch (2025) the table carries no
%   secular variation, so the coefficients are extrapolated with the 2020->2025
%   rate (at most 5 years); the Standard Code refused any date after 2025.
    if nargin < 2 || isempty(coefs), coefs = asils.env.igrf_coefs(); end
    yrs = [coefs.year];
    assert(decyear >= yrs(1) && decyear <= yrs(end) + 5, 'asils:igrf:range', ...
        'IGRF valid %d..%d, got %.2f', yrs(1), yrs(end)+5, decyear);
    i = find(yrs <= decyear, 1, 'last');
    if i == numel(yrs), i = i - 1; end      % extrapolate from the last interval
    g0 = coefs(i).gh; g1 = coefs(i+1).gh;
    gh = g0 + (g1 - g0) * (decyear - yrs(i)) / (yrs(i+1) - yrs(i));
end
