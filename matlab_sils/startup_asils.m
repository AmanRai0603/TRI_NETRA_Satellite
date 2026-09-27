function startup_asils()
%STARTUP_ASILS  Put the TRI-NETRA ADCS SILS (asils) and the Precision Orbit
%   Propagator (pop/) on the path. Run once per MATLAB/Octave session.
%
%     >> startup_asils
%     >> rec = asils.run('nadir_hold_ais', 'cases/ais_3u.csv');
%
%   Copyright (c) 2026 Agastya. All rights reserved. See NOTICE.md.
    here = fileparts(mfilename('fullpath'));
    if exist('OCTAVE_VERSION', 'builtin')
        warning('off', 'Octave:shadowed-function');
        warning('off', 'Octave:language-extension');
    end
    addpath(here);
    % The POP organises itself into numbered segments; its own setup adds them.
    popRoot = fullfile(here, 'pop');
    old = pwd; cleaner = onCleanup(@() cd(old)); %#ok<NASGU>
    cd(popRoot);
    evalc('setup_paths();');          % silence its banner
    cd(old);
    fprintf('TRI-NETRA ADCS SILS %s ready (root: %s)\n', asils.version(), here);
end
