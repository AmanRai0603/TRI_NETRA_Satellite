function a = gravity(ctx)
%FORCES.GRAVITY  Central-body gravitational acceleration (ECI), model-switchable.
%   a = forces.gravity(ctx)
%   Reads ctx.cfg.forces.gravity: .model .degree .order and ctx.grav (field).
%   Models:
%     'twobody'  point mass (mu from field)
%     'j2'..'j6' fast analytic zonal (grav.j2accel)
%     'sphharm'  native normalised spherical harmonics (grav.sphericalHarmonic)
%     'toolbox'  MATLAB Aerospace gravitysphericalharmonic (needs .mat field+TB)
%   Gravity is evaluated in ECEF (body-fixed) then rotated back to ECI with the
%   cached transform in ctx.C/ctx.Ct.
    g   = ctx.cfg.forces.gravity;
    fld = ctx.grav;
    switch lower(g.model)
        case 'twobody'
            a = grav.twoBody(ctx.r_eci, fld.mu);
            return
        case {'j2','j3','j4','j5','j6'}
            nz = str2double(g.model(2));           % 2..6
            J  = fld.J(1:nz-1);                     % [J2..Jn]
            a_ecef = grav.j2accel(ctx.r_ecef, fld.mu, fld.Re, J);
        case 'sphharm'
            nmax = min(g.degree, fld.nmax);
            mmax = min(getf(g,'order',nmax), nmax);
            a_ecef = grav.sphericalHarmonic(ctx.r_ecef, fld.mu, fld.Re, ...
                        fld.Cbar, fld.Sbar, nmax, mmax);
        case 'toolbox'
            % Aerospace Toolbox path (Pines/Gottlieb, high degree). fld.matfile
            % must point at a {C,S} .mat compatible with gravitysphericalharmonic.
            [gx,gy,gz] = gravitysphericalharmonic(ctx.r_ecef.', 'Custom', ...
                            double(g.degree), {fld.matfile, @load}, 'Error');
            a_ecef = [gx;gy;gz];
        otherwise
            error('forces:gravity','unknown gravity model "%s"',g.model);
    end
    a = ctx.Ct * a_ecef;                            % ECEF -> ECI
end
function val=getf(s,f,d), if isfield(s,f)&&~isempty(s.(f)), val=s.(f); else, val=d; end, end
