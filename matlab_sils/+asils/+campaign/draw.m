function [set, d] = draw(C, P0, k)
%ASILS.CAMPAIGN.DRAW  Dispersed overrides for Monte Carlo run k.
%   Every run draws from its own stream: seed = C.seed + 7919*k, so run k is
%   the same run whatever machine or worker executes it (SPEC.md 10.2 rule).
%   Returns set (asils.config override struct) and d (the drawn values, for
%   scatter plots). Sensor/actuator part errors are drawn inside the run
%   from the part descriptors (asils.devices.init) with the run's seed.
    rng(C.seed + 7919*k, 'twister');
    set = struct(); d = struct();
    if isfield(C, 'duration_s'), set.sim__duration_s = C.duration_s; end
    U = @(a, b) a + (b - a)*rand;
    ds = C.dispersions; if ~iscell(ds), ds = num2cell(ds); end
    % edge campaign (type = "edge"): run 2j-1 / 2j puts dispersion j at its
    % low / high bound with every other dispersion nominal; run 2n+1 puts
    % every dispersion at its adverse (upper) bound
    edge = strcmp(asils.util.getf(C, 'type', 'mc'), 'edge');
    which = 0; hi = true;
    if edge && k <= 2*numel(ds), which = ceil(k/2); hi = mod(k, 2) == 0; end
    for i = 1:numel(ds)
        s = ds{i};
        if edge
            if which > 0 && i ~= which, continue, end
            if which > 0, U = @(a, b) edge_(a, b, hi);
            else, U = @(a, b) b; end               % adverse = upper bound for every kind below
            d.edge_case = which; d.edge_high = double(hi);
        end
        switch s.kind
            case 'inertia'           % each principal moment +/- s.frac (uniform)
                f = 1 + s.frac*(2*rand(3,1) - 1);
                if edge, f = 1 + s.frac*(2*U(0, 1) - 1)*[1; -1; 1]; end
                set.sc__I = diag(diag(P0.sc.I).*f); d.inertia_scale_x = f(1); d.inertia_scale_y = f(2); d.inertia_scale_z = f(3);
            case 'mass'
                v = P0.sc.mass_kg*(1 + s.frac*randn); set.sc__mass_kg = v; d.mass_kg = v;
            case 'cm_offset'         % random direction, magnitude uniform in [lo, hi] x case CP-CM
                u = randn(3,1); u = u/norm(u); m = norm(P0.sc.cm_offset_m)*U(s.lo, s.hi);
                set.sc__cm_offset_m = m*u; d.cm_offset_mm = m*1000;
            case 'residual_dipole'
                u = randn(3,1); u = u/norm(u); m = norm(P0.sc.m_res)*U(s.lo, s.hi);
                set.sc__m_res = m*u; d.residual_dipole_Am2 = m;
            case 'solar_flux'
                v = U(s.lo, s.hi); set.env__F107 = v; set.env__F107a = v; d.F107 = v;
            case 'kp'
                v = U(s.lo, s.hi); set.env__Kp = v; set.env__ap = kp2ap(v); d.Kp = v;
            case 'accommodation'
                v = U(s.lo, s.hi); set.sc__sigma_n = v; set.sc__sigma_t = v; d.sigma_accom = v;
            case 'reflectivity'
                v = U(s.lo, s.hi); set.sc__refl = v; d.reflectivity = v;
            case 'initial_error_deg'
                u = randn(3,1); u = u/norm(u); v = U(s.lo, s.hi);
                set.scenario__initial__attitude__axis_body = u; set.scenario__initial__attitude__angle_deg = v;
                d.initial_error_deg = v;
            case 'initial_rate_deg_s'
                v = U(s.lo, s.hi);
                set.scenario__initial__rate__magnitude_deg_s = v; d.initial_rate_deg_s = v;
            case 'epoch_days'          % the season: the mission starts this many days later
                v = U(s.lo, s.hi); e = asils.util.jd2utc(asils.util.jd(P0.epoch_utc) + v); e(6) = round(e(6));
                set.epoch_utc = e; d.epoch_days = v;
            case 'ltan_h'              % the beta angle: the truth orbit's local time of the ascending node
                v = U(s.lo, s.hi); set.orbit__ltan_h = v; d.ltan_h = v;
            case 'alt_km'              % the truth orbit's altitude (the flight software keeps the nominal)
                v = U(s.lo, s.hi); a = 6378137 + 1e3*v;
                set.orbit__alt_km = v; set.orbit__period_s = 2*pi*sqrt(a^3/P0.mu); d.alt_km = v;
            case 'inertia_products'    % Ixy, Ixz, Iyz, each a fraction of sqrt(I_ii I_jj)
                f = [U(s.lo, s.hi), U(s.lo, s.hi), U(s.lo, s.hi)];
                if isfield(set, 'sc__I'), I = set.sc__I; else, I = P0.sc.I; end
                ij = [1 2; 1 3; 2 3];
                for q = 1:3
                    a = ij(q, 1); b = ij(q, 2); I(a, b) = f(q)*sqrt(I(a, a)*I(b, b)); I(b, a) = I(a, b);
                end
                set.sc__I = I; d.product_xy = f(1); d.product_xz = f(2); d.product_yz = f(3);
            case 'arg_lat_deg'
                v = U(0, 360); set.orbit__u0_deg = v; d.arg_lat_deg = v;
            otherwise
                error('asils:campaign:kind', 'unknown dispersion %s', s.kind);
        end
    end
end

function v = edge_(a, b, hi)
    if hi, v = b; else, v = a; end
end

function ap = kp2ap(Kp)
%KP2AP  The standard Kp -> ap table, nearest node: the propagator's kp2ap
%   (atmos.spaceweather). ap ends at 400 (Kp 9).
    kpv = [0 .33 .67 1 1.33 1.67 2 2.33 2.67 3 3.33 3.67 4 4.33 4.67 5 5.33 5.67 6 6.33 6.67 7 7.33 7.67 8 8.33 8.67 9];
    apv = [0 2 3 4 5 6 7 9 12 15 18 22 27 32 39 48 56 67 80 94 111 132 154 179 207 236 300 400];
    ap = interp1(kpv, apv, max(0, min(9, Kp)), 'nearest');
end
