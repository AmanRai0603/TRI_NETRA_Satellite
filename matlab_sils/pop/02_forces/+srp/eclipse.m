function nu = eclipse(rSat, rSun, model, opts)
%SRP.ECLIPSE  Shadow lighting fraction nu in [0,1] (1 sun, 0 umbra).
%   nu = srp.eclipse(rSat, rSun[, model][, opts])
%
%   model : 'cylindrical' | 'conical' (default) | 'fine'
%     cylindrical  - hard on/off, spherical Earth. Fastest, no penumbra.
%     conical      - dual-cone umbra+penumbra via fractional solar-disk
%                    occultation (spherical Earth). Standard, recommended.
%     fine         - conical + oblate Earth (z-scaling to a sphere) +
%                    opaque atmosphere layer of height opts.hAtm. Softens and
%                    slightly shifts the penumbra. See docs/ECLIPSE_MODELS.md.
%
%   opts fields (all optional): Re=6378137, Rp=6356752.314245, Rsun=6.957e8, hAtm=12000 (match de440.constants)
%   rSat, rSun in ECI [m] (rSun geocentric, from de440.sun / ephemInputs.sun_eci).
    if nargin < 3 || isempty(model), model = 'conical'; end
    if nargin < 4, opts = struct(); end
    Kc=de440.constants(); Re=getf(opts,'Re',Kc.Re_earth); Rp=getf(opts,'Rp',Kc.Rp_earth);
    Rsun = getf(opts,'Rsun',6.957e8); hAtm = getf(opts,'hAtm',12000.0);
    rSat = rSat(:); rSun = rSun(:);
    switch lower(model)
        case 'cylindrical'
            sHat = (rSun - rSat)/norm(rSun - rSat);
            if dot(-rSat, sHat) < 0, nu = 1; return; end
            perp = norm(-rSat - dot(-rSat,sHat)*sHat);
            nu = double(perp >= Re);
        case 'conical'
            nu = fracConical(rSat, rSun, Re, Rsun);
        case 'fine'
            S = [1;1;Re/Rp];                 % scale ellipsoid -> sphere of radius Re
            nu = fracConical(rSat.*S, rSun.*S, Re + hAtm, Rsun);
        otherwise
            error('srp:eclipse','unknown model "%s"', model);
    end
end

function nu = fracConical(rSat, rSun, Re, Rsun)
    dSun = rSun - rSat; ds = norm(dSun); sHat = dSun/ds;
    rr = norm(rSat); eHat = -rSat/rr;
    if dot(sHat, eHat) < 0, nu = 1; return; end          % day side
    th_s   = asin(min(Rsun/ds,1));                        % Sun apparent radius
    th_e   = asin(min(Re/rr,1));                          % Earth apparent radius
    th_sep = acos(max(min(dot(sHat,eHat),1),-1));         % Sun-Earth separation
    if th_sep >= th_s + th_e, nu = 1; return; end         % no overlap
    if th_e - th_s >= th_sep, nu = 0; return; end         % Sun fully hidden (umbra)
    if th_s - th_e >= th_sep, nu = 1 - (th_e/th_s)^2; return; end   % annular
    occ = lensArea(th_sep, th_s, th_e);                   % partial: lens overlap
    nu  = 1 - occ/(pi*th_s^2);
end

function A = lensArea(d, r1, r2)
    if d >= r1 + r2, A = 0; return; end
    if d <= abs(r1 - r2), A = pi*min(r1,r2)^2; return; end
    a = (d*d + r1*r1 - r2*r2)/(2*d);
    h = sqrt(max(r1*r1 - a*a, 0));
    A = r1*r1*acos(a/r1) + r2*r2*acos((d-a)/r2) - d*h;
end

function v = getf(s,f,def); if isfield(s,f), v = s.(f); else, v = def; end; end
