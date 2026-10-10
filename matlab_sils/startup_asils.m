function startup_asils()
%STARTUP_ASILS  Put the TRI-NETRA ADCS SILS (asils) on the path. Run once per
%   MATLAB/Octave session. The vendored Precision Orbit Propagator (pop/) is
%   not: no run calls it (the twin's precision orbit flies env's generated
%   models); it is the referent the tests put on the path themselves.
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
    fprintf('TRI-NETRA ADCS SILS %s ready (root: %s)\n', asils.version(), here);
end
