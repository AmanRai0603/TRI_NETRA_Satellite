function atm = dtm2020_research(geo, sw)
%ATMOS.DTM2020_RESEARCH  DTM2020 RESEARCH variant -- F30 + ap60 driven.
%
%   This is the model that was ported bit-for-bit against the CNES/SWAMI Fortran.
%   It is NOT the operational one (atmos.dtm2020), and it is NOT a drop-in for it:
%
%     operational (dtm3, DTM_2020_F107_Kp.dat) : F10.7, F10.7_bar, Kp
%     research    (dtm5, DTM_2020_F30_ap60.dat): F30,  F30_bar,   ap60
%
%   F30 is a different measurement on a different scale to F10.7 -- feeding one
%   into the other's slot is a quantity error, not a units nicety.
%
%   ---------------------------------------------------------------------------
%   THE RESCALING -- the part that is easy to get silently wrong
%   ---------------------------------------------------------------------------
%   dtm5 is driven by F30 RESCALED to the F10.7 scale (DTM2020 paper eq. 2, via
%   f30_to_f107scale). That regression carries a DRIFT TERM in decimal year, so it
%   is date-dependent and cannot be a constant.
%
%   BUT: if get_f30 could not reach CLS/LISIRD it falls back to a pseudo-F30
%   DERIVED from F10.7. Rescaling that would inflate it by the drift term twice
%   over. SWAMI's own guidance is to feed F10.7 DIRECTLY in that case. So the
%   caller must tell us which one it got -- sw.f30_is_derived -- rather than us
%   guessing from the magnitude. Mirrors run_comparison_study.m in the GOCE study.
%
%   Expects from sw (assembled by forces.drag from the buildWorld-resolved DRV):
%     .F30 .F30_bar   native F30 [sfu]  (or F10.7 if .f30_is_derived)
%     .ap60           hourly ap60
%     .f30_is_derived logical
    F30    = req(sw,'F30');
    F30bar = req(sw,'F30_bar');
    ap60   = req(sw,'ap60');

    if getf(sw,'f30_is_derived',false)
        % Real F30 was unavailable: this is already on the F10.7 scale. Use as-is.
        F30in = F30;  F30bin = F30bar;
    else
        decYr = decimalYear(geo.utc);
        F30in  = f30_to_f107scale(F30,    decYr);
        F30bin = f30_to_f107scale(F30bar, decYr);
    end

    persistent ST
    if isempty(ST), ST = DTM2020_coeffs_init(); end     % coefficients: load ONCE
    out = dtm2020_density(geo.alt_km, geo.lat_deg, geo.lon_deg, geo.lst_h, ...
                          geo.doy, F30in, F30bin, ap60, ST);
    atm.rho = out.rho_kgm3; atm.T = out.T_K;
    if isfield(out,'n')
        atm.n = out.n;
    else
        atm.Mmol = 16; K = de440.constants(); atm.nO = out.rho_kgm3*K.N_A*1000/16;
    end
end

function y = decimalYear(utc)
%DECIMALYEAR  Decimal year for the F30->F10.7 drift term. Leap-year aware.
    y0 = utc(1);
    d0 = datenum(y0,1,1); d1 = datenum(y0+1,1,1);
    y  = y0 + (datenum(utc(1),utc(2),utc(3),utc(4),utc(5),utc(6)) - d0) / (d1 - d0);
end

function val=getf(s,f,d), if isfield(s,f)&&~isempty(s.(f)), val=s.(f); else, val=d; end, end
function v=req(s,f)
    if ~isfield(s,f) || isempty(s.(f)) || any(~isfinite(s.(f)))
        error('atmos:dtm2020_research:driver', ...
          ['DTM2020 research needs a finite "%s". Its drivers are F30/F30_bar/ap60 -- ' ...
           'NOT F10.7/Kp. They are resolved by op.buildWorld via data.drivers(''dtm2020_research'',..).'], f);
    end
    v = s.(f);
end
