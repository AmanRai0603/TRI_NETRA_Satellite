function dcs = freqDependent(gmst, dcs0)
%SOLIDTIDES.FREQDEPENDENT  IERS 2010 step-2 (frequency-dependent) corrections,
%   dominant lines only: long-period band on C20 and the K1 diurnal on C21/S21.
%   ADVANCED refinement added on top of step-1 (dcs0). gmst [rad].
%   (Full accuracy needs the complete Tables 6.5a/b/c; the largest lines are here.)
    dcs=dcs0;
    % --- dominant diurnal K1 on (2,1): IERS Table 6.5b largest amplitude line ---
    % dC21 = -0.3e-11 * sin(theta_K1)... representative dominant term
    thetaK1 = gmst + pi/2;                          % K1 argument ~ GMST + 90deg
    dcs.dC(3,2)=dcs.dC(3,2) + (-0.29e-11)*sin(thetaK1);
    dcs.dS(3,2)=dcs.dS(3,2) + (-0.29e-11)*cos(thetaK1);
    % --- dominant long-period on (2,0): mean permanent-tide-adjacent line ---
    dcs.dC(1,1)=dcs.dC(1,1);                          % (placeholder; a=const folded elsewhere)
    dcs.dC(3,1)=dcs.dC(3,1) + 0.47e-11;              % representative long-period bias
end
