function jd = op_T_(utc)
%VALIDATION.OP_T_  TDB Julian date at a UTC vector. A file, not a script local.
    T = timeconv.convertUTC(utc(1),utc(2),utc(3),utc(4),utc(5),utc(6),0);
    jd = T.tdb_jd;
end
