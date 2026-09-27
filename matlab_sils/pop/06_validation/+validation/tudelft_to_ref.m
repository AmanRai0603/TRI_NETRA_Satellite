function D = tudelft_to_ref(T)
%VALIDATION.TUDELFT_TO_REF  TU Delft density timetable -> the REF.TUD struct.
%   D = validation.tudelft_to_ref(T)
%
%   data.tudelft_density returns a MATLAB timetable (DateTime, Density_kgm3,
%   Altitude_m, Latitude_deg, Longitude_deg, ...). od_metrics wants plain arrays
%   on MJD, like every other REF product, so the metric code does not have to know
%   which group shipped which container. One adapter, one place.
%
%   TU Delft ships the geodetic track WITH the density, so the model is evaluated
%   at TU Delft's own points -- our position error never enters metric (5). That is
%   the opposite of the ITSG r1.0 neutralDensity_ACC product, which is MJD+rho only
%   and therefore has to borrow our state.
    D = struct();
    dt = T.DateTime;
    D.mjd = datenum(dt) - 678942;
    D.rho = T.Density_kgm3(:);
    D.alt = T.Altitude_m(:);
    D.lat = T.Latitude_deg(:);
    D.lon = T.Longitude_deg(:);
    % LST is not shipped: derive it the same way compare_density.m does, so the two
    % agree by construction rather than by coincidence.
    v = datevec(dt);
    utc_h = v(:,4) + v(:,5)/60 + v(:,6)/3600;
    D.lst = mod(utc_h + D.lon/15, 24);
end
