function [week, sow] = gpsWeekSow(jd_gps)
%GPSWEEKSOW  GPS-time JD -> [GPS week, seconds-of-week]. Epoch 1980-01-06.
    dt = jd_gps - 2444244.5; week = floor(dt/7); sow = (dt - week*7)*86400;
end
