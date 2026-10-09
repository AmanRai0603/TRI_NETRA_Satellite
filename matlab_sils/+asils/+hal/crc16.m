function c = crc16(p)
%ASILS.HAL.CRC16  CRC-16/CCITT (init 0xFFFF, polynomial 0x1021) of the bytes p: the device emulators' framing (the
%   engine's emu.rs crc16, the rig's), by its table.
%   Owner: Agastya. Copyright (c) 2026 Agastya. All rights reserved.
    persistent T
    if isempty(T)
        T = zeros(256, 1);
        for n = 0:255
            c = n*256;
            for k = 1:8
                if c >= 32768, c = bitxor(mod(c*2, 65536), 4129); else, c = mod(c*2, 65536); end
            end
            T(n + 1) = c;
        end
    end
    c = 65535;
    for k = 1:numel(p)
        c = bitxor(mod(c*256, 65536), T(bitxor(floor(c/256), p(k)) + 1));
    end
end
