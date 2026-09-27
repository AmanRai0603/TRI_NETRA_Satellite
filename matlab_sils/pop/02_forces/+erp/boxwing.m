function [a, comp] = boxwing(rSat, rSun, R_b2i, sc, doy, nrings, nseg)
%ERP.BOXWING  Earth radiation pressure on a box-wing spacecraft.
%   [a, comp] = erp.boxwing(rSat, rSun, R_b2i, sc[, doy][, nrings][, nseg])
%
%   The attitude-dependent counterpart of erp.knocke. Same Knocke ring integration
%   of the visible Earth cap -- but instead of collapsing the spacecraft to a single
%   CrAoM number, each element's beam is applied to the FACETS, so the answer depends
%   on how the satellite is pointing and on each surface's optics.
%
%   ---------------------------------------------------------------------------
%   WHY THIS IS NOT JUST srp.boxwing WITH A DIFFERENT SOURCE
%   ---------------------------------------------------------------------------
%   SRP has ONE source direction: the Sun is effectively a point 1 AU away, so every
%   facet sees the same sHat. The Earth is not a point. At 350 km it subtends about
%   140 degrees, so different elements of the cap illuminate the satellite from
%   genuinely different directions -- including from BEHIND facets that the net flux
%   direction would call illuminated. Collapsing the cap to one effective direction
%   and reusing srp.boxwing would be an approximation, and a bad one at low altitude
%   where ERP matters most.
%
%   So the facet sum lives INSIDE the element loop. That is the whole cost of this
%   function: nrings*nseg*numel(facets) facet evaluations per call (768*6 ~ 4.6k at
%   the defaults), inside the integrator's RHS. erp.knocke already pays the 768; this
%   pays 6x more. If that hurts, drop nrings/nseg -- the cap integration converges
%   quickly -- rather than reaching for the cannonball, which cannot see attitude at
%   all.
%
%   ---------------------------------------------------------------------------
%   INPUTS
%   ---------------------------------------------------------------------------
%     rSat, rSun   ECI [m]
%     R_b2i        body->inertial DCM (from ctx.sc.R_bi, i.e. the attitude)
%     sc           .mass and .facets in the SRP format (srp.facet):
%                    .type ('body'|'array') .n .A .alpha .rho_s .rho_d .axis .double
%                  The SAME facets drag and SRP use. One geometry, three forces.
%     doy          day of year (drives the zonal albedo/emissivity)
%
%   comp.sw / comp.lw split albedo (short-wave) and IR (long-wave), as in erp.knocke.
%
%   ---------------------------------------------------------------------------
%   OPTICS ACROSS BANDS -- a real limitation, stated
%   ---------------------------------------------------------------------------
%   srp.facet carries ONE set of (alpha, rho_s, rho_d). Real surfaces do not have the
%   same optical properties in the visible (albedo, ~0.5 um) and the thermal IR
%   (~10 um): MLI and solar cells differ a lot between the two. This function applies
%   the same coefficients to both bands because that is what the facet struct holds.
%   The albedo/IR SPLIT is still physical (comp.sw/comp.lw), so the error is in the
%   surface response, not the source. Fixing it properly means adding IR optics to
%   srp.facet -- a change to the geometry contract, not to this file.
    if nargin<5||isempty(doy),    doy=80;  end
    if nargin<6||isempty(nrings), nrings=16;end
    if nargin<7||isempty(nseg),   nseg=48; end
    if ~isfield(sc,'facets') || isempty(sc.facets)
        error('erp:boxwing:noFacets', ...
          ['erp.boxwing needs sc.facets (srp format). Without a geometry there is ' ...
           'nothing for the attitude to act on -- use model ''knocke'' instead, which ' ...
           'is the cannonball and does not read attitude.']);
    end

    Kc=de440.constants(); Re=Kc.Re_earth; S=Kc.TSI; c=Kc.c;
    rSat=rSat(:); rSun=rSun(:);
    d=norm(rSat); zhat=rSat/d; shat=rSun/norm(rSun);
    rho_max=acos(min(Re/d,1));
    t=[1;0;0]; if abs(zhat(1))>0.9, t=[0;1;0]; end
    e1=cross(zhat,t); e1=e1/norm(e1); e2=cross(zhat,e1);

    Fsw=[0;0;0]; Flw=[0;0;0];
    for ir=1:nrings
        psi=(ir-0.5)/nrings*rho_max; dpsi=rho_max/nrings;
        for js=1:nseg
            az=2*pi*(js-0.5)/nseg;
            n_el=cos(psi)*zhat+sin(psi)*(cos(az)*e1+sin(az)*e2);
            r_el=Re*n_el; svec=rSat-r_el; rho=norm(svec); es=svec/rho;
            cos_e=dot(n_el,es); if cos_e<=0, continue; end
            dA=Re*Re*sin(psi)*dpsi*(2*pi/nseg);
            lat=asin(max(min(n_el(3),1),-1));
            [alb,emi]=erp.zonalCoeffs(lat,doy);
            cz=dot(n_el,shat);
            Msw=0; if cz>0, Msw=alb*S*cz; end
            Mlw=emi*S/4;

            % irradiance at the satellite from THIS element [W/m^2], per band
            g   = cos_e*dA/(pi*rho*rho);
            Esw = Msw*g;  Elw = Mlw*g;
            if Esw<=0 && Elw<=0, continue; end

            % The element lies at -es as seen from the satellite, so the beam ARRIVES
            % from -es. The srp.boxwing formula is written with sHat = sat->source, so
            % the analogue here is uHat = -es, expressed in BODY axes.
            uHat_b = R_b2i.' * (-es);

            % facet response, same form as srp.boxwing (Montenbruck & Gill / McMahon)
            fsw = facetSum_(sc.facets, uHat_b, Esw/c);
            flw = facetSum_(sc.facets, uHat_b, Elw/c);
            Fsw = Fsw + R_b2i*fsw;
            Flw = Flw + R_b2i*flw;
        end
    end
    comp.sw = Fsw/sc.mass;
    comp.lw = Flw/sc.mass;
    a = comp.sw + comp.lw;
end

function Fb = facetSum_(facets, uHat_b, P)
%FACETSUM_  Body-frame force from one beam of radiation pressure P arriving along
%   -uHat_b (i.e. uHat_b points from the spacecraft TOWARD the source).
    Fb = [0;0;0];
    for k = 1:numel(facets)
        f = facets(k);
        if strcmp(f.type,'array')
            n = srp.arrayNormal(f.axis, uHat_b);
        else
            n = f.n;
        end
        cth = dot(n, uHat_b);
        if f.double && cth < 0, n = -n; cth = -cth; end
        if cth <= 0, continue; end                % facet not lit by THIS element
        Fb = Fb - P*f.A*cth*( (f.alpha + f.rho_d)*uHat_b ...
                              + 2*(f.rho_s*cth + f.rho_d/3)*n );
    end
end
