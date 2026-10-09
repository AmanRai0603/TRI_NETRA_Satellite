function b = le32(c)
%ASILS.HAL.LE32  Whole numbers as int32, little-endian bytes, one row a number (the device emulators' framing).
%   Owner: Agastya. Copyright (c) 2026 Agastya. All rights reserved.
    c = c(:); c(c < 0) = c(c < 0) + 4294967296;
    b = [mod(c, 256), mod(floor(c/256), 256), mod(floor(c/65536), 256), floor(c/16777216)];
end
