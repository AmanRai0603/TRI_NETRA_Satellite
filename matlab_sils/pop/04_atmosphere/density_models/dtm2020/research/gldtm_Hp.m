function [da, gdel] = gldtm_Hp(f, fbar, akp, day, a, plg, hloc, xlon, ff0)
%GLDTM_HP  Evaluate the DTM2020 g(L) basis-function expansion
%
% Input  : solar/geomagnetic drivers, day, coefficient vector, Legendre values, local hour,
%          longitude, ff0 flag
% Process: accumulates latitude/annual/diurnal/semidiurnal/terdiurnal/longitudinal and ap60-driven
%          geomagnetic terms
% Output : da (partial sums) and gdel = g(L) used in exp() by dtm5
%
%
% Computes the full variability function gdel = g(L) for one constituent.
%
% INPUTS
%   f, fbar  - [2x1] solar flux and mean flux
%   akp      - [8x1] Kp index array (already converted from ap60)
%   day      - day of year
%   a        - [96x1] coefficient vector for this constituent
%   plg      - struct of Legendre polynomial values (from dtm5)
%   hloc     - struct of local time trig values
%   xlon     - longitude [rad]
%   ff0      - flag: 1 for temperature constituents, 0 for density
%
% OUTPUTS
%   da       - [96x1] basis-function values
%   gdel     - scalar result of g(L)

da = zeros(96, 1);

% ---- Legendre terms ----------------------------------------------------
da(2)  = plg.p20;
da(3)  = plg.p40;
da(74) = plg.p10;
da(77) = plg.p30;
da(78) = plg.p50;
da(79) = plg.p60;

% ---- Solar flux terms --------------------------------------------------
fmfb1  = f(1) - fbar(1);
fmfb2  = f(2) - fbar(2);
fbm150 = fbar(1) - 150.0;

da(4)  = fmfb1;
da(6)  = fbm150;
da(5)  = da(4)^2;
da(69) = da(6)^2;
da(82) = da(4) * plg.p10;
da(83) = da(4) * plg.p20;
da(84) = da(4) * plg.p30;
da(85) = da(6) * plg.p20;
da(86) = da(6) * plg.p30;
da(87) = da(6) * plg.p40;

% ---- Kp saturation during strong storms --------------------------------
akp_mod = akp;
if akp(1) >= 9.0 && akp(3) > 7.5,  akp_mod(1) = 9.0 + (akp(1)-9.0)/5.0; end
if akp(4) >= 9.0 && akp(3) > 7.5,  akp_mod(4) = 9.0 + (akp(4)-9.0)/5.0; end
if akp(5) >= 9.0 && akp(3) > 7.5,  akp_mod(5) = 9.0 + (akp(5)-9.0)/5.0; end
if akp(6) >= 9.0 && akp(3) > 7.5,  akp_mod(6) = 9.0 + (akp(6)-9.0)/5.0; end
if akp(7) >= 9.0 && akp(3) > 7.5,  akp_mod(7) = 9.0 + (akp(7)-9.0)/5.0; end
if akp(8) >= 9.0 && akp(3) > 7.5,  akp_mod(8) = 9.0 + (akp(8)-9.0)/5.0; end

dkp  = akp_mod(1);
dkpm = akp_mod(3);

% ---- Kp basis functions ------------------------------------------------
da(7)  = dkp;
da(8)  = plg.p20mg * dkp;
da(60) = dkp^2;
da(61) = plg.p20mg * da(60);
da(62) = plg.p30mg * dkp;
da(63) = plg.p10mg * dkp;
da(67) = plg.p60mg * dkp;
da(68) = plg.p40mg * dkp;

da(64) = dkpm;
da(65) = plg.p20mg * dkpm;
da(66) = dkpm^2;
da(73) = plg.p20mg * da(66);

% ---- Flux-dependent storm-time high-order Kp terms ---------------------
flux = max(f(1), fbar(1));
iflux = floor(flux);

if iflux >= 200
    scale75 = 0.333;  scale4  = 0.1;
elseif iflux >= 190
    scale75 = 0.55;   scale4  = 0.15;
elseif iflux >= 180
    scale75 = 0.733;  scale4  = 0.2;
elseif iflux >= 160
    scale75 = 1.0;    scale4  = 0.4;
elseif iflux >= 140
    scale75 = 1.0;    scale4  = 0.8;
else
    scale75 = 1.0;    scale4  = 1.0;
end

da(75) = scale75 * da(60)^2;
da(71) = scale4  * akp_mod(5)^4;
da(72) = scale4  * akp_mod(6)^4;
da(76) = scale4  * akp_mod(7)^4;
da(79) = scale4  * akp_mod(8)^4;   % Note: overwrites p60 term above (matches Fortran)

da(70) = akp_mod(2);

% ---- Static part of g(L): flux + latitude + Kp ------------------------
f0 = a(4)*da(4)  + a(5)*da(5)  + a(6)*da(6)  + a(69)*da(69) + ...
     a(82)*da(82) + a(83)*da(83) + a(84)*da(84) + a(85)*da(85) + ...
     a(86)*da(86) + a(87)*da(87);

f1f = 1.0 + f0*ff0;

f0 = f0 + a(2)*da(2)  + a(3)*da(3)  + a(74)*da(74) + a(77)*da(77) + ...
     a(7)*da(7)   + a(8)*da(8)   + a(60)*da(60) + a(61)*da(61) + ...
     a(68)*da(68) + a(64)*da(64) + a(65)*da(65) + a(66)*da(66) + ...
     a(72)*da(72) + a(73)*da(73) + a(75)*da(75) + a(76)*da(76) + ...
     a(78)*da(78) + a(79)*da(79) + a(70)*da(70) + a(71)*da(71) + ...
     a(62)*da(62) + a(63)*da(63) + a(67)*da(67);

% ---- Seasonal annual and semi-annual terms -----------------------------
rot  = 0.017214206;   % 2pi/365.25 [rad/day]
rot2 = 0.034428412;   % 4pi/365.25

da(9)  = cos(rot  * (day - a(11)));
da(10) = plg.p20  * da(9);
da(12) = cos(rot2 * (day - a(14)));
da(13) = plg.p20  * da(12);

coste  = cos(rot  * (day - a(18)));
da(15) = plg.p10  * coste;
da(16) = plg.p30  * coste;
da(17) = da(6)    * da(15);

cos2te = cos(rot2 * (day - a(20)));
da(19) = plg.p10  * cos2te;
da(39) = plg.p30  * cos2te;
da(59) = da(6)    * da(19);

% ---- Diurnal (S1) terms ------------------------------------------------
ch  = hloc.ch;   sh  = hloc.sh;
c2h = hloc.c2h;  s2h = hloc.s2h;
c3h = hloc.c3h;  s3h = hloc.s3h;

da(21) = plg.p11 * ch;
da(22) = plg.p31 * ch;
da(23) = da(6)   * da(21);
da(24) = da(21)  * coste;
da(25) = plg.p21 * ch * coste;
da(26) = plg.p11 * sh;
da(27) = plg.p31 * sh;
da(28) = da(6)   * da(26);
da(29) = da(26)  * coste;
da(30) = plg.p21 * sh * coste;
da(94) = plg.p51 * ch;
da(95) = plg.p51 * sh;

% ---- Semi-diurnal (S2) terms -------------------------------------------
da(31) = plg.p22 * c2h;
da(37) = plg.p42 * c2h;
da(32) = plg.p32 * c2h * coste;
da(33) = plg.p22 * s2h;
da(38) = plg.p42 * s2h;
da(34) = plg.p32 * s2h * coste;
da(88) = plg.p32 * c2h;
da(89) = plg.p32 * s2h;
da(90) = da(6)   * da(31);
da(91) = da(6)   * da(33);
da(92) = plg.p62 * c2h;
da(93) = plg.p62 * s2h;

% ---- Ter-diurnal (S3) terms --------------------------------------------
da(35) = plg.p33 * c3h;
da(36) = plg.p33 * s3h;

% ---- Periodic variability part of g(L) --------------------------------
fp = a(9)*da(9)   + a(10)*da(10) + a(12)*da(12) + a(13)*da(13) + ...
     a(15)*da(15) + a(16)*da(16) + a(17)*da(17) + a(19)*da(19) + ...
     a(21)*da(21) + a(22)*da(22) + a(23)*da(23) + a(24)*da(24) + ...
     a(25)*da(25) + a(26)*da(26) + a(27)*da(27) + a(28)*da(28) + ...
     a(29)*da(29) + a(30)*da(30) + a(31)*da(31) + a(32)*da(32) + ...
     a(33)*da(33) + a(34)*da(34) + a(35)*da(35) + a(36)*da(36) + ...
     a(37)*da(37) + a(38)*da(38) + a(39)*da(39) + a(59)*da(59) + ...
     a(88)*da(88) + a(89)*da(89) + a(90)*da(90) + a(91)*da(91) + ...
     a(92)*da(92) + a(93)*da(93) + a(94)*da(94) + a(95)*da(95);

% ---- Geomagnetic seasonal-diurnal coupling terms -----------------------
da(40) = plg.p10mg * cos2te * dkpm;
da(41) = plg.p10mg * coste  * dkpm;
da(42) = plg.p10mg * cos2te * dkp;
da(43) = plg.p11mg * ch     * dkp;
da(44) = plg.p31mg * ch     * dkp;
da(45) = plg.p22mg * c2h    * dkp;
da(46) = plg.p11mg * sh     * dkp;
da(47) = plg.p31mg * sh     * dkp;
da(48) = plg.p22mg * s2h    * dkp;

fp = fp + a(40)*da(40) + a(41)*da(41) + a(42)*da(42) + a(43)*da(43) + ...
          a(44)*da(44) + a(45)*da(45) + a(46)*da(46) + a(47)*da(47) + ...
          a(48)*da(48);

% ---- Non-migrating (longitude-dependent) tides -------------------------
clfl = cos(xlon);
slfl = sin(xlon);
da(49) = plg.p11 * clfl;
da(50) = plg.p21 * clfl;
da(51) = plg.p31 * clfl;
da(52) = plg.p41 * clfl;
da(53) = plg.p51 * clfl;
da(54) = plg.p11 * slfl;
da(55) = plg.p21 * slfl;
da(56) = plg.p31 * slfl;
da(57) = plg.p41 * slfl;
da(58) = plg.p51 * slfl;

fp = fp + a(49)*da(49) + a(50)*da(50) + a(51)*da(51) + a(52)*da(52) + ...
          a(53)*da(53) + a(54)*da(54) + a(55)*da(55) + a(56)*da(56) + ...
          a(57)*da(57) + a(58)*da(58);

% ---- Final result ------------------------------------------------------
gdel = f0 + fp * f1f;

end  % gldtm_Hp
