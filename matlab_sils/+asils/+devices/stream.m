function s = stream(seed, name)
%ASILS.DEVICES.STREAM  The run's random stream of a name: the language's stream (asils.pc.stream_new, SplitMix64 over a
%   counter) of the seed and the name's FNV-1a 64-bit hash, as the engine's Rng::new(seed, stream_id(name)) (adcs-sim-core
%   rng.rs, the engine's core): the same seed and name give the twin and the engine the same draws, value for value.
%   Owner: Agastya. Copyright (c) 2026 Agastya. All rights reserved.
    h = [3421674724, 2216829733];                 % 0xcbf29ce4 84222325
    for c = double(name)
        h = [h(1), bitxor(h(2), c)];
        h = asils.pc.u64_mul_(h, [256, 435]);     % 0x100 000001b3
    end
    s = asils.pc.stream_new(seed, h);
end
