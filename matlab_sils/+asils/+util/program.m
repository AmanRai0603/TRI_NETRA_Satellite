function exe = program(name)
%ASILS.UTIL.PROGRAM  Where one of the library's programs is (adcs, the engine; tndb, the design library), first found of:
%   $ADCS_BIN / $TNDB_BIN; bin/<name> beside the twin (an install's bundled programs); engine/target/release/<name> in
%   the repository the twin sits in (python3 tools/engine.py build makes them). '' when none is there.
%   Owner: Agastya. Copyright (c) 2026 Agastya. All rights reserved.
    exe = getenv([upper(name) '_BIN']);
    if ~isempty(exe), return, end
    ext = ''; if ispc, ext = '.exe'; end
    R = asils.util.root();
    for c = {fullfile(R, 'bin', [name ext]), fullfile(fileparts(R), 'engine', 'target', 'release', [name ext])}
        if exist(c{1}, 'file') == 2, exe = c{1}; return, end
    end
    exe = '';
end
