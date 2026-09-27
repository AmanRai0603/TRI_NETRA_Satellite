function out = force(r, v, atm, facets, model, gsi, opts)
%DRAG.FORCE  Total free-molecular aerodynamic force (DRAG + LIFT + SIDE) on a
%   box-wing satellite, multi-species, attitude-dependent. Works for LEO & VLEO.
%   Density/composition/temperature are INPUTS from your atmosphere model.
%
%   atm  : .T [K], and EITHER  .rho [kg/m^3] + .Mmol [kg/kmol]  (single/mean)
%          OR  .n  (struct of number densities [m^-3] per species, e.g.
%              atm.n.O, atm.n.N2, atm.n.He, ...) for multi-species.
%          SESAM/DRIA uses atm.n.O (or atm.nO) as the atomic-oxygen density.
%   facets: struct array, each .n (body-frame outward normal) and .A [m^2].
%   model : 'sentman' | 'dria' | 'cll'
%   gsi   : .Tw [K]; sentman-> .aT ; cll-> .sig_n,.sig_t
%   opts  : .mass [kg], .Aref [m^2], .R_bi (body->inertial, default eye),
%           .wind (ECI m/s), .omega [rad/s]
%
%   out: F, a, drag, lift, side (wind-frame scalars), drag_vec, lift_vec,
%        Cd     drag coeff referenced to opts.Aref  (a bookkeeping choice)
%        Cd_A   drag coeff referenced to A_proj      (MATERIAL + GEOMETRIC: ~2-3)
%        A_proj projected area actually in the flow [m^2]
%        Cl, Cs, alpha, beta, Fbody (body-frame force), Vrel, s_O, qd.
    NA=6.02214076e26;
    if ~isfield(opts,'R_bi')||isempty(opts.R_bi), opts.R_bi=eye(3); end
    if ~isfield(opts,'wind'),  opts.wind=[0;0;0]; end
    if ~isfield(opts,'omega')||isempty(opts.omega), opts.omega=7.2921150e-5; end   % matches de440.constants().omega_earth
    % omega may be a scalar (naive z-rate) or a 3-vector (the true rate about the
    % CIP). relVelocity took only a scalar, which is why forces/drag.m could not
    % hand the panel models the same Earth rate the cannonball uses.
    [vrel,uhat,V] = drag.relVelocity(r,v,opts.wind,opts.omega);
    [alpha,beta] = drag.windAngles(vrel, opts.R_bi);

    % --- assemble species list -> [rho_s, M_s] pairs ---
    if isfield(atm,'n') && isstruct(atm.n)
        names=fieldnames(atm.n); ns=numel(names); rhoS=zeros(ns,1); Ms=zeros(ns,1); nO=0;
        for i=1:ns
            Ms(i)=drag.species(names{i}); rhoS(i)=atm.n.(names{i})*Ms(i)/NA;
            if strcmp(names{i},'O'), nO=atm.n.O; end
        end
    else
        rhoS=atm.rho; Ms=atm.Mmol; nO=0;
        if isfield(atm,'nO'), nO=atm.nO; end
    end
    if isfield(atm,'nO'), nO=atm.nO; end

    % --- accommodation (once) ---
    switch lower(model)
        % 'dria' IS "Sentman with a SESAM-derived accommodation coefficient" -- DRIA
        % = Diffuse Reflection with Incomplete Accommodation, and the incompleteness
        % is what SESAM supplies from the local atomic-oxygen environment. So 'sesam'
        % is not a separate model here; it is an ALIAS, kept because people ask for it
        % by that name. The real distinction is 'sentman' (you TYPE aT, usually 0.9)
        % vs 'dria'/'sesam' (aT comes from nO and T). At VLEO that difference is the
        % physics, not a detail: adsorbed O drives aT and it is rarely 0.9.
        case {'dria','sesam'}, aT=drag.sesam(nO,atm.T);
        case 'sentman',        aT=gsi.aT;
        case 'cll',            aT=[];        % CLL uses sig_n/sig_t instead
        otherwise
            error('drag:force:model', ...
              ['unknown GSI model "%s". Known: sentman (needs gsi.aT), dria/sesam ' ...
               '(aT from SESAM given nO,T), cll (gsi.sig_n, gsi.sig_t).'], model);
    end

    F=[0;0;0]; Aproj=0; s_O=drag.speedRatio(V,atm.T,drag.species('O'));
    for k=1:numel(facets)
        % ---- SOLAR ARRAYS ---------------------------------------------------
        % srp.addArray stores n = [0;0;0] because a tracking array's normal is not a
        % constant -- srp.boxwing computes it per call via srp.arrayNormal(axis,sun).
        % This loop used to do n = n/norm(n) on that zero vector: 0/0 = NaN, the NaN
        % summed into the force, and the run died three layers up as "dynamics
        % non-finite at t0" with no hint that a solar panel was the cause.
        %
        % A tracking array's orientation depends on the SUN, which drag does not
        % otherwise need. So opts.sunHat_eci must be supplied when arrays are present;
        % forces/drag.m passes ctx.E.sun_eci. Without it we cannot know where the
        % panel is pointing, and guessing would silently invent drag area -- so error.
        fk = facets(k);
        nb = fk.n(:);
        if isfield(fk,'type') && strcmp(fk.type,'array')
            if ~isfield(opts,'sunHat_eci') || isempty(opts.sunHat_eci)
                error('drag:force:arrayNeedsSun', ...
                  ['facet %d is a tracking solar array (type=''array''), whose normal ' ...
                   'depends on the Sun. Pass opts.sunHat_eci. Its stored .n is [0;0;0] ' ...
                   'by construction -- normalising that gives NaN, which is how this ' ...
                   'used to surface as "dynamics non-finite".'], k);
            end
            sHat_b = opts.R_bi.' * (opts.sunHat_eci(:)/norm(opts.sunHat_eci));
            nb = srp.arrayNormal(fk.axis, sHat_b);
        end
        if norm(nb) < eps
            error('drag:force:zeroNormal','facet %d has a zero normal and is not an array', k);
        end
        n=opts.R_bi*nb(:); n=n/norm(n);
        cosd=dot(n,uhat);
        % double-sided facets (arrays) are lit from either face
        if isfield(fk,'double') && fk.double && cosd<0, n=-n; cosd=-cosd; end
        if cosd<=0, continue; end
        Aproj = Aproj + fk.A*cosd;          % the area the flow actually sees
        delta=acos(min(cosd,1)); sind=sqrt(max(1-cosd^2,0));
        if sind>1e-9, tgas=(-uhat+cosd*n)/sind; else, tgas=[0;0;0]; end
        for j=1:numel(rhoS)
            s=drag.speedRatio(V, atm.T, Ms(j));
            % NOTE this duplicates drag.panelCoeffs' dispatch. Two switches for one
            % decision is how they drift apart: panelCoeffs knew about 'sesam' and
            % this one did not, so drag.force('sesam') fell through BOTH cases,
            % never assigned cp/ct, and died with "'ct' undefined" -- an unknown
            % model should say so, not surface as an undefined variable three lines
            % later. The 'otherwise' below closes that; collapsing the two switches
            % into panelCoeffs is the real fix and is left as a deliberate TODO
            % (panelCoeffs takes gsi.Talt where this has atm.T to hand).
            switch lower(model)
                case {'sentman','dria','sesam'}
                    [cp,ct]=drag.sentman(s,delta,aT,gsi.Tw,atm.T);
                case 'cll'
                    [cp,ct]=drag.cll(s,delta,gsi.sig_n,gsi.sig_t,gsi.Tw,atm.T);
                otherwise
                    error('drag:force:model','unknown GSI model "%s"', model);
            end
            F = F + 0.5*rhoS(j)*V^2*fk.A*(ct*tgas - cp*n);
        end
    end

    % --- wind-frame decomposition (drag, side, lift) ---
    Fbody = opts.R_bi.'*F; Fwind = drag.bodyToWind(alpha,beta)*Fbody;
    D=-Fwind(1); S=Fwind(2); L=-Fwind(3);
    rho_tot=sum(rhoS); qd=0.5*rho_tot*V^2;
    out.F=F; out.a=F/opts.mass; out.Fbody=Fbody;
    out.drag=D; out.side=S; out.lift=L;
    out.drag_vec=(dot(F,uhat))*uhat; out.lift_vec=F-(dot(F,uhat))*uhat;
    out.uhat=uhat; out.Vrel=V; out.s_O=s_O; out.qd=qd;
    out.alpha=alpha; out.beta=beta;
    % ---- TWO Cds, and only one of them is physics -------------------------------
    % out.Cd is referenced to opts.Aref, which is a BOOKKEEPING choice -- whatever
    % number the catalog happens to call the reference area. For a 16U whose bus face
    % is 0.04 m^2 but which flies 0.068 m^2 of bus+array into the flow, that gives
    % Cd = 4.0 and looks broken. It is not: Cd is DEFINED as D/(q*Aref), and dividing
    % by a small Aref makes a big coefficient.
    %
    % out.Cd_A is referenced to A_proj, the area the flow ACTUALLY sees. That is the
    % material+geometric drag coefficient -- what gas-surface physics returns, ~2-3
    % for free-molecular flow, and the number to sanity-check against literature.
    % Measured on the 16U at 300 km: Cd = 4.0129, Cd_A = 2.3605, ratio 1.7000 =
    % exactly A_proj/Aref.
    %
    % A coefficient without its reference area is half a statement. Both are returned
    % so nobody has to guess which convention a number is in -- and the drag force
    % itself never depended on the choice.
    out.Cd=D/(qd*opts.Aref); out.Cl=L/(qd*opts.Aref); out.Cs=S/(qd*opts.Aref);
    out.A_proj = Aproj;
    if Aproj > 0
        out.Cd_A = D/(qd*Aproj);   % material + geometric
    else
        out.Cd_A = NaN;            % nothing in the flow: no coefficient to speak of
    end
    out.Aref_used = opts.Aref;
end
