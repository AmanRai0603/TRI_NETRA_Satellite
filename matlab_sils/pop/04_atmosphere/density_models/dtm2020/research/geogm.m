function [gmlat, gmlong] = geogm(xlat, xlong)
%GEOGM  Convert geographic to geomagnetic latitude and longitude
%
% Input  : geographic latitude, longitude [deg]
% Process: rotates about the dipole pole at 78.5 N, 291 E (IGRF ~1990)
% Output : geomagnetic latitude and longitude [deg]
%
%       Uses dipole approximation: magnetic pole at 78.5°N, 291°E (IGRF ~1990)
%
% INPUTS
%   xlat   - geographic latitude [degrees]
%   xlong  - geographic longitude [degrees]
%
% OUTPUTS
%   gmlat  - geomagnetic latitude [degrees]
%   gmlong - geomagnetic longitude [degrees]

rfac  = pi / 180.0;
platr = 78.5  * rfac;
plongr = 291.0 * rfac;

% Avoid exact singularity (matches Fortran source)
if xlong == 291.0,  xlong = 291.1;  end

spl = sin(platr);
cpl = cos(platr);
rlat  = xlat  * rfac;
rlong = xlong * rfac;

slm = spl*sin(rlat) + cpl*cos(rlat)*cos(plongr - rlong);
clm = sqrt(1.0 - slm^2);

phim1 = cos(rlat)*sin(rlong - plongr) / clm;
phim2 = (spl*slm - sin(rlat)) / (cpl*clm);

gmlat = asin(slm) / rfac;

if     phim1 >= 0 && phim2 >= 0
    gmlong = asin(phim1) / rfac;
elseif phim1 >= 0 && phim2 < 0
    gmlong = (pi - abs(asin(phim1))) / rfac;
elseif phim1 < 0  && phim2 < 0
    gmlong = (pi + abs(asin(phim1))) / rfac;
elseif phim1 < 0  && phim2 >= 0
    gmlong = (2.0*pi - abs(asin(phim1))) / rfac;
else
    error('GEOGM: angle error at xlat=%.2f xlong=%.2f', xlat, xlong);
end

end
