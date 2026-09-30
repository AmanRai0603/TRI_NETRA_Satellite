function atm = exponential(alt_km, varargin)
%ATMOS.EXPONENTIAL  Zero-dependency piecewise-exponential density model.
%   atm = atmos.exponential(alt_km)
%   Returns a struct ready for drag.force:
%     atm.rho   total mass density [kg/m^3]
%     atm.T     ambient temperature [K]  (coarse thermospheric estimate)
%     atm.Mmol  mean molar mass [kg/kmol] (16 = atomic-O dominated in low LEO)
%     atm.nO    atomic-oxygen number density [m^-3] (for DRIA/SESAM accommodation)
%
%   Uses the standard Vallado exponential-atmosphere table (base altitude,
%   base density, scale height) -- purely a function of altitude, no space
%   weather, no data files.  Good enough for first-order drag, force-impact
%   studies, and offline demos.  For real density use atmos.provider(...,'nrlmsise'|
%   'jb2008'|'dtm2020'), which call the GOCE-study models (needs their data on path).
%
%   NOTE: this ignores diurnal/latitudinal/solar variation, so absolute low LEO
%   density can be off by a factor of a few; it is a SHAPE/SANITY model, not a
%   space-weather model.
    h = alt_km;
    % Vallado Table (h0 [km], rho0 [kg/m^3], H [km])
    T = [   0  1.225      7.249
           25  3.899e-2   6.349
           30  1.774e-2   6.682
           40  3.972e-3   7.554
           50  1.057e-3   8.382
           60  3.206e-4   7.714
           70  8.770e-5   6.549
           80  1.905e-5   5.799
           90  3.396e-6   5.382
          100  5.297e-7   5.877
          110  9.661e-8   7.263
          120  2.438e-8   9.473
          130  8.484e-9  12.636
          140  3.845e-9  16.149
          150  2.070e-9  22.523
          180  5.464e-10 29.740
          200  2.789e-10 37.105
          250  7.248e-11 45.546
          300  2.418e-11 53.628
          350  9.518e-12 53.298
          400  3.725e-12 58.515
          450  1.585e-12 60.828
          500  6.967e-13 63.822
          600  1.454e-13 71.835
          700  3.614e-14 88.667
          800  1.170e-14 124.64
          900  5.245e-15 181.05
         1000  3.019e-15 268.00 ];
    if h < 0, h = 0; end
    idx = find(T(:,1) <= h, 1, 'last'); if isempty(idx), idx=1; end
    h0=T(idx,1); rho0=T(idx,2); H=T(idx,3);
    atm.rho  = rho0 * exp(-(h-h0)/H);

    % Coarse thermospheric temperature (rises from ~186K at 90km to ~1000K plateau)
    atm.T = 1000 - (1000-186)*exp(-(max(h,90)-90)/70);
    atm.Mmol = meanMolarMass(h);
    K = de440.constants(); NA = K.N_A * 1000;  % per kmol (Mmol is kg/kmol)
    atm.nO   = atm.rho * NA / atm.Mmol;       % treat as O-dominated for GSI
end

function M = meanMolarMass(h)
% crude mean molar mass transition N2/O2 (~29) -> atomic O (~16) -> H (~1)
    if     h < 150, M = 29 - (29-22)*(h-100)/50; M=max(M,22);
    elseif h < 500, M = 22 - (22-16)*(h-150)/350;
    else            M = 16 - (16-4 )*min((h-500)/500,1);
    end
    M = max(M, 4);
end
