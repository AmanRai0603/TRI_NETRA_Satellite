function utc = addsec(utc0, dt)
%ASILS.UTIL.ADDSEC  UTC [Y M D h m s] plus dt seconds (no leap-second insertion).
    jd = asils.util.jd(utc0) + dt/86400;
    utc = asils.util.jd2utc(jd);
end
