function [q_ref, w_ref, wd_ref] = guidance(kind, r, v, t, gd)
%ASILS.FSW.GUIDANCE  Attitude reference q_ref (B/ECI) and rate w_ref (body, rad/s).
%   'nadir'   PORTED from Standard Code guid.nadir (R42 compute_quat_ref_nadir_obc):
%             X_B = -ram, Y_B = nadir, Z_B = -orbit normal; then turned by
%             gd.q_off so the product's payload boresight is the nadir axis.
%   'target'  nadir frame rotated by a fixed roll offset about the along-track
%             axis (off-nadir imaging target) -- gd.roll_deg.
%   'slew'    turn from nadir by gd.roll_deg about gd.axis (default X, roll), a
%             cycloidal profile of duration gd.T_s starting at gd.t0 (zero rate
%             and acceleration at both ends), tracked in the rotating frame.
%   'inertial' fixed gd.q_inertial.
%   'sun'     three-axis Sun referencing: the power face gd.sun_axis (body,
%             default -Z) on the Sun (gd.sun_eci, unit, ECI -- the caller's Sun
%             model), and gd.roll_axis (body, default the payload boresight
%             made normal to the power face) on the orbit normal projected
%             across the Sun line. The Sun line moves ~1 deg/day, so the
%             reference is an inertial hold: w_ref = 0.
%   w_ref: orbit rate of the LVLH frame (r x v / |r|^2) expressed in the reference frame.
    rh = r/norm(r); vh = v/norm(v);
    nrm = asils.util.cross3(vh, -rh); nrm = nrm/norm(nrm);
    ram = asils.util.cross3(-rh, nrm); ram = ram/norm(ram);
    R = [-ram'; -rh'; -nrm'];
    q_nad = asils.quat.fromdcm(R);
    if isfield(gd, 'q_off'), q_nad = asils.quat.mult(q_nad, gd.q_off); R = asils.quat.dcm(q_nad); end
    if isfield(gd, 'flip') && gd.flip                  % yaw flip: 180 deg about the boresight (power face to the Sun)
        u = [1; 0; 0]; if isfield(gd, 'roll_axis'), u = gd.roll_axis(:)/norm(gd.roll_axis); end
        q_nad = asils.quat.mult(q_nad, [u; 0]); R = asils.quat.dcm(q_nad);
    end
    w_orb = asils.util.cross3(r, v)/(r'*r);
    wd_ref = zeros(3,1);                 % reference angular acceleration (slew feedforward)
    ax_ = [1; 0; 0];                     % slew / target offset axis (body): gd.axis, default roll about X
    if isfield(gd, 'axis'), ax_ = gd.axis(:)/norm(gd.axis); end
    switch kind
        case 'nadir'
            q_ref = q_nad; w_ref = R*w_orb;
        case 'target'
            q_ref = asils.quat.mult(q_nad, asils.quat.fromrotvec(ax_*gd.roll_deg*pi/180));
            w_ref = asils.quat.dcm(q_ref)*w_orb;
        case 'slew'
            % roll offset about X_B ramped inside the rotating nadir frame with a
            % cycloidal profile: s(0)=0, s(1)=1, zero rate and acceleration at both ends
            tau = min(1, max(0, (t - gd.t0)/gd.T_s));
            if t > gd.t0 && t < gd.t0 + gd.T_s
                sd = (1 - cos(2*pi*tau))/gd.T_s; sdd = 2*pi*sin(2*pi*tau)/gd.T_s^2;
            else
                sd = 0; sdd = 0;
            end
            s_ = tau - sin(2*pi*tau)/(2*pi);
            ph = gd.roll_deg*pi/180;
            q_ref = asils.quat.mult(q_nad, asils.quat.fromrotvec(ax_*ph*s_));
            wo = asils.quat.dcm(q_ref)*w_orb;
            w_ref = wo + ax_*ph*sd;
            wd_ref = ax_*ph*sdd - asils.util.cross3(ax_*ph*sd, wo);   % transport term: the slew frame turns in the orbit frame
        case 'inertial'
            q_ref = gd.q_inertial; w_ref = zeros(3,1);
        case 'sun'
            q_ref = sun_ref_(gd, nrm); w_ref = zeros(3,1);
        otherwise
            error('asils:fsw:guidance', 'unknown guidance %s', kind);
    end
end

function q = sun_ref_(gd, nrm)
%SUN_REF_  Reference attitude for Sun referencing (see 'sun' above).
    a = [0; 0; -1]; if isfield(gd, 'sun_axis'), a = gd.sun_axis(:)/norm(gd.sun_axis); end
    b = [1; 0; 0]; if isfield(gd, 'roll_axis'), b = gd.roll_axis(:); end
    b = b - (a'*b)*a;
    if norm(b) < 1e-6, b = [0; 1; 0] - a(2)*a; end
    b = b/norm(b);
    s = gd.sun_eci(:)/norm(gd.sun_eci);
    e2 = nrm - (nrm'*s)*s;
    if norm(e2) < 1e-6, e2 = [0; 0; 1] - s(3)*s; end    % Sun on the orbit normal: any roll
    e2 = e2/norm(e2);
    Bm = [a, b, asils.util.cross3(a, b)];
    Em = [s, e2, asils.util.cross3(s, e2)];
    q = asils.quat.fromdcm(Bm*Em');                      % R (ECI -> body): R*s = a, R*e2 = b
end
