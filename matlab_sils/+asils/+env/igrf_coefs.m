function coefs = igrf_coefs()
%ASILS.ENV.IGRF_COEFS  IGRF-13 Gauss coefficients (nT), degree 13, every 5-year epoch 1900-2025.
%   Read from data/igrf13.json, which tools/gen_fsw_params.py writes from the IAGA coefficient file
%   data/igrf13coeffs.txt (2025.0 = 2020.0 + 5 x the secular variation): the same numbers the
%   engine and both flight-software builds carry. Dates after 2025 are extrapolated by
%   asils.env.igrf_gh. (This table was once ported from the Standard Code's env.igrfCoefs, whose
%   values were rounded and partly IGRF-12; they differed from IGRF-13 by up to 75 nT.)
    persistent C
    if isempty(C)
        d = asils.util.readjson(fullfile(asils.util.root(), 'data', 'igrf13.json'));
        E = d.epochs; if isstruct(E), E = num2cell(E); end
        C = repmat(struct('year', 0, 'gh', zeros(1, 195)), 1, numel(E));
        for i = 1:numel(E)
            C(i).year = E{i}.year;
            C(i).gh = reshape(E{i}.gh, 1, []);
        end
    end
    coefs = C;
end
