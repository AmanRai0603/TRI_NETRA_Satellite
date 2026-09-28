function Dm = demand(caseFile, opts)
%ASILS.SIZING.DEMAND  What the case asks of an attitude actuator.
%   Dm = asils.sizing.demand('cases/ais_3u.csv')
%
%   1 DISTURBANCES. One orbit on the case's orbit (the in-loop POP, the same
%     torque models the SILS flies: gravity gradient, per-facet aero and SRP,
%     residual dipole) at four held attitudes: each body axis on nadir, and the
%     Sun-referenced attitude. For each: peak torque, the momentum the torque
%     builds up in body axes over the orbit -- its cyclic part (what a rotor
%     must store between dumps) and its secular part (what must be dumped).
%     The worst attitude sizes the actuator.
%   2 FIELD. |B| along the orbit: the weakest field sets the coil dipole.
%   3 DETUMBLE. H0 = J_max w0 (case mission.w0).
%   4 SLEW. mission.sangle in req.slew seconds (60 s when the case is blank),
%     cycloidal rest-to-rest: w_pk = 2 theta/T, a_pk = 2 pi theta/T^2.
%   All in SI; Dm.notes lists every assumption taken for a blank case value.
    if nargin < 2, opts = struct(); end
    S = struct('id', 'sizing_survey', 'product', asils.util.getf(opts, 'product', 'TRN-P-3U-AIS'), 'label', 'sizing survey', ...
        'time', struct('duration_s', 5740, 'dt_s', 10, 'record_dt_s', 10), ...
        'initial', struct('attitude', struct('kind', 'nadir'), 'rate', struct('kind', 'lvlh')), ...
        'fsw', struct('start_mode', 'detumble', 'guidance', struct('kind', 'nadir')), 'metrics', {{}});
    P = asils.config(S, caseFile, struct('seed', 1));
    v = P.case.v; notes = {};
    O = asils.orbit.init(P);
    igrf = asils.env.igrf_coefs(); jd0 = asils.util.jd(P.epoch_utc);
    gh = asils.env.igrf_gh(asils.util.decyear(jd0), igrf);
    G = asils.env.geometry(P.sc); I = P.sc.I;
    T = P.orbit.period_s; dt = 10; ts = 0:dt:T; n = numel(ts);
    att = {'X_nadir', 'Y_nadir', 'Z_nadir', 'sun'};
    offs = {asils.fsw.boresight_offset([1;0;0]), asils.fsw.boresight_offset([0;1;0]), asils.fsw.boresight_offset([0;0;1])};
    tau = zeros(3, n, 4); Bm = zeros(1, n); nu = zeros(1, n);
    for k = 1:n
        t = ts(k);
        [r, vv, O] = asils.orbit.state(O, t);
        X = asils.orbit.context(O, t);
        B_eci = asils.env.field(X.C*r, X.C, gh, 13);
        sun_rel = X.sun_eci - r; nu(k) = asils.env.shadow(r, X.sun_eci);
        v_rel = vv - O.omega_e*[-r(2); r(1); 0];
        Bm(k) = norm(B_eci);
        for a = 1:4
            if a < 4
                q = asils.fsw.guidance('nadir', r, vv, t, struct('q_off', offs{a}));
            else
                q = asils.fsw.guidance('sun', r, vv, t, struct('sun_eci', sun_rel/norm(sun_rel)));
            end
            tau(:, k, a) = asils.env.torques(q, r, v_rel, B_eci, sun_rel, nu(k), X.P_srp, X.rho, I, G, P.sc.m_res, P.mu, P.env.on);
        end
    end
    Dm = struct('case', P.case.id, 'orbit_period_s', T, 'attitudes', {att});
    Dm.tau_peak = zeros(1, 4); Dm.h_cyclic = zeros(1, 4); Dm.h_secular_orbit = zeros(1, 4); Dm.tau_axis_peak = zeros(3, 4);
    for a = 1:4
        H = cumsum(tau(:, :, a), 2)*dt;                      % body-axis momentum build-up
        trend = H(:, end)*(ts/T);
        Dm.tau_peak(a) = max(sqrt(sum(tau(:, :, a).^2, 1)));
        Dm.tau_axis_peak(:, a) = max(abs(tau(:, :, a)), [], 2);
        Dm.h_cyclic(a) = max(sqrt(sum((H - trend).^2, 1)));
        Dm.h_secular_orbit(a) = norm(H(:, end));
    end
    [Dm.tau_dist, ia] = max(Dm.tau_peak); Dm.worst_attitude = att{ia};
    Dm.h_dist = max(Dm.h_cyclic + 0.25*Dm.h_secular_orbit);   % stored between dumps (dumping runs continuously)
    Dm.h_secular = max(Dm.h_secular_orbit);
    Dm.B_min = min(Bm); Dm.B_mean = mean(Bm); Dm.eclipse_frac = mean(nu < 0.5);
    % detumble
    w0 = v.mission_w0; if isnan(w0), w0 = 10; notes{end+1} = 'mission.w0 blank: 10 deg/s taken'; end
    Dm.w0_deg_s = w0; Dm.J = diag(I)'; Dm.h_detumble = max(diag(I))*w0*pi/180;
    % slew
    th = v.mission_sangle; if isnan(th), th = 30; notes{end+1} = 'mission.sangle blank: 30 deg taken'; end
    Ts = v.req_slew; if isnan(Ts), Ts = 60; notes{end+1} = 'req.slew blank: 60 s taken for the reference slew'; end
    Dm.slew_deg = th; Dm.slew_s = Ts;
    Dm.w_slew = 2*th*pi/180/Ts; Dm.a_slew = 2*pi*th*pi/180/Ts^2;
    Dm.h_slew = max(diag(I))*Dm.w_slew; Dm.tau_slew = max(diag(I))*Dm.a_slew;
    % life and slews for the propellant budget
    Dm.life_yr = v.mission_life; if isnan(Dm.life_yr), Dm.life_yr = 3; notes{end+1} = 'mission.life blank: 3 years taken'; end
    Dm.slews_per_day = v.mission_spd; if isnan(Dm.slews_per_day), Dm.slews_per_day = 0; end
    % margins: momentum x2 (or 1/(1-req.hsat)), torque x1.5
    Dm.k_h = 2; if isfinite(v.req_hsat) && v.req_hsat > 0 && v.req_hsat < 1, Dm.k_h = 1/(1 - v.req_hsat); end
    Dm.k_tau = 1.5;
    Dm.h_req = Dm.k_h*max(Dm.h_dist, Dm.h_slew);
    Dm.tau_req = Dm.k_tau*max(Dm.tau_dist, Dm.tau_slew);
    Dm.req = struct('ape', v.req_ape, 'ake', v.req_ake, 'rks', v.req_rks, 'mass', v.req_mass, 'pavg', v.req_pavg, ...
                    'ppk', v.req_ppk, 'vol', v.req_vol, 'detumble', v.req_detumble, 'sunacq', v.req_sunacq);
    Dm.notes = notes;
end
