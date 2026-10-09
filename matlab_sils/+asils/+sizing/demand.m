function Dm = demand(caseFile, k)
%ASILS.SIZING.DEMAND  What the case asks of an attitude actuator (the engine's adcs-design demand, in MATLAB).
%   Dm = asils.sizing.demand('cases/ais_3u.csv')
%   Dm = asils.sizing.demand('cases/ais_3u.csv', asils.sizing.knobs(struct('k_tau', 2)))
%
%   Every survey of the design's sweep (sizedemand's survey_runs and survey_sweep: the seasons and the solar activity)
%   flown: one orbit of the case on the POP (the scenario sizedemand's survey_setup states, the product TRN-P-3U-AIS),
%   at every sample the environment (asils.orbit.env), the four attitudes' reference by the flight software's guidance
%   (+asils/+alg, sizedemand's survey_attitude) and the disturbance torques by env's models (asils.env.torques); the
%   design reduces them (survey_orbit, survey_field, eclipse's eclipse_fraction), takes the worst of the sweep
%   (survey_worst) and the case's lines with its defaults where the case is blank (sizedemand's demand). Every law is
%   the design's, generated into +asils/+models; here only the steps and the reading of the case.
%   Owner: Agastya. Copyright (c) 2026 Agastya. All rights reserved.
    if nargin < 2 || isempty(k), k = asils.sizing.knobs(); end
    att = {'X_nadir', 'Y_nadir', 'Z_nadir', 'sun'};
    base = []; sweep = {};
    for i = 0:(asils.models.sizedemand.survey_runs() - 1)
        [ep, f] = asils.models.sizedemand.survey_sweep(i);
        [P, sv] = survey_(caseFile, struct('engine__epoch_days', ep, 'engine__f107', f, 'engine__f107a', f));
        sweep{end+1} = struct('epoch_days', ep, 'f107', f, 'tau_peak', sv.tau_peak.', 'h_secular_orbit', sv.h_secular_orbit.', ...
            'eclipse_frac', sv.eclipse_frac); %#ok<AGROW>
        if isempty(base)
            base = struct('P', P, 'sv', sv);
        else
            w = base.sv;
            [tp, tap, hc, hs, bmin, ecl] = asils.models.sizedemand.survey_worst(w.tau_peak, sv.tau_peak, w.tau_axis_peak, sv.tau_axis_peak, ...
                w.h_cyclic, sv.h_cyclic, w.h_secular_orbit, sv.h_secular_orbit, w.b_min, sv.b_min, w.eclipse_frac, sv.eclipse_frac);
            base.sv = struct('tau_peak', tp, 'tau_axis_peak', tap, 'h_cyclic', hc, 'h_secular_orbit', hs, 'b_min', bmin, ...
                'b_mean', w.b_mean, 'eclipse_frac', ecl);
        end
    end
    P = base.P; sv = base.sv; v = P.case.v;
    t_orb = P.orbit.period_s;
    j = [P.sc.I(1, 1); P.sc.I(2, 2); P.sc.I(3, 3)];
    kh = k.k_h; if isempty(kh), kh = NaN; end
    [worst, tau_dist, h_dist, h_secular, dump_dflt, w0, w0_dflt, h_detumble, slew_deg, sangle_dflt, slew_s, slew_dflt, ...
     w_slew, a_slew, h_slew, tau_slew, life_yr, life_dflt, slews_per_day, k_h, h_req, tau_req, fine] = asils.models.sizedemand.demand(sv.tau_peak, ...
        sv.h_cyclic, sv.h_secular_orbit, t_orb, v.req_dump, v.mission_w0, v.mission_sangle, v.req_slew, v.mission_life, ...
        v.mission_spd, v.req_hsat, kh, k.k_tau, j, v.req_ake);
    % each default the design took where the case is blank, said in the case's terms
    notes = {};
    if dump_dflt, notes{end+1} = 'req.dump blank: a quarter orbit of secular momentum held between dumps taken'; end
    if w0_dflt, notes{end+1} = sprintf('mission.w0 blank: %s deg/s taken', num_(w0)); end
    if sangle_dflt, notes{end+1} = sprintf('mission.sangle blank: %s deg taken', num_(slew_deg)); end
    if slew_dflt, notes{end+1} = sprintf('req.slew blank: %s s taken for the reference slew', num_(slew_s)); end
    if life_dflt, notes{end+1} = sprintf('mission.life blank: %s years taken', num_(life_yr)); end
    req = struct();
    for r = {'ape', 'ake', 'rks', 'mass', 'pavg', 'ppk', 'vol', 'detumble', 'sunacq'}, req.(r{1}) = v.(['req_' r{1}]); end
    % what the platform allocates to the ADCS (blank: no allocation stated, nothing checked)
    for r = {'malloc', 'palloc', 'valloc'}, req.(r{1}) = v.(['resources_' r{1}]); end
    Dm = struct('case', P.case.id, 'orbit_period_s', t_orb, 'attitudes', {att}, 'tau_peak', sv.tau_peak.', ...
        'h_cyclic', sv.h_cyclic.', 'h_secular_orbit', sv.h_secular_orbit.', 'tau_axis_peak', sv.tau_axis_peak, ...
        'tau_dist', tau_dist, 'worst_attitude', att{worst + 1}, 'h_dist', h_dist, 'h_secular', h_secular, ...
        'B_min', sv.b_min, 'B_mean', sv.b_mean, 'eclipse_frac', sv.eclipse_frac, 'w0_deg_s', w0, 'J', j.', ...
        'h_detumble', h_detumble, 'slew_deg', slew_deg, 'slew_s', slew_s, 'w_slew', w_slew, 'a_slew', a_slew, ...
        'h_slew', h_slew, 'tau_slew', tau_slew, 'life_yr', life_yr, 'slews_per_day', slews_per_day, 'k_h', k_h, ...
        'k_tau', k.k_tau, 'h_req', h_req, 'tau_req', tau_req, 'req', req, 'notes', {notes}, ...
        'class', ifelse_(fine, 'fine', 'coarse'), 'body_class', P.case.class, 'box_m', reshape(P.sc.box_m, 1, 3), ...
        'survey_sweep', {sweep});
end

function [P, sv] = survey_(caseFile, set)
% one survey: the orbit on the POP, at each sample the environment, the four attitudes' reference and torques
    [duration, dt, record_dt] = asils.models.sizedemand.survey_setup();
    S = struct('schema', 'adcs-scenario/1', 'id', 'sizing_survey', 'product', 'TRN-P-3U-AIS', 'label', 'sizing survey', ...
        'time', struct('duration_s', duration, 'dt_s', dt, 'record_dt_s', record_dt), ...
        'initial', struct('attitude', struct('kind', 'nadir'), 'rate', struct('kind', 'lvlh')), ...
        'fsw', struct('start_mode', 'detumble', 'guidance', struct('kind', 'nadir')), 'metrics', {{}});
    P = asils.config(S, caseFile, struct('seed', 1, 'set', set));
    O = asils.orbit.init(P);
    gh = asils.models.frames.igrf_gh(asils.models.caltime.decimal_year(P.jd0));
    facets = asils.models.facets.facets_box(P.sc.box_m, P.sc.cm_offset_m, P.sc.sigma_n, P.sc.sigma_t, P.sc.vb_ratio, ...
        P.sc.refl, P.sc.spec_frac);
    t_orb = P.orbit.period_s;
    n = asils.models.sizedemand.survey_samples(t_orb, dt);
    A = cell(4, 1); qoff = cell(4, 1);
    for a = 1:4
        [bs, sun, mode, sun_axis, roll_axis] = asils.models.sizedemand.survey_attitude(a - 1);
        A{a} = struct('sun', sun, 'mode', mode, 'sun_axis', sun_axis, 'roll_axis', roll_axis);
        if ~sun, qoff{a} = asils.alg.guidance.boresight_offset(bs); end
    end
    z3 = zeros(3, 1); z4 = zeros(4, 1);
    tau = zeros(12*n, 1); bm = zeros(n, 1); nu = zeros(n, 1);
    for kk = 0:(n - 1)
        t = kk*dt;
        [r, vv, O] = asils.orbit.state(O, t);
        E = asils.orbit.env(O, t, P.jd0, r, vv, gh, P.env.igrf_nmax);
        bm(kk + 1) = norm3_(E.b_eci); nu(kk + 1) = E.nu;
        for a = 1:4
            g = A{a};
            if ~g.sun   % the payload axis on nadir: the boresight offset, nothing else (Guid's other fields zero)
                q = asils.alg.guidance.guidance(g.mode, r, vv, t, qoff{a}, 0, 0, 0, z3, z4, z3, z3, z3, false);
            else        % the Sun on the power face
                q = asils.alg.guidance.guidance(g.mode, r, vv, t, z4, 0, 0, 0, z3, z4, g.sun_axis, g.roll_axis, unit_(E.sun_rel), false);
            end
            p = asils.env.torques(q, r, E.v_rel, E.b_eci, E.sun_rel, E.nu, E.p_srp, E.rho, P.sc.I, facets, P.sc.m_res, P.mu, P.env.on);
            tau(12*kk + 3*(a - 1) + (1:3)) = (p(:, 1) + p(:, 2)) + (p(:, 3) + p(:, 4));
        end
    end
    sv = struct();
    [sv.tau_peak, sv.tau_axis_peak, sv.h_cyclic, sv.h_secular_orbit] = asils.models.sizedemand.survey_orbit(tau, n, dt, t_orb);
    [sv.b_min, sv.b_mean] = asils.models.sizedemand.survey_field(bm, n);
    % env's eclipse fraction (m2_7) holds at most ECLIPSE_NMAX (2048) samples
    assert(n >= 1 && n <= 2048, 'asils:sizing:refused', ...
        'the sizing survey''s orbit has %d samples; the eclipse fraction (m2_7) takes 1 to 2048', n);
    nup = zeros(2048, 1); nup(1:n) = nu;
    sv.eclipse_frac = asils.models.eclipse.eclipse_fraction(nup, n);
end

function n = norm3_(a)
    n = sqrt(a(1)*a(1) + a(2)*a(2) + a(3)*a(3));
end
function u = unit_(a)
% the engine's la unit: a times one over its norm, zero below 1e-300
    n = norm3_(a);
    if n < 1e-300, u = zeros(3, 1); else, u = a*(1.0/n); end
end
function s = num_(x)
% a number as Rust's Display writes it (the shortest that reads back)
    if x == round(x) && abs(x) < 1e15, s = sprintf('%d', x); return, end
    s = sprintf('%.17g', x);
    for p = 1:17
        c = sprintf('%.*g', p, x);
        if str2double(c) == x, s = c; break; end
    end
end
function x = ifelse_(c, a, b)
    if c, x = a; else, x = b; end
end
