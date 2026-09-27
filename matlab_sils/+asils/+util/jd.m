function jd = jd(utc)
%ASILS.UTIL.JD  Julian date of a UTC [Y M D h m s] vector (Vallado alg. 14).
    Y = utc(1); M = utc(2); D = utc(3);
    jd = 367*Y - floor(7*(Y + floor((M+9)/12))/4) + floor(275*M/9) + D + 1721013.5 ...
         + ((utc(6)/60 + utc(5))/60 + utc(4))/24;
end
