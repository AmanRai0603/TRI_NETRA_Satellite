%% EXAMPLE_TIDAL  Use the IERS sub-daily / zonal EOP models standalone, and
%  reproduce their official IERS test cases. Put the 5 tidal_*.m files + this file
%  in the same folder and run.
clc;
DAS2R = pi/(180*3600);

%% -- combined sub-daily correction at any epoch (what build C adds) --
mjd = 60735.5;                                   % 2025-03-01 12:00 UTC
[dxp, dyp, dut1] = tidal_eop(mjd);               % uas, uas, us
fprintf('sub-daily @ MJD %.1f:  dxp=%.3f uas  dyp=%.3f uas  dUT1=%.3f us\n', mjd, dxp, dyp, dut1);
fprintf('   -> add to interpolated EOP as:  xp += %.3e rad,  UT1-UTC += %.3e s\n\n', dxp*1e-6*DAS2R, dut1*1e-6);

%% -- validate each model against its official IERS test case --
[ox,oy,ou] = tidal_eop_ocean(47100);             % ORTHO_EOP test (MJD 47100)
fprintf('ORTHO_EOP(47100): dx=%.10f  dy=%.10f  dUT1=%.10f\n', ox,oy,ou);
fprintf('   expected      : -162.8386373 117.7907526 -23.3909237   -> err %.1e uas\n', ...
        max([abs(ox+162.8386373279636530),abs(oy-117.7907525842668974),abs(ou+23.39092370609808214)]));

[px,py] = tidal_pm_libration(54335);             % PMSDNUT2 test
fprintf('PMSDNUT2(54335) : %.8f %.8f   expected 24.8314424 -14.0924069 -> err %.1e uas\n', ...
        px,py, max(abs(px-24.83144238273364834),abs(py+14.09240692041837661)));

[uu,ul] = tidal_ut1_libration(44239.1);          % UTLIBR test
fprintf('UTLIBR(44239.1) : %.9f %.8f   expected 2.4411438 -14.7897125 -> err %.1e us\n\n', ...
        uu,ul, max(abs(uu-2.441143834386761746),abs(ul+14.78971247349449492)));

%% -- zonal UT1 (RG_ZONT2) used for regularising UT1 before interpolation --
fprintf('RG_ZONT2 zonal UT1 @ MJD %.1f: %.4f ms\n', mjd, tidal_ut1_zonal(mjd)*1e3);
