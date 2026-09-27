function sw = research_drivers(DRV, utc)
%ATMOS.RESEARCH_DRIVERS  Sample the DTM2020-research drivers at ONE epoch.
%   sw = atmos.research_drivers(DRV, utc)
%
%   DRV is the bundle from data.drivers('dtm2020_research',...): .f30TT .ap60TT
%   .f30_is_derived. This picks the values in force AT utc and hands them on as the
%   sw struct atmos.dtm2020_research expects.
%
%   SAMPLING RULE -- 'previous', i.e. the most recent sample at or before the
%   epoch. Same rule as align_drivers_to_track in the GOCE study: daily F30 by
%   most-recent-day, hourly ap60 by most-recent-sample. NOT 'linear': these indices
%   are step-wise measurements with a validity period, not a continuous signal, and
%   interpolating them invents values that were never observed and smears storm
%   onsets across the hour before they happened.
    if isempty(DRV) || ~isstruct(DRV) || ~isfield(DRV,'f30TT')
        error('atmos:research_drivers:noDrivers', ...
          ['DTM2020 research has no F30/ap60 drivers. They are resolved by ' ...
           'op.buildWorld via data.drivers(''dtm2020_research'',..) -- reaching here ' ...
           'means the pipeline was bypassed. This model does NOT run on F10.7/Kp.']);
    end
    tq = datenum(utc(1),utc(2),utc(3),utc(4),utc(5),utc(6));
    sw = struct();
    sw.F30            = prevSample(DRV.f30TT.Time,  DRV.f30TT.F30,     tq, 'F30');
    sw.F30_bar        = prevSample(DRV.f30TT.Time,  DRV.f30TT.F30_bar, tq, 'F30_bar');
    sw.ap60           = prevSample(DRV.ap60TT.Time, DRV.ap60TT.ap60,   tq, 'ap60');
    sw.f30_is_derived = isfield(DRV,'f30_is_derived') && DRV.f30_is_derived;
end

function v = prevSample(T, V, tq, name)
%PREVSAMPLE  Most recent finite sample at or before tq. Errors rather than
%   extrapolating: a driver outside its fetched window is a window bug, and
%   silently reusing the nearest edge value is how a storm goes missing.
    td = datenum(T);
    k  = find(td <= tq & isfinite(V(:)), 1, 'last');
    if isempty(k)
        error('atmos:research_drivers:coverage', ...
          ['no %s sample at or before %s (table starts %s). Widen the window: ' ...
           'cfg.spaceweather.padDays.'], name, datestr(tq), datestr(td(1)));
    end
    v = V(k);
end
