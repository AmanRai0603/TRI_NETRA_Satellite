function T = convertUTC(Y, M, D, h, mi, s, dUT1)
%CONVERTUTC  One UTC calendar instant -> ALL time scales & representations.
%   T = timeconv.convertUTC(Y,M,D,h,mi,s[,dUT1])
%
%   This is the single entry point for the whole application: work in UTC,
%   call this once, then read whatever scale each module needs from T.
%
%   dUT1 = UT1-UTC (seconds), from your EOP feed (default 0). Only UT1/GMST
%   depend on it; everything else is exact.
%
%   Fields:
%     .utc_jd .tai_jd .tt_jd .tdb_jd .ut1_jd .gps_jd   Julian dates per scale
%     .utc_mjd .tt_mjd .tdb_mjd                        Modified JD
%     .leap        TAI-UTC (s)          .dUT1  UT1-UTC (s)
%     .T_tt        Julian centuries TT since J2000  (precession/nutation)
%     .j2000_tt_sec seconds past J2000 TT            (propagator time base)
%     .gps_week .gps_sow                              GPS week / sec-of-week
%     .gmst_rad    Greenwich Mean Sidereal Time (rad)
%     .doy         day of year
%
%   Feed T.tdb_jd to de440 / ephemInputs.  Feed T.ut1_jd / T.T_tt to your
%   ECI<->ECEF chain.  Feed T.gps_* to GNSS measurement models.
    if nargin < 7 || isempty(dUT1), dUT1 = 0; end
    jdU = timeconv.cal2jd(Y, M, D, h, mi, s);

    T.utc_jd = jdU;
    T.tai_jd = timeconv.utc2tai(jdU);
    T.tt_jd  = timeconv.utc2tt(jdU);
    T.tdb_jd = timeconv.utc2tdb(jdU);
    T.ut1_jd = timeconv.utc2ut1(jdU, dUT1);
    T.gps_jd = timeconv.utc2gps(jdU);

    T.utc_mjd = timeconv.jd2mjd(jdU);
    T.tt_mjd  = timeconv.jd2mjd(T.tt_jd);
    T.tdb_mjd = timeconv.jd2mjd(T.tdb_jd);

    T.leap = timeconv.taiMinusUTC(jdU);
    T.dUT1 = dUT1;
    T.T_tt = timeconv.tt2jc(T.tt_jd);
    T.j2000_tt_sec = timeconv.j2000sec(T.tt_jd);
    [T.gps_week, T.gps_sow] = timeconv.gpsWeekSow(T.gps_jd);
    T.gmst_rad = timeconv.gmst82(T.ut1_jd);
    T.doy = timeconv.doy(Y, M, D);
end
