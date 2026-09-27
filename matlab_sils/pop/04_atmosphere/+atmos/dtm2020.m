function atm = dtm2020(geo, sw)
%ATMOS.DTM2020  Adapter to the GOCE-study DTM2020 operational wrapper.
%   Requires GOCE_density_study/3_density_models/dtm2020/operational on the path
%   (coefficient .dat in reference_data/).  Returns species densities + temperature.
    % NO SILENT DEFAULTS. These were getf(sw,'F107',120) / getf(sw,'Kp',3): call this
    % with an sw that is missing an index and you silently got moderate-activity air
    % instead of an error. Exactly the fault already fixed in atmos.nrlmsise -- and
    % the Kp default here is what made the "DTM2020 returns NaN" hunt so confusing,
    % because atmos.spaceweather handed over a NaN Kp FIELD and getf only defaults on
    % missing/empty, never on NaN. atmos.spaceweather now guarantees finite indices
    % or errors, so demand them rather than inventing them.
    F107  = req(sw,'F107');    % F10.7 at t-24h, per this model's own header
    F107a = req(sw,'F107a');   % 81-day mean
    Kp    = req(sw,'Kp');      % scalar Kp (or [4x1] akp) -- the storm driver
    out = dtm2020_oper_density(geo.alt_km, geo.lat_deg, geo.lon_deg, geo.lst_h, ...
                               geo.doy, F107, F107a, Kp);
    atm.rho = out.rho_kgm3; atm.T = out.T_K;   % actual DTM2020 output fields
    % map DTM species (fields vary by version); fall back to mean if absent
    if isfield(out,'n')                        % species number densities [1/cm^3]
        % ---------------------------------------------------------------------
        % UNITS. dtm2020_oper_density's own header says:
        %     out.n.{H,He,O,N2,O2,N}  [1/cm^3]
        % and this line used to hand that straight on as atm.n. But atmos.nrlmsise
        % fills atm.n from the Aerospace Toolbox, which returns [1/m^3]. So the SAME
        % field carried two different unit systems depending on which model produced
        % it, and drag.force -- correctly SI (NA per kmol, species in kg/kmol) --
        % assumes m^-3.
        %
        % Consequence: every PANEL drag model (sentman/dria/cll/sesam) running on
        % DTM2020 was 1e6 too small. Measured at 350 km: cannonball 5.00e-07,
        % panel 4.15e-13. Not subtly wrong -- a millionth. The cannonball branch was
        % unaffected because it reads atm.rho, which was always correct, which is
        % exactly why this survived: the default path never touched atm.n.
        %
        % SI is the convention everywhere else in this toolbox, so convert here at
        % the source rather than patching each consumer.
        atm.n = struct();
        nf = fieldnames(out.n);
        for ii = 1:numel(nf)
            atm.n.(nf{ii}) = out.n.(nf{ii}) * 1e6;   % cm^-3 -> m^-3
        end
    else
        atm.Mmol=16; atm.nO=out.rho_kgm3*avog16();
    end
end
function val=getf(s,f,d), if isfield(s,f)&&~isempty(s.(f)), val=s.(f); else, val=d; end, end
function v=req(s,f)
    if ~isfield(s,f) || isempty(s.(f)) || any(~isfinite(s.(f)))
        error('atmos:dtm2020:index', ['DTM2020 (operational) needs a finite "%s". Its drivers ' ...
            'are F10.7 + Kp; pass real indices via atmos.spaceweather (table or manual). A ' ...
            'silent stand-in here is how you end up reporting the wrong air on a quiet day.'], f);
    end
    v = s.(f);
end
function y=avog16(), K=de440.constants(); y=K.N_A*1000/16; end
