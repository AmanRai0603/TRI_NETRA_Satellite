function T = modes()
%ASILS.FSW.MODES  The flight software's controller states, and the MISSION MODE
%   (catalogue/modes) and guidance law each one serves. One table, read by the
%   FSW, the recorder, the metrics and the report, so a new controller state
%   is one row here plus its branch in asils.fsw.step.
%
%   state          mission mode       guidance   actuators doing the job
%   detumble       detumble           -          coils (B-dot family)
%   detumble_rcs   detumble           -          thrusters (rate damping)
%   spinup         sun_acquisition    -          coils (Standard Code L1)
%   sun_spin       sun_acquisition    -          coils (Standard Code L2)
%   sun_acq_rotor  sun_acquisition    -          momentum devices (Sun vector)
%   sun_mtq        sun_referencing    sun        coils, three-axis
%   sun_fine       sun_referencing    sun        momentum devices (+ coil / RCS dump)
%   nadir_mtq      nadir_pointing     nadir      coils, three-axis
%   nadir_fine     nadir_pointing     nadir      momentum devices (+ coil / RCS dump)
%   target_fine    target_pointing    target     momentum devices
%   slew_fine      slew               slew       momentum devices (+ RCS assist)
    T.state   = {'detumble', 'nadir_mtq', 'nadir_fine', 'target_fine', 'slew_fine', 'spinup', 'sun_spin', ...
                 'detumble_rcs', 'sun_acq_rotor', 'sun_mtq', 'sun_fine'};
    T.mission = {'detumble', 'nadir_pointing', 'nadir_pointing', 'target_pointing', 'slew', 'sun_acquisition', ...
                 'sun_acquisition', 'detumble', 'sun_acquisition', 'sun_referencing', 'sun_referencing'};
    T.guidance = {'', 'nadir', 'nadir', 'target', 'slew', '', '', '', '', 'sun', 'sun'};
    % the guidance law of each state as the flight software's guidance takes it (-1 none; 0 nadir, 1 target, 2 slew,
    % 3 inertial, 4 sun): the engine's config.rs GUID (no node holds it, S7.13's finding)
    T.guid = [-1, 0, 0, 1, 2, -1, -1, -1, -1, 4, 4];
end
