function rec = run(tn, scenario, varargin)
%TRINETRA.RUN  Fly one scenario of an opened design (trinetra.open) with its generated code (trinetra.build): the twin's
%   runner (asils.run) over the design's inputs and the design's code (trinetra.use), and file the run in the user's
%   copy, results/<scenario>/: its channels and manifest (naming the design it was flown from), the recording, the case
%   as flown, its figures and its report (drawn by the engine's plotting, as the engine's runs are: adcs figures,
%   adcs report).
%
%   rec = trinetra.run(tn, 'detumble_ais')
%   rec = trinetra.run(tn, 'nadir_hold_ais', 'seed', 3, 'set', struct('engine__duration_s', 600), 'quiet', true)
%   options: 'case' (an id the design holds; default the scenario's), 'seed', 'set', 'quiet', 'checkpoint' (as
%   asils.run); 'save' (default true), 'figures' and 'report' (default true: need the engine, adcs), 'out' (a folder in
%   place of results/<scenario>)
%   Owner: Agastya. Copyright (c) 2026 Agastya. All rights reserved.
    o = struct('case', '', 'seed', 1, 'set', struct(), 'quiet', false, 'checkpoint', '', 'save', true, 'figures', true, ...
               'report', true, 'out', '');
    for k = 1:2:numel(varargin)
        assert(isfield(o, varargin{k}), 'trinetra:run', 'trinetra.run has no option %s', varargin{k});
        o.(varargin{k}) = varargin{k+1};
    end
    assert(any(strcmp(tn.scenarios, scenario)), 'trinetra:run', 'the design %s holds no scenario %s', tn.file, scenario);
    c = trinetra.use(tn); %#ok<NASGU>
    S = asils.scenario.load(scenario);
    caseId = o.case;
    if isempty(caseId), caseId = S.case_id; end
    assert(any(strcmp(tn.cases, caseId)), 'trinetra:run', 'the design %s holds no case %s', tn.file, caseId);
    caseFile = fullfile(asils.util.root(), 'cases', [caseId '.csv']);
    args = {'seed', o.seed, 'set', o.set, 'quiet', o.quiet};
    if ~isempty(o.checkpoint), args = [args, {'checkpoint', o.checkpoint}]; end
    rec = asils.run(scenario, caseFile, args{:});
    if ~o.save, return, end
    out = o.out;
    if isempty(out), out = fullfile(tn.results, scenario); end
    asils.result.save(rec, out);
    if o.figures, adcs_({'figures', out, '--out', fullfile(out, 'figures')}, o.quiet); end
    if o.report, adcs_({'report', out}, o.quiet); end
    if ~o.quiet, fprintf('[trinetra] %s filed in %s\n', scenario, out); end
end

function adcs_(args, quiet)
% the engine's plotting of a run folder; a run is filed whatever it says
    q = cellfun(@asils.util.shellq, [{asils.util.engine()}, args], 'UniformOutput', false);
    [rc, txt] = system([strjoin(q, ' ') ' 2>&1']);
    if rc ~= 0
        warning('trinetra:plot', 'adcs %s: %s', args{1}, strtrim(txt));
    elseif ~quiet && ~isempty(strtrim(txt))
        fprintf('  %s\n', strtrim(txt));
    end
end
