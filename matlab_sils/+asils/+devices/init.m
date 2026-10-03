function D = init(P)
%ASILS.DEVICES.INIT  Draw each fitted part's constant errors once per run.
%   Bias, scale factor and axis misalignment come from the part's [dispersion]
%   table (catalogue/parts/*.toml -> data/parts/*.json). In a nominal run the
%   draws are still made (seeded), so a Monte Carlo run k differs from the
%   nominal only by its seed and its dispersed case/scenario values.
    dv = P.dev; D = struct('gps_dead', false);
    D.gpsh = struct('t', zeros(1,0), 'r', zeros(3,0), 'v', zeros(3,0));   % truth history for the GNSS latency
    ma = @(s) asils.quat.dcm(asils.quat.fromrotvec(s*randn(3,1)));   % small misalignment DCM
    if dv.gyro.fitted
        g = dv.gyro;
        D.gyro.M = ma(g.misalign_rad) * diag(1 + g.sf_sigma*randn(3,1));
        D.gyro.b = g.bias_sigma*randn(3,1);        % turn-on bias [rad/s]
        D.gyro.brw = zeros(3,1);                   % rate-random-walk state
    end
    if dv.mag.fitted
        m = dv.mag;
        D.mag.M = ma(m.misalign_rad) * diag(1 + m.sf_sigma*randn(3,1));
        D.mag.b = m.bias_T*ones(3,1)/sqrt(3) + m.bias_sigma*randn(3,1);
        D.mag.dead = false;
    end
    if dv.sun.fitted
        s = dv.sun; D.sun.n = s.normals;
        D.sun.bias = s.bias_sigma*randn(3, size(s.normals,2));
    end
    if dv.st.fitted
        s = dv.st;
        nh = size(s.boresight, 2);
        for h = 1:nh
            D.st.q_bias(:,h) = asils.quat.fromrotvec(s.bias_sigma*randn(3,1));
            D.st.q_mis(:,h) = asils.quat.fromrotvec(s.misalign_sigma*randn(3,1));
        end
        D.st.hist_q = []; D.st.hist_t = []; D.st.dead = false(1, nh);
        D.st.blind_until = -Inf(1, nh);        % blind after the Sun / Moon left the exclusion cone
        if any(strcmp(s.model, {'quest', 'image'})), D.st.cat = asils.devices.star_catalogue(4000); end
        if strcmp(s.model, 'image')          % component level: camera + onboard pair table
            D.st.cam = asils.comp.star_tracker.camera(s.fov, s.camera);   % every value from the part
            D.st.K = asils.comp.star_tracker.pairs(D.st.cat, s.fov);
        end
    end
    if dv.mtq.fitted
        t = dv.mtq; n = size(t.axes, 2);
        D.mtq.scale = 1 + t.scale_sigma*randn(1, n);
        A = t.axes;
        for j = 1:n, A(:,j) = ma(t.misalign_rad) * A(:,j); end
        D.mtq.A = A; D.mtq.dead = false(1, n);
        D.mtq.m = zeros(1, n); D.mtq.mc = zeros(1, n);   % coil dipoles now, and the last command (RL lag)
    end
    if dv.es.fitted
        D.es.bias = asils.quat.fromrotvec(dv.es.bias_sigma*randn(3,1));
    end
    if dv.css.fitted
        c = dv.css; k = size(c.normals, 2);
        D.css.scale = 1 + c.scale_sigma*randn(1, k); D.css.dead = false(1, k);
        D.css.R = ma(c.misalign_rad);
    end
    if dv.mex.fitted
        w = dv.mex; n = numel(w.kind);
        D.mex.tscale = 1 + w.torque_scale_sigma.*randn(1, n);
        D.mex.fscale = w.friction_scale_lo + (w.friction_scale_hi - w.friction_scale_lo).*rand(1, n);
        D.mex.eta = w.eta_lo + (w.eta_hi - w.eta_lo).*rand(1, n);      % fmr pump efficiency
        A = w.A0;
        for j = 1:n, A(:,j) = ma(w.misalign_rad(j)) * A(:,j); end
        D.mex.A0 = A; D.mex.failed = false(1, n); D.mex.gfailed = false(1, size(w.G, 2));
        D.mex.htgt = zeros(1, n); D.mex.hf = zeros(1, n);          % fluid-ring driver state
    end
    if dv.rcs.fitted
        r = dv.rcs; nc = size(r.tau_couple, 2);
        D.rcs.tscale = 1 + r.thrust_sigma*randn(1, nc);
        T = r.tau_couple;
        for j = 1:nc, T(:,j) = ma(r.misalign_rad) * T(:,j); end
        D.rcs.tau_couple = T; D.rcs.failed = false(1, nc);
        D.rcs.isp = r.isp_lo + (r.isp_hi - r.isp_lo)*rand;        % N2O: 60-80 s
    end
end
