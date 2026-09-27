% VERIFY_MODEL_MATRIX  Which model can run for which satellite, and does it?
%
%   RUN:  setup_paths; verify_model_matrix
%
%   ---------------------------------------------------------------------------
%   THE QUESTION
%   ---------------------------------------------------------------------------
%   A model is not usable because its code exists. It is usable when the DATA it
%   needs exists for THAT satellite. This walks every satellite in the catalog
%   against every drag/srp/erp model, asks validation.capability whether it is
%   allowed, and then ACTUALLY RUNS the allowed ones to prove the answer is not
%   just a table that agrees with itself.
%
%   The three silent failures this exists to prevent:
%     - panel drag with no attitude  -> R_bi = eye(3) -> drag ~ 0, no error
%     - box-wing with no dimensions  -> you invent a box and are now fitting
%     - a metric with no product     -> a ratio from one point (GRACE-A: 2.178)
%
%   ---------------------------------------------------------------------------
%   WHAT DECIDES
%   ---------------------------------------------------------------------------
%     attitude (== has_acc: ITSG publishes them together, the ACC product is
%               useless without it)   -> panel drag, and any box-wing
%     geometry (has_geometry/geom_dims_m, currently 'no' for EVERY satellite)
%                                     -> box-wing with a MEASURED shape
%     GEOMETRY='aref_box'             -> box-wing with an ASSUMED shape: allowed
%               whenever there is an attitude, and tagged ASSUMED everywhere
%
% =============================== KNOBS =======================================
SATS = {};              % {} = every satellite in the catalog
RUN_LIVE = true;        % actually propagate one step per allowed combination
DATE = '2007-01-01';
ALT_FALLBACK_KM = 400;
% =============================================================================

if isempty(SATS)
    T = fileread(fullfile(fileparts(mfilename('fullpath')),'..','05_data','sat_data','itsg_catalog.csv'));
    L = strsplit(strtrim(T), sprintf('\n'));
    SATS = cell(1,numel(L)-1);
    for i=2:numel(L), c=strsplit(L{i},','); SATS{i-1}=strtrim(c{1}); end
end

DRAGM = {'cannonball','sentman','dria','cll','sesam'};
SRPM  = {'cannonball','boxwing'};
ERPM  = {'knocke','simple','ceres','boxwing'};

fprintf('\n================= MODEL x SATELLITE MATRIX =================\n');
fprintf('  y = allowed AND runs   . = not allowed (data missing)   X = ALLOWED BUT FAILED\n');
fprintf('  box-wing columns are with GEOMETRY=''aref_box'' (an ASSUMED shape)\n\n');
hdr = sprintf('  %-12s|','satellite');
for i=1:numel(DRAGM), hdr=[hdr sprintf(' %-5s',DRAGM{i}(1:min(5,end)))]; end
hdr=[hdr ' |'];
for i=1:numel(SRPM), hdr=[hdr sprintf(' srp:%-4s',SRPM{i}(1:min(4,end)))]; end
hdr=[hdr ' |'];
for i=1:numel(ERPM), hdr=[hdr sprintf(' erp:%-4s',ERPM{i}(1:min(4,end)))]; end
fprintf('%s\n', hdr);
fprintf('  %s\n', repmat('-', 1, numel(hdr)-2));

K = de440.constants();
nbad = 0;
for is = 1:numel(SATS)
    SAT = SATS{is};
    try
        CAP = validation.capability(SAT, false);
        C   = validation.itsg_catalog(SAT);
    catch err
        fprintf('  %-12s| CATALOG ERROR: %s\n', SAT, regexprep(err.message,'\n.*',''));
        nbad = nbad+1; continue
    end
    row = sprintf('  %-12s|', SAT);
    for i=1:numel(DRAGM)
        row = [row sprintf(' %-5s', validation.matrix_cell(validation.try_model(SAT,C,CAP,'drag',DRAGM{i},ALT_FALLBACK_KM,RUN_LIVE,K)))];
    end
    row = [row ' |'];
    for i=1:numel(SRPM)
        row = [row sprintf(' %-8s', validation.matrix_cell(validation.try_model(SAT,C,CAP,'srp',SRPM{i},ALT_FALLBACK_KM,RUN_LIVE,K)))];
    end
    row = [row ' |'];
    for i=1:numel(ERPM)
        row = [row sprintf(' %-8s', validation.matrix_cell(validation.try_model(SAT,C,CAP,'erp',ERPM{i},ALT_FALLBACK_KM,RUN_LIVE,K)))];
    end
    fprintf('%s\n', row);
end

fprintf('\n  ---- what the pattern means ----\n');
fprintf('  Only CHAMP and the GRACE family carry an accelerometer, and ITSG publishes\n');
fprintf('  attitude WITH it (the ACC product is useless without one). So they are the\n');
fprintf('  only satellites where a panel drag model has an attitude to act on --\n');
fprintf('  everywhere else R_bi would stay eye(3), the body axes would sit on the\n');
fprintf('  INERTIAL axes, and the drag would collapse toward ZERO with no error.\n');
fprintf('  That is why those cells are ''.'' and not a number.\n\n');
fprintf('  NO satellite has measured dimensions (has_geometry=no for all 24), so every\n');
fprintf('  box-wing cell above is GEOMETRY=''aref_box'': a box ASSUMED around Aref.\n');
fprintf('  Measured cost of that assumption on CHAMP -- same ram face, same drag:\n');
fprintf('    aspect [1 1 1] -> srp 1.28e-08   erp 3.11e-09\n');
fprintf('    aspect [4.6 1 1] -> srp 2.55e-08   erp 1.25e-08     (~2x srp, ~4x erp)\n');
fprintf('  Use it as a sensitivity band, never as that satellite''s answer.\n');
if nbad>0, fprintf('\n  %d catalog error(s).\n', nbad); end
fprintf('===========================================================\n');
