function setup_paths()
%SETUP_PATHS  Put the Precision Orbit Propagator on the path (run once/session).
%   >> setup_paths
%   The toolbox is organised into numbered segments; each segment folder is added
%   so the +packages inside it resolve. Package (+xxx) folders are exposed by
%   adding their PARENT segment, so they are deliberately not added individually.
%
%   Segments:
%     01_core         engine        (+op +config +grav +integ)
%     02_forces       force models  (+forces +drag +srp +thirdbody +erp
%                                     +relativity +solidtides +oceantides +tideutil)
%     03_frames_time  frames + time (+frames, ephemeris = de440/timeconv/ephemInputs)
%     04_atmosphere   density       (+atmos + DTM2020/JB2008 model code + data)
%     05_data         fetch+cache   (+data +sat data_sources sat_data)
%     06_validation   validation    (+validation, realsat, GPS/POD tools)
%     07_examples     demos         08_test  regressions     09_docs  documentation
    here = fileparts(mfilename('fullpath'));
    A = @(varargin) addpath(fullfile(here, varargin{:}));

    A('01_core');
    A('02_forces');
    A('03_frames_time');
    A('03_frames_time/tidal_eop_models');   % bare tidal-EOP helpers used by eci2ecef_A/B/C
    A(fullfile('03_frames_time','ephemeris'));      % de440, timeconv, ephemInputs
    A('04_atmosphere');
    A(fullfile('04_atmosphere','density_models','dtm2020','operational'));
    A(fullfile('04_atmosphere','density_models','dtm2020','research'));
    A(fullfile('04_atmosphere','density_models','reference_data'));
    A(fullfile('04_atmosphere','density_models','jb2008'));
    A('05_data');
    % The space-weather fetchers live here. Without these three lines
    % data.spaceweather and data.jb2008_indices throw "undefined function"
    % (get_f107 / get_gfz_hpo / get_jb2008_indices), buildWorld swallows it in a
    % try/catch, W.swtable stays [], and every non-exponential drag run then dies
    % per-step with a misleading "pass opts.manual" message. The header above has
    % always listed data_sources; the path was never added.
    A(fullfile('05_data','data_sources','spaceweather','solar'));        % get_f107, get_omni2, get_f30
    A(fullfile('05_data','data_sources','spaceweather','geomag'));       % get_gfz_hpo
    A(fullfile('05_data','data_sources','spaceweather','model_inputs')); % get_jb2008_indices
    A(fullfile('05_data','data_sources','density_reference'));            % fetch_tudelft_density
    % ^ data.tudelft_density was a dead wrapper too: it calls fetch_tudelft_density,
    %   which was never shipped. That killed compare_density.m and the TU Delft
    %   overlay in validation.track_reference.
    A('11_compare');
    A('06_validation');
    A(fullfile('06_validation','realsat'));
    A('07_examples');   % TEMPLATE_16U, EXAMPLE_16U, TEMPLATE_propagation, show_16U
    A('08_test');

    % shared data / geometry payloads (referenced by packages)
    A('gravity_data');
    A('force_data');

    fprintf('Precision Orbit Propagator paths added (root: %s)\n', here);
end
