function varargout = link(op, varargin)
%ASILS.HAL.LINK  The OILS/HILS link's two frames (the rig's layout, little-endian):
%   sensors (twin -> OBC):  u64 now_ns | u8 present (1 mag, 2 gyro, 4 Sun, 8 Earth) | mag 7 | gyro 13 | Sun 7 |
%                           Earth 7 | u16 n, UART port 1 bytes | u16 n, UART port 2 bytes | u8 frames, each u32 id,
%                           u8 dlc, 8 data
%   commands (OBC -> twin): i16 PWM x 8 | u8 frames, each u32 id, u8 dlc, 8 data
%   b = asils.hal.link('pack_sensors', bus)        bus = asils.hal.link('unpack_sensors', b, bus)
%   b = asils.hal.link('pack_commands', bus)       bus = asils.hal.link('unpack_commands', b, bus)
%   Owner: Agastya. Copyright (c) 2026 Agastya. All rights reserved.
    switch op
        case 'pack_sensors'
            B = varargin{1};
            pres = ~isempty(B.mag) + 2*~isempty(B.gyro) + 4*~isempty(B.sun) + 8*~isempty(B.es);
            blk = @(x, n) [reshape(x, [], 1); zeros(n - numel(x), 1)];
            u1 = B.uart{2}(:); u2 = B.uart{3}(:);
            t = B.now_ns; ns = zeros(8, 1);
            for k = 1:8, ns(k) = mod(t, 256); t = floor(t/256); end
            b = [ns; pres; blk(B.mag, 7); blk(B.gyro, 13); blk(B.sun, 7); blk(B.es, 7); ...
                 mod(numel(u1), 256); floor(numel(u1)/256); u1; mod(numel(u2), 256); floor(numel(u2)/256); u2; can_(B.can_rx)];
            varargout{1} = uint8(b);
        case 'unpack_sensors'
            b = double(varargin{1}(:)); B = varargin{2};
            B.now_ns = sum(b(1:8).*256.^(0:7)'); pres = b(9); i = 10;
            [B.mag, i] = take_(b, i, 7, bitand(pres, 1)); [B.gyro, i] = take_(b, i, 13, bitand(pres, 2));
            [B.sun, i] = take_(b, i, 7, bitand(pres, 4)); [B.es, i] = take_(b, i, 7, bitand(pres, 8));
            n = b(i) + 256*b(i + 1); B.uart{2} = b(i + 2:i + 1 + n).'; i = i + 2 + n;
            n = b(i) + 256*b(i + 1); B.uart{3} = b(i + 2:i + 1 + n).'; i = i + 2 + n;
            [B.can_rx, ~] = uncan_(b, i);
            varargout{1} = B;
        case 'pack_commands'
            B = varargin{1};
            w = B.pwm(:); w(w < 0) = w(w < 0) + 65536;
            varargout{1} = uint8([reshape([mod(w, 256), floor(w/256)].', [], 1); can_(B.can_tx)]);
        case 'unpack_commands'
            b = double(varargin{1}(:)); B = varargin{2};
            w = b(1:2:16) + 256*b(2:2:16); w(w >= 32768) = w(w >= 32768) - 65536;
            B.pwm = w; [B.can_tx, ~] = uncan_(b, 17);
            varargout{1} = B;
        otherwise
            error('asils:hal:link', 'unknown link operation %s', op);
    end
end

function b = can_(F)
    n = size(F, 1);
    b = zeros(1 + 13*n, 1); b(1) = n;
    for k = 1:n
        x = asils.hal.le32(F(k, 1));
        b(2 + 13*(k - 1):1 + 13*k) = [x(:); F(k, 2); F(k, 3:10).'];
    end
end

function [F, i] = uncan_(b, i)
    n = b(i); i = i + 1; F = zeros(n, 10);
    for k = 1:n
        F(k, :) = [b(i) + 256*b(i + 1) + 65536*b(i + 2) + 16777216*b(i + 3), b(i + 4), b(i + 5:i + 12).'];
        i = i + 13;
    end
end

function [x, i] = take_(b, i, n, present)
    if present, x = b(i:i + n - 1); else, x = []; end
    i = i + n;
end
