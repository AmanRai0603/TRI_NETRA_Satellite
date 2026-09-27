function jd = utc2gps(jd_utc)
%UTC2GPS  UTC JD -> GPS-time JD.
    jd = timeconv.tai2gps(timeconv.utc2tai(jd_utc));
end
