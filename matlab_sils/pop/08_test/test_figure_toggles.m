function nfail = test_figure_toggles(verbose)
%TEST_FIGURE_TOGGLES  Every figure show_OD draws can be switched off. By RUNNING it.
%
%   The previous version of this check read show_OD's SOURCE and compared the
%   defaults struct against `tags{end+1} = 'literal'` lines. It broke the moment a
%   tag came from a variable (the density loop emits tags{end+1} = tg), and it
%   counted sampling KNOBS as figure toggles because both live in one struct and both
%   have numeric defaults. It was measuring the text, not the behaviour.
%
%   So: build a fixture, call show_OD with everything on, count the figures. Then
%   switch each toggle off in turn and check the count drops by exactly one. That
%   cannot be fooled by formatting, and it fails for the right reason -- a toggle
%   that does not suppress its figure.
    if nargin < 1, verbose = true; end
    nfail = 0;
    R = fixture();
    sol = R.sol; R = rmfield(R, 'sol');

    figs = show_OD(R, sol, 'toggle test', '');
    n0   = numel(figs); close all
    if verbose, printf('    all on: %d figures\n', n0); end

    % the tags of the figures that were ACTUALLY drawn -- not every toggle, because a
    % toggle whose figure needs data the fixture does not have (tracks) cannot
    % suppress anything, and demanding that it does would be testing the fixture.
    [~, drawn] = show_OD(R, sol, 'toggle test', '');
    close all
    nt = 0;
    for i = 1:numel(drawn)
        t = drawn{i};
        Q = struct(t, 0);
        f2 = show_OD(R, sol, 'toggle test', '', Q);
        n2 = numel(f2); close all
        nt = nt + 1;
        if n2 ~= n0 - 1
            printf('    FAIL: P.%s = 0 gave %d figures, expected %d\n', t, n2, n0-1);
            nfail = nfail + 1;
        end
    end
    if verbose && nfail == 0
        printf('    %d toggles, each suppresses exactly its own figure   PASS\n', nt);
    end
end

function R = fixture()
    K = de440.constants(); n = 200; t = (0:n-1).'*30;
    a = K.Re_earth + 480e3; om = sqrt(K.mu_earth/a^3);
    rr = [a*cos(om*t), a*sin(om*t), 0.05*a*sin(0.5*om*t)];
    vv = [-7650*sin(om*t), 7650*cos(om*t), 100*cos(0.5*om*t)];
    xT = 4.6*(t/t(end)).^2;
    dr = [0.5*sin(om*t)-0.17, xT, 0.7*sin(0.7*om*t)];
    dv = [-om*xT*0.5, 0.001*ones(n,1), 0.0016*sin(om*t)];
    sig = 2e-8*[cos(om*t), 0.4*sin(om*t)+0.5, 0.3*cos(2*om*t)];
    bias = repmat([-0.3e-8 2.5e-8 0.05e-8], n, 1);
    mod_ = 0.53*sig;
    rho = 1e-13*(1+0.8*sin(om*t));

    R = struct();
    R.rdo = struct('t',t,'r_our',rr,'r_ref',rr-dr,'v_our',vv,'v_ref',vv-dv, ...
        'radial',0.371,'along',2.091,'cross',0.681,'pos3D',2.23, ...
        'rad',dr(:,1),'alo',dr(:,2),'cro',dr(:,3),'rmag',a*ones(n,1), ...
        'mean_rad',-0.17,'mean_alo',1.67,'mean_cro',0.09, ...
        'std_rad',0.33,'std_alo',1.26,'std_cro',0.68, ...
        'dv',0.0024,'dv_vec',dv,'dv_rtn',dv,'dv_mag',sqrt(sum(dv.^2,2)));
    tk = t(1:3:end);
    R.kin = struct('t',tk,'r_ref',rr(1:3:end,:)-dr(1:3:end,:)*1.1,'r_our',rr(1:3:end,:), ...
        'v_our',vv(1:3:end,:),'radial',0.36,'along',2.0,'cross',0.66,'pos3D',2.2, ...
        'rad',dr(1:3:end,1),'alo',dr(1:3:end,2),'cro',dr(1:3:end,3), ...
        'mean_rad',-0.16,'mean_alo',1.6,'mean_cro',0.09, ...
        'std_rad',0.33,'std_alo',1.26,'std_cro',0.68,'rmag',a*ones(numel(tk),1));
    R.acc = struct('t',t,'meas',sig+bias,'mod',mod_, ...
        'rms_meas',rms3(sig+bias),'rms_mod',rms3(mod_),'rms_diff',rms3(mod_-sig-bias), ...
        'rms_ac_meas',rms3(sig-mean(sig)),'rms_ac_mod',rms3(mod_-mean(mod_)), ...
        'bias_meas',mean(sig+bias,1),'bias_mod',mean(mod_,1), ...
        'ac_meas',sig+bias-mean(sig+bias,1),'ac_mod',mod_-mean(mod_,1), ...
        'ratio',0.529,'ratio_raw',0.447,'movavg_win',180, ...
        'cmp',struct('drag',0.6*mod_,'srp',0.4*mod_,'erp',0.05*mod_));
    R.den = struct('t',t,'meas',rho*0.95,'mod',1.05*rho,'ratio',0.94, ...
                   'rms_logerr',0.20,'model','dtm2020');
    R.tud = struct('t',t,'meas',rho,'mod',1.14*rho,'ratio',1.14, ...
                   'rms_logerr',0.24,'model','dtm2020');
    R.refSpread = 0.01;
    R.closure = validation.closure(R, false);
    R.sol = struct('t',t,'r',rr,'v',vv);
end

function v = rms3(x), v = sqrt(mean(sum(x.^2,2))); end
