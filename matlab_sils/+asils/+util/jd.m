function jd = jd(utc)
%ASILS.UTIL.JD  Julian date of a UTC [Y M D h m s] vector (Vallado alg. 14): env's method env_calendar_time,
%   generated from the design (asils.models.caltime.jd_utc, tools/engine_build.py), the engine's own.
    jd = asils.models.caltime.jd_utc(utc);
end
