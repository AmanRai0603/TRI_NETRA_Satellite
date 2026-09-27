function sol = propagate(cfg)
%OP.PROPAGATE  Precise Cowell orbit propagator -- main entry point.
%
%   sol = op.propagate(cfg)
%
%   Propagates an initial ECI state from a UTC epoch under a configurable stack
%   of force models, using a selectable integrator, and returns the trajectory
%   plus a callable state interpolator.
%
%   REQUIRED cfg fields
%     .epoch      [Y Mo D H Mi S]  UTC epoch of the initial state
%     .r0         3x1 ECI position [m]      (GCRF/J2000)
%     .v0         3x1 ECI velocity [m/s]
%     .tspan      either total duration [s] (scalar) or [t0 tf] [s]
%     .spacecraft struct: .mass .Aref (+ .facets/.optics for panel models,
%                 .Cd .Cr as needed, .R_bi attitude DCM if attitude-dependent)
%     .forces     struct of force toggles/models (see config.defaultConfig)
%     .integrator struct: .method ('rk78'|'rk45'|'rk4'|'rk6luther'|'nystrom4'|
%                 'gaussJackson8'), .h (fixed) or .rtol/.atol (adaptive)
%   OPTIONAL
%     .gravityField .field ('default' | path to .gfc), .degree
%     .frame        .build ('gmst'|'A'|'B'|'C'), .dUT1
%     .output       .times (explicit report times [s]) or .dt (uniform step)
%
%   RETURNS sol
%     .t  (Kx1) times since epoch [s]        .utc (Kx6) UTC per sample
%     .r  (Kx3) ECI position [m]             .v (Kx3) ECI velocity [m/s]
%     .stateAt   @(t) -> [r;v] at arbitrary time(s) since epoch (Hermite)
%     .W, .cfg, .integrator meta, .raw (integrator nodes)
%
%   EXAMPLE
%     cfg = config.defaultConfig();
%     cfg.epoch=[2025 6 1 0 0 0]; cfg.r0=[6778137;0;0]; cfg.v0=[0;7668;0];
%     cfg.tspan=6000; sol=op.propagate(cfg);
%     rv = sol.stateAt(1234.5);      % state 1234.5 s after epoch
%
%   See also OP.ACCEL, INTEG.RUN, CONFIG.DEFAULTCONFIG.

    % ---- validate required inputs (clear errors instead of cryptic ones) ----
    req = {'epoch','r0','v0','tspan'};
    for i=1:numel(req)
        if ~isfield(cfg,req{i}) || isempty(cfg.(req{i}))
            error('op:propagate:input','cfg.%s is required and must be non-empty', req{i});
        end
    end
    if numel(cfg.r0)~=3 || numel(cfg.v0)~=3
        error('op:propagate:input','cfg.r0 and cfg.v0 must each have 3 elements (got %d, %d)', numel(cfg.r0), numel(cfg.v0));
    end
    if ~all(isfinite(cfg.r0(:))) || ~all(isfinite(cfg.v0(:)))
        error('op:propagate:input','cfg.r0/cfg.v0 contain non-finite values');
    end
    if norm(cfg.r0) < 6.3e6
        warning('op:propagate:input','|r0| = %.0f m is below the Earth surface -- check units (r0 must be metres, ECI).', norm(cfg.r0));
    end

    % ---- normalise time span ----
    if isscalar(cfg.tspan), t0=0; tf=cfg.tspan; else, t0=cfg.tspan(1); tf=cfg.tspan(2); end
    if ~(tf>t0), error('op:propagate:input','tspan must give tf>t0 (got t0=%.3f, tf=%.3f)', t0, tf); end

    W = op.buildWorld(cfg);
    y0 = [cfg.r0(:); cfg.v0(:)];
    f  = @(t,y) op.rhs(t,y,W);

    I = getf(cfg,'integrator',struct('method','rk78'));
    method = getf(I,'method','rk78');
    opts = I;                                  % pass .h / .rtol / .atol through

    % ---- output sampling ----
    tout = [];
    if isfield(cfg,'output')
        if isfield(cfg.output,'times')&&~isempty(cfg.output.times)
            tout = cfg.output.times(:);
        elseif isfield(cfg.output,'dt')&&~isempty(cfg.output.dt)
            tout = (t0:cfg.output.dt:tf).';
        end
    end

    % --- dense-output guard -------------------------------------------------
    % The adaptive high-order integrators are accurate AT THEIR NODES, but the
    % dense output between nodes is a CUBIC Hermite: its error grows like h^4, so
    % a long step (rk78 at a tight rtol happily takes 300-600 s on a smooth orbit)
    % gives TENS OF METRES at the requested output times even though the nodes are
    % near machine precision. Verified on a two-body orbit with an analytic truth:
    %   no cap -> 27.3 m max ; hmax=120 s -> 5.9 m ; hmax=30 s -> 0.13 m.
    % So cap the step at the output spacing (never coarser than the output grid).
    if ~isempty(tout) && numel(tout)>1
        dtOut = min(diff(sort(tout(:))));
        if dtOut > 0
            if ~isfield(opts,'hmax') || isempty(opts.hmax), opts.hmax = dtOut;
            else,                                           opts.hmax = min(opts.hmax, dtOut); end
        end
    end
    tic; raw = integ.run(method, f, t0, tf, y0, opts, tout); wall=toc;

    sol.t = raw.t; sol.r = raw.r; sol.v = raw.v; sol.y=raw.y;
    sol.utc = zeros(numel(sol.t),6);
    for k=1:numel(sol.t), sol.utc(k,:)=op.addsec(W.epoch, sol.t(k)); end
    R=raw.raw;                                  % integrator nodes for dense output
    sol.stateAt = @(tq) integ.hermite(R.T, R.Y, R.D, tq(:)).';   % [r;v] column (6xM)
    sol.W=W; sol.cfg=cfg; sol.method=method; sol.nodes=raw.nodes; sol.walltime=wall;
    sol.raw=R;
end
function val=getf(s,f,d), if isfield(s,f)&&~isempty(s.(f)), val=s.(f); else, val=d; end, end
