function F = init(P, jd0)
%ASILS.FSW.INIT  Flight-software state at power-on (threaded through every tick,
%   no persistent/global -- the Standard Code rule).
    F.P = P.fsw; F.jd0 = jd0;
    F.mode = F.P.start_mode; F.t_mode = 0; F.hold = 0;
    F.K = []; F.ad_ok = false; F.t_st = -1e9;
    F.r = []; F.v = []; F.t_fix = -1;
    F.b1 = []; F.bsum = zeros(3,1); F.bn = 0; F.m_body = zeros(3,1); F.m_hold = zeros(3,1);
    F.tau_w = zeros(0, 1);
    if P.dev.rw.fitted, F.tau_w = zeros(size(P.dev.rw.axes, 2), 1); end
    F.I_q = zeros(3,1); F.q_ref = nan(4,1); F.w_ref = nan(3,1);
    F.gd = F.P.guidance; F.gh = []; F.Bref = []; F.t_Bref = -1e9;
    F.last_ctrl = -1e9; F.tau_req = zeros(3,1); F.B_dump = zeros(3,1); F.m_dump = zeros(3,1);
    F.w_est = zeros(3,1);
    F.log = struct('t', {}, 'mode', {});
    F.log(1).t = 0; F.log(1).mode = F.mode;
end
