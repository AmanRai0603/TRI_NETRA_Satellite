function g = probe_geo(alt_km, DATE, lat_deg, lon_deg, lst_h)
%VALIDATION.PROBE_GEO  A geodetic probe point for atmosphere checks.
%   A FILE, not a script-local: MATLAB hoists a script's local functions and Octave
%   does NOT, so a helper at the bottom of a script is undefined when the script
%   calls it. That bug has bitten this codebase repeatedly -- most recently in the
%   script written to verify the drag chain.
    if nargin<3||isempty(lat_deg), lat_deg=0;  end
    if nargin<4||isempty(lon_deg), lon_deg=0;  end
    if nargin<5||isempty(lst_h),   lst_h=12;   end
    v = datevec(datenum(DATE));
    g = struct('alt_km',alt_km,'lat_deg',lat_deg,'lon_deg',lon_deg,'lst_h',lst_h, ...
               'doy',timeconv.doy(v(1),v(2),v(3)),'utc',[v(1) v(2) v(3) 0 0 0]);
end
