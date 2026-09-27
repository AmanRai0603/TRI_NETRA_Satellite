function [dxp, dyp, dut1] = tidal_eop(mjd)
%TIDAL_EOP  Full IERS sub-daily EOP correction = ocean tides (ORTHO_EOP) +
%  libration in polar motion (PMSDNUT2) + libration in UT1 (UTLIBR).
%  Returns dxp,dyp in MICROARCSECONDS and dut1 in MICROSECONDS. Add to the
%  interpolated daily EOP. Matches the IERS Conventions (2010) sub-daily model
%  (the same set Orekit applies with simpleEOP=false).
[ox, oy, ou] = tidal_eop_ocean(mjd);       % ocean tides (dominant)
[px, py]     = tidal_pm_libration(mjd);    % libration in polar motion
[uu, ~]      = tidal_ut1_libration(mjd);   % libration in UT1
dxp = ox + px;  dyp = oy + py;  dut1 = ou + uu;
end
