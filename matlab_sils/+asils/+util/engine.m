function exe = engine()
%ASILS.UTIL.ENGINE  The engine's command (adcs), which the twin asks for the flight software's blob (`adcs params`):
%   $ADCS_BIN when set, else engine/target/release/adcs beside the twin (python3 tools/engine.py build makes it).
%   Owner: Agastya. Copyright (c) 2026 Agastya. All rights reserved.
    exe = getenv('ADCS_BIN');
    if isempty(exe)
        exe = fullfile(fileparts(asils.util.root()), 'engine', 'target', 'release', 'adcs');
        if ispc, exe = [exe '.exe']; end
    end
    if exist(exe, 'file') ~= 2
        error('asils:engine:missing', ['the twin flies the flight software''s blob the engine builds (adcs params), and the engine ' ...
            'is not built (%s): python3 tools/engine.py build, or set ADCS_BIN'], exe);
    end
end
