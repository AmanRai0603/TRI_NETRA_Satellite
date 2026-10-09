function bus = drv_write(bus, p, m_body, cmd_r, cmd_g, duty)
%ASILS.FSW.DRV_WRITE  The command words on the buses: the runtime of fsw-rs/src/drv.rs (write) in MATLAB. The words
%   are the design's (asils.alg.drivers.drv_write: the coils' PWM, the rotor and gimbal words, the valves' on-times);
%   here only their frames: PWM channels 0 to 2, CAN 0x100 + i (rotor), 0x140 + j (gimbal), 0x300 (valves).
%   Owner: Agastya. Copyright (c) 2026 Agastya. All rights reserved.
    nc = min(p.nc, 6);
    [pwm, rot, gim, valves] = asils.alg.drivers.drv_write(m_body, p.m_max, cmd_r, p.rot_tmax, p.nr, cmd_g, p.gim_rate_max, ...
                                                           p.ng, duty, nc, p.dt);
    bus.pwm(1:3) = pwm(1:3);
    f = zeros(p.nr + p.ng + (p.nc > 0), 10); k = 0;
    for i = 1:p.nr
        k = k + 1; f(k, 1:4) = [256 + i - 1, 2, le16_(rot(i))];
    end
    for i = 1:p.ng
        k = k + 1; f(k, 1:4) = [320 + i - 1, 2, le16_(gim(i))];
    end
    if p.nc > 0
        k = k + 1; f(k, 1:2) = [768, 8]; f(k, 3:2 + nc) = valves(1:nc);
    end
    bus.can_tx = [bus.can_tx; f];
end

function b = le16_(w)
% a word as int16, little-endian
    if w < 0, w = w + 65536; end
    b = [mod(w, 256), floor(w/256)];
end
