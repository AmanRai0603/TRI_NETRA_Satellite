function [da, gdel] = gldtm(f, fbar, akp, day, a, plg, hloc, xlon, ff0)
%GLDTM  DTM2020 operational g(L) basis-function expansion (F10.7/Kp).
%
% Input  : solar/geomagnetic drivers, day, coefficient vector, Legendre values, local hour,
%          longitude, ff0 flag
% Process: same structure as gldtm_Hp but with the 4-element Kp driver, nonlinear dkp terms and
%          dipole-only geomagnetic harmonics
% Output : da and gdel = g(L)
%
%
% Counterpart of gldtm_Hp (research). Differences from the research version:
%   - 4-element akp (Kp) with a nonlinear dkp/dkpm dependence on akp(2),akp(4)
%   - geomagnetic terms use only p10mg, p20mg, p40mg (dipole approx)
%   - the dkp storm terms da(40..48) use GEOGRAPHIC Legendre (p10,p30,p50,...)
%   - no Kp>=9 saturation, no flux-dependent storm scaling
%
% INPUTS / OUTPUTS: same interface as gldtm_Hp.
%   ff0 = 1 for temperature constituents, 0 for density.

da = zeros(96, 1);

% ---- Legendre terms ----------------------------------------------------
da(2)  = plg.p20;
da(3)  = plg.p40;
da(74) = plg.p10;
da(77) = plg.p30;
da(78) = plg.p50;
da(79) = plg.p60;

% ---- solar flux terms --------------------------------------------------
fmfb1  = f(1) - fbar(1);
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

% ---- Kp (geomagnetic) handling -----------------------------------------
% Nonlinear dependence on akp via the implicit coefficients a(62),a(63),a(67).
% Note: akp(2) and akp(4) are 0 in normal operational use, which makes the
% nonlinear correction terms vanish; the structure is kept for faithfulness.
ikp  = 62;
ikpm = 67;
c2fi = 1.0 - plg.p10mg^2;

dkp  = akp(1) + (a(ikp) + c2fi*a(ikp+1)) * akp(2);
dakp = a(7) + a(8)*plg.p20mg + a(68)*plg.p40mg + ...
       2.0*dkp*(a(60) + a(61)*plg.p20mg + a(75)*2.0*dkp*dkp);
da(ikp)   = dakp * akp(2);
da(ikp+1) = da(ikp) * c2fi;

dkpm  = akp(3) + a(ikpm)*akp(4);
dakpm = a(64) + a(65)*plg.p20mg + a(72)*plg.p40mg + ...
        2.0*dkpm*(a(66) + a(73)*plg.p20mg + a(76)*2.0*dkpm*dkpm);
da(ikpm) = dakpm * akp(4);

da(7)  = dkp;
da(8)  = plg.p20mg * dkp;
da(68) = plg.p40mg * dkp;
da(60) = dkp^2;
da(61) = plg.p20mg * da(60);
da(75) = da(60)^2;
da(64) = dkpm;
da(65) = plg.p20mg * dkpm;
da(72) = plg.p40mg * dkpm;
da(66) = dkpm^2;
da(73) = plg.p20mg * da(66);
da(76) = da(66)^2;

% ---- static part of g(L) ----------------------------------------------
f0 = a(4)*da(4)  + a(5)*da(5)  + a(6)*da(6)  + a(69)*da(69) + ...
     a(82)*da(82) + a(83)*da(83) + a(84)*da(84) + a(85)*da(85) + ...
     a(86)*da(86) + a(87)*da(87);

f1f = 1.0 + f0*ff0;

f0 = f0 + a(2)*da(2)  + a(3)*da(3)  + a(74)*da(74) + a(77)*da(77) + ...
     a(7)*da(7)   + a(8)*da(8)   + a(60)*da(60) + a(61)*da(61) + ...
     a(68)*da(68) + a(64)*da(64) + a(65)*da(65) + a(66)*da(66) + ...
     a(72)*da(72) + a(73)*da(73) + a(75)*da(75) + a(76)*da(76) + ...
     a(78)*da(78) + a(79)*da(79);

% ---- seasonal annual / semi-annual terms -------------------------------
rot  = 0.017214206;
rot2 = 0.034428412;

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

% ---- diurnal (S1) ------------------------------------------------------
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

% ---- semi-diurnal (S2) -------------------------------------------------
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

% ---- ter-diurnal (S3) --------------------------------------------------
da(35) = plg.p33 * c3h;
da(36) = plg.p33 * s3h;

% ---- periodic part -----------------------------------------------------
fp = a(9)*da(9)   + a(10)*da(10) + a(12)*da(12) + a(13)*da(13) + ...
     a(15)*da(15) + a(16)*da(16) + a(17)*da(17) + a(19)*da(19) + ...
     a(21)*da(21) + a(22)*da(22) + a(23)*da(23) + a(24)*da(24) + ...
     a(25)*da(25) + a(26)*da(26) + a(27)*da(27) + a(28)*da(28) + ...
     a(29)*da(29) + a(30)*da(30) + a(31)*da(31) + a(32)*da(32) + ...
     a(33)*da(33) + a(34)*da(34) + a(35)*da(35) + a(36)*da(36) + ...
     a(37)*da(37) + a(38)*da(38) + a(39)*da(39) + a(59)*da(59) + ...
     a(88)*da(88) + a(89)*da(89) + a(90)*da(90) + a(91)*da(91) + ...
     a(92)*da(92) + a(93)*da(93) + a(94)*da(94) + a(95)*da(95);

% ---- geomagnetic storm-time diurnal coupling (GEOGRAPHIC Legendre, dkp) -
da(40) = plg.p10 * coste * dkp;
da(41) = plg.p30 * coste * dkp;
da(42) = plg.p50 * coste * dkp;
da(43) = plg.p11 * ch    * dkp;
da(44) = plg.p31 * ch    * dkp;
da(45) = plg.p51 * ch    * dkp;
da(46) = plg.p11 * sh    * dkp;
da(47) = plg.p31 * sh    * dkp;
da(48) = plg.p51 * sh    * dkp;

fp = fp + a(40)*da(40) + a(41)*da(41) + a(42)*da(42) + a(43)*da(43) + ...
          a(44)*da(44) + a(45)*da(45) + a(46)*da(46) + a(47)*da(47) + ...
          a(48)*da(48);

% ---- second Kp adjustment (vestigial for forward eval; akp(2)=0) --------
dakp2 = (a(40)*plg.p10 + a(41)*plg.p30 + a(42)*plg.p50)*coste + ...
        (a(43)*plg.p11 + a(44)*plg.p31 + a(45)*plg.p51)*ch + ...
        (a(46)*plg.p11 + a(47)*plg.p31 + a(48)*plg.p51)*sh;
da(ikp)   = da(ikp) + dakp2*akp(2);
da(ikp+1) = da(ikp) + dakp2*c2fi*akp(2);

% ---- non-migrating (longitude) tides -----------------------------------
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

% ---- final -------------------------------------------------------------
gdel = f0 + fp * f1f;

end  % gldtm
