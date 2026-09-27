function utc = addsec(utc0, dt)
%OP.ADDSEC  Add dt seconds to a [Y Mo D H Mi S] UTC vector (handles rollover).
    dn  = datenum(utc0(1),utc0(2),utc0(3),utc0(4),utc0(5),utc0(6)) + dt/86400;
    dv  = datevec(dn);
    utc = dv;
end
