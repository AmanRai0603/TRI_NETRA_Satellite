function a = oceantides(ctx)
%FORCES.OCEANTIDES  Ocean-tide acceleration (ECI), main lines (FES-based).
%   Delegates to oceantides.mainLines using the TT date; converts the resulting
%   degree-2 dC/dS to an acceleration in ECEF and rotates to ECI.  Requires the
%   FES data file (force_data/fes2004_deg10.mat) on the path for 'fromModel'.
    fld = ctx.grav;
    dcs = oceantides.mainLines(ctx.T.tt_jd);
    a_ecef = tideutil.accelFromDeg2(ctx.r_ecef, dcs, fld.mu, fld.Re);
    a = ctx.Ct * a_ecef;
end
