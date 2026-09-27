function idx = window(rec, spec)
%ASILS.METRICS.WINDOW  Sample indices of a metric window.
%   'all' | 'last_orbit' | 'last_half_orbit' | 'after_s:<seconds>' | 'pointing' (modes > detumble)
    t = rec.t; T = rec.P.orbit.period_s;
    switch spec
        case 'all', idx = 1:numel(t);
        case 'last_orbit', idx = find(t >= t(end) - T);
        case 'last_half_orbit', idx = find(t >= t(end) - T/2);
        case 'pointing', idx = find(rec.mode > 1);
        otherwise
            if strncmp(spec, 'after_s:', 8)
                idx = find(t >= str2double(spec(9:end)));
            else
                error('asils:metrics:window', 'unknown window %s', spec);
            end
    end
end
