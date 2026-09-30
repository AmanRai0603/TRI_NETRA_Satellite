function atm = nrlmsise(geo, sw)
%ATMOS.NRLMSISE  Adapter to MATLAB Aerospace Toolbox atmosnrlmsise00.
%   Requires the Aerospace Toolbox (function atmosnrlmsise00).  Mirrors the call
%   used in your gm_comparison drag block.  Returns multi-species number
%   densities so drag.force can run the full multi-species GSI.
    v=geo.utc; yr=v(1); doy=geo.doy;
    utsec = v(4)*3600+v(5)*60+v(6);
    % NO SILENT DEFAULTS. These used to be getf(sw,'F107',150) etc -- so calling
    % this with sw=struct() quietly gave you solar-max-ish air with no warning.
    % atmos.spaceweather now guarantees finite indices or errors, so demand them.
    F107  = req(sw,'F107');    % PREVIOUS day's flux, per the NRLMSISE-00 spec
    F107a = req(sw,'F107a');   % 81-day centred mean
    Ap    = req(sw,'ap');      % daily

    % ---------------------------------------------------------------------
    % STORM RESPONSE. This was: aph = Ap*ones(1,7); flags = ones(1,23);
    % Two separate faults, both of which erase storms:
    %   1. aph filled with the DAILY Ap in all 7 slots, discarding the 57 h
    %      history the array exists to carry (sw.ap3 was computed and dropped).
    %   2. flags(9) = 1 tells atmosnrlmsise00 to use the DAILY Ap and IGNORE aph
    %      entirely. flags(9) = -1 is what switches the aph array on. So even a
    %      correctly-filled aph would have been thrown away.
    % At low LEO the storm response is not a refinement: density can double in hours.
    % >> THIS CHANGES RESULTS vs every previous run. <<
    % ---------------------------------------------------------------------
    % PROVEN DEFAULT. run_comparison_study.m does  Aph = D.ap .* ones(n,7)  and
    % passes the DEFAULT flags -- i.e. flat ap, flags(9)=+1, no storm history. The
    % GOCE-validated numbers were produced that way, so that is what happens unless
    % you ask for the spec form via atmos.spaceweather's opts.aph_mode='history'.
    %
    % flags(9) is the switch that MATTERS: with +1 atmosnrlmsise00 uses the daily Ap
    % and IGNORES aph completely, so a correctly-built history would be silently
    % discarded. Only flip it when a real history was actually supplied.
    aph = getf(sw,'aph', Ap*ones(1,7));
    if any(~isfinite(aph)), aph = Ap*ones(1,7); end
    flags = ones(1,23);
    if strncmpi(getf(sw,'aph_source','flat'), 'history', 7)
        flags(9) = -1;         % -1 => honour the aph array instead of the daily Ap
    end
    [Tz,rho] = atmosnrlmsise00(geo.alt_km*1000, geo.lat_deg, geo.lon_deg, ...
                               yr, doy, utsec, geo.lst_h, F107a, F107, aph, flags);
    % rho columns (NRLMSISE-00): 1 He,2 O,3 N2,4 O2,5 Ar,6 total mass,7 H,8 N,9 anomO
    atm.T = Tz(2);                       % neutral temperature
    atm.n.He=rho(1); atm.n.O=rho(2); atm.n.N2=rho(3); atm.n.O2=rho(4);
    atm.n.Ar=rho(5); atm.n.H=rho(7);  atm.n.N=rho(8);
    atm.rho = rho(6); atm.nO = rho(2);
end
function val=getf(s,f,d), if isfield(s,f)&&~isempty(s.(f)), val=s.(f); else, val=d; end, end
function v=req(s,f)
    if ~isfield(s,f) || isempty(s.(f)) || any(~isfinite(s.(f)))
        error('atmos:nrlmsise:index', ['NRLMSISE needs a finite "%s". Pass real indices ' ...
            'via atmos.spaceweather (table or manual) -- a silent stand-in here is how ' ...
            'you end up reporting solar-max density on a quiet day.'], f);
    end
    v = s.(f);
end
