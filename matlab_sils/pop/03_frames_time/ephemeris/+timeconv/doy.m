function n = doy(Y, M, D)
%DOY  Day of year (1..366).
    n = timeconv.cal2jd(Y, M, D) - timeconv.cal2jd(Y, 1, 1) + 1;
end
