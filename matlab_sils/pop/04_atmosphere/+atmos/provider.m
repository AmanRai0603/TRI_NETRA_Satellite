function atm = provider(model, geo, sw)
%ATMOS.PROVIDER  Return an atmosphere struct (T + species/density) for drag.
%   atm = atmos.provider(model, geo, sw)
%
%   INPUTS
%     model : 'exponential' | 'nrlmsise' | 'jb2008' | 'dtm2020' | 'dtm2020_research'
%             NOTE dtm2020 is the OPERATIONAL variant (F10.7+Kp, dtm3).
%             dtm2020_research is the F30+ap60 variant (dtm5) -- a different model
%             with different drivers, not a refinement of the same one.
%     geo   : struct with .alt_km .lat_deg .lon_deg .lst_h .doy .utc([Y..S])
%     sw    : space-weather struct from atmos.spaceweather(...) carrying the
%             indices the chosen model needs (.F107 .F107a .Kp .ap / .jb_idx).
%
%   OUTPUT  atm : struct consumed by drag.force (fields .T and either .rho/.Mmol/
%                 .nO or .n.<species>).
%
%   NO SILENT FALLBACK.  Each model is called directly; if its code or data
%   dependencies are missing, the underlying error propagates unchanged so you
%   know exactly what to install/fetch.  'exponential' is a *deliberate* choice
%   (altitude-only, no space weather), not an automatic substitute -- select it
%   explicitly if that is what you want.  See docs/ATMOSPHERE.md.
    if nargin<3, sw=struct(); end
    switch lower(model)
        case 'exponential'
            atm = atmos.exponential(geo.alt_km);
        case 'nrlmsise'
            atm = atmos.nrlmsise(geo, sw);
        case 'jb2008'
            atm = atmos.jb2008(geo, sw);
        case 'dtm2020'
            atm = atmos.dtm2020(geo, sw);              % operational: F10.7 + Kp
        case {'dtm2020_research','dtm2020_res'}
            atm = atmos.dtm2020_research(geo, sw);     % research: F30 + ap60
        otherwise
            error('atmos:provider', ['unknown atmosphere model "%s". Known: exponential, ' ...
                'nrlmsise, dtm2020 (operational, F10.7+Kp), dtm2020_research (F30+ap60), jb2008.'], model);
    end
end
