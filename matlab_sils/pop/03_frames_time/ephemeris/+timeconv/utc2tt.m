function jd = utc2tt(jd_utc)
%UTC2TT  UTC JD -> TT JD.
    jd = timeconv.tai2tt(timeconv.utc2tai(jd_utc));
end
