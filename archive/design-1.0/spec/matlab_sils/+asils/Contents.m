% ASILS  ADCS SILS twin — plain-MATLAB SILS, the platform's second engine (SPEC.md §10.8)
%
% Your input is a case CSV; everything else is released with this download.
% To ask for a change, use the node forms in forms/ (manual/04_asking_for_changes.md).
%
% Entry points
%   run            - one scenario for one case and product
%   tune           - tune a product's algorithms for a case (same rule as adcs-tune)
%   version        - release, commit, catalogue and case format of this download
%   check          - verify every file against MANIFEST.sha256
%   app            - one window: case CSV, product, scenario, run, plots
%
% Packages
%   +case          - read, check, template: the fixed case CSV (adcs-case/1)
%   +scenario      - load, with case: references resolved
%   +product       - load: parts, counts, mounts, algorithms
%   +physics       - the same functions as adcs_core::physics, same names and argument order
%   +plant         - state, dynamics, RK4, tick order
%   +env           - field, density, J2, eclipse
%   +sensors       - descriptor-level sensor models
%   +actuators     - descriptor-level actuator models
%   +fsw           - one function per catalogue/algorithms/<id>.toml
%   +campaign      - run: nominal, montecarlo, edge, sweep, fault
%   +metrics       - the metric kinds of SPEC.md §10.3
%   +rng           - draw: the platform's counter-based draws, reproduced exactly
%   +rec           - write, read: adcs-rec/1 channels
%   +result        - save, open, import, index: result documents in store/, each beside its case
%   +viz           - run, campaign, compare, case
%   +data          - install: reference data that may not be redistributed
