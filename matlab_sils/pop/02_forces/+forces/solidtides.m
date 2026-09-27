function a = solidtides(ctx)
%FORCES.SOLIDTIDES  Solid-Earth tide acceleration (ECI), IERS2010 degree-2.
%   Builds the tide-induced dC/dS (needs Sun & Moon in ECEF), forms the degree-2
%   acceleration perturbation in ECEF, then rotates to ECI.  Small at VLEO but
%   included for completeness / precise OD.
    fld = ctx.grav;
    % Sun & Moon in ECEF via the cached transform
    rSun_ecef  = ctx.C * ctx.E.sun_eci;
    rMoon_ecef = ctx.C * ctx.E.moon_eci;
    dcs = solidtides.iers2010(rMoon_ecef, rSun_ecef, fld.mu, fld.Re);
    a_ecef = tideutil.accelFromDeg2(ctx.r_ecef, dcs, fld.mu, fld.Re);
    a = ctx.Ct * a_ecef;
end
