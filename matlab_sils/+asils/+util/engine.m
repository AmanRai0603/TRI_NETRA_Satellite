function exe = engine()
%ASILS.UTIL.ENGINE  The engine's command (adcs), which the twin asks for the flight software's blob (`adcs params`):
%   $ADCS_BIN when set, else bin/adcs beside the twin (an install's), else engine/target/release/adcs in the repository
%   (python3 tools/engine.py build makes it): asils.util.program.
%   Owner: Agastya. Copyright (c) 2026 Agastya. All rights reserved.
    exe = asils.util.program('adcs');
    if isempty(exe) || exist(exe, 'file') ~= 2
        if isempty(exe), exe = fullfile(fileparts(asils.util.root()), 'engine', 'target', 'release', 'adcs'); end
        error('asils:engine:missing', ['the twin flies the flight software''s blob the engine builds (adcs params), and the engine ' ...
            'is not built (%s): python3 tools/engine.py build, or set ADCS_BIN'], exe);
    end
end
