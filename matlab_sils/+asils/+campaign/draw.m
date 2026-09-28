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
                v = U(s.lo, s.hi); set.env__Kp = v; set.env__ap = round(exp(1.07*v + 0.9)); d.Kp = v;
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
