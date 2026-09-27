function D = init(P)
%ASILS.DEVICES.INIT  Draw each fitted part's constant errors once per run.
%   Bias, scale factor and axis misalignment come from the part's [dispersion]
%   table (catalogue/parts/*.toml -> data/parts/*.json). In a nominal run the
%   draws are still made (seeded), so a Monte Carlo run k differs from the
%   nominal only by its seed and its dispersed case/scenario values.
    dv = P.dev; D = struct();
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
    end
    if dv.sun.fitted
        s = dv.sun; D.sun.n = s.normals;
        D.sun.bias = s.bias_sigma*randn(3, size(s.normals,2));
    end
    if dv.st.fitted
        s = dv.st;
        for h = 1:size(s.boresight, 2)
            D.st.q_bias(:,h) = asils.quat.fromrotvec(s.bias_sigma*randn(3,1));
            D.st.q_mis(:,h) = asils.quat.fromrotvec(s.misalign_sigma*randn(3,1));
        end
        D.st.hist_q = []; D.st.hist_t = [];
    end
    if dv.mtq.fitted
        t = dv.mtq; n = size(t.axes, 2);
        D.mtq.scale = 1 + t.scale_sigma*randn(1, n);
        A = t.axes;
        for j = 1:n, A(:,j) = ma(t.misalign_rad) * A(:,j); end
        D.mtq.A = A;
    end
    if dv.rw.fitted
        w = dv.rw; n = size(w.axes, 2);
        D.rw.tscale = 1 + w.torque_scale_sigma*randn(1, n);
        D.rw.fscale = w.friction_scale_lo + (w.friction_scale_hi - w.friction_scale_lo)*rand(1, n);
        A = w.axes;
        for j = 1:n, A(:,j) = ma(w.misalign_rad) * A(:,j); end
        D.rw.A = A;
    end
end
