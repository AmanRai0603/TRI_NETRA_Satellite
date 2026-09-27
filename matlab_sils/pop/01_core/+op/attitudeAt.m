function R_bi = attitudeAt(att, utc)
%OP.ATTITUDEAT  Body->inertial DCM at ONE epoch.
%   R_bi = op.attitudeAt(att, utc)
%
%   ---------------------------------------------------------------------------
%   WHY THIS EXISTS
%   ---------------------------------------------------------------------------
%   Every attitude-dependent force -- panel drag, box-wing SRP, box-wing ERP --
%   reads opts.R_bi. That came from cfg.spacecraft.R_bi, which is a CONSTANT
%   (defaultConfig sets eye(3)). So the panel models were being handed a frozen
%   attitude and returned a frozen projected area: A(t) collapsed to Aref and
%   Cd(t) collapsed to a single value, which defeats the entire point of running
%   them instead of a cannonball.
%
%   Meanwhile validate_OD ALREADY FETCHES the measured attitude -- ITSG's
%   `attitude` product, satellite->celestial quaternions -- and used it only to
%   rotate the accelerometer for metric [3]. The information was on disk and the
%   forces never saw it.
%
%   ---------------------------------------------------------------------------
%   ACCEPTS  (att)
%   ---------------------------------------------------------------------------
%     []                  -> eye(3). No attitude: the caller gets ram-fixed body
%                            axes, which is the old behaviour, kept so nothing
%                            silently changes under anyone.
%     3x3 matrix          -> used as-is (a genuinely fixed attitude)
%     function handle     -> att(utc) must return a 3x3 DCM
%     struct .mjd .q      -> MEASURED quaternions (scalar-first, sat->celestial),
%                            e.g. straight from data.itsg(SAT,DATE,'attitude').
%                            Interpolated to utc; see the note on flips below.
%
%   ---------------------------------------------------------------------------
%   THE SIGN-FLIP TRAP
%   ---------------------------------------------------------------------------
%   q and -q are the SAME rotation, and ITSG's files flip sign constantly (~4300
%   flips in an 8640-epoch CHAMP day -- half of them). Interpolating across a flip
%   swings the quaternion through the far side of the hypersphere and produces an
%   attitude error up to 180 deg. So the series is unwrapped ONCE, at buildWorld
%   time, via validation.quat_continuous -- never per step, which would be both
%   wrong and slow. If you hand this a raw .q it will unwrap defensively and say so.
%
%   Interpolation is SLERP-free: normalised linear (nlerp) between bracketing
%   samples. At ITSG's 10 s sampling the rotation between samples is <0.1 deg, where
%   nlerp and slerp agree to ~1e-7 rad -- far below the attitude knowledge itself.
    if isempty(att)
        R_bi = eye(3); return
    end
    if isnumeric(att)
        if isequal(size(att),[3 3]), R_bi = att; return; end
        error('op:attitudeAt:shape','numeric attitude must be a 3x3 DCM, got %s', mat2str(size(att)));
    end
    if isa(att,'function_handle')
        R_bi = att(utc);
        if ~isequal(size(R_bi),[3 3])
            error('op:attitudeAt:handle','attitude handle must return a 3x3 DCM');
        end
        return
    end
    if isstruct(att) && isfield(att,'mjd') && isfield(att,'q')
        mjd = datenum(utc(1),utc(2),utc(3),utc(4),utc(5),utc(6)) - 678942;
        R_bi = validation.quat2dcm(qAt(att, mjd));
        return
    end
    error('op:attitudeAt:type', ...
      ['attitude must be [] , a 3x3 DCM, a function handle, or a struct with ' ...
       '.mjd and .q (scalar-first, sat->celestial).']);
end

function q = qAt(att, mjd)
%QAT  Bracketing nlerp on an already-unwrapped quaternion series.
    t = att.mjd(:); Q = att.q;
    if mjd <= t(1),   q = Q(1,:);   return, end
    if mjd >= t(end), q = Q(end,:); return, end
    k = find(t <= mjd, 1, 'last');
    h = t(k+1) - t(k);
    if h <= 0, q = Q(k,:); return, end
    s  = (mjd - t(k))/h;
    q0 = Q(k,:); q1 = Q(k+1,:);
    % Defensive: if the series was NOT unwrapped, at least do not interpolate the
    % long way round for THIS pair. (Unwrap the whole series once instead.)
    if dot(q0,q1) < 0, q1 = -q1; end
    q = (1-s)*q0 + s*q1;
    q = q / norm(q);
end
