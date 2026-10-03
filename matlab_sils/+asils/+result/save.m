function f = save(rec, dir_out)
%ASILS.RESULT.SAVE  File a finished run: its channels and manifest (asils.rec.write), the
%   recording itself (rec.mat) and a copy of its case, in store/results/<scenario>/.
%   Returns the manifest's path.
%
%   The run's figures and its report are drawn by the project's one plotting module, the same
%   for the engine's runs and the twin's (docs/MAIN_APP.md):
%       adcs figures store/results/<scenario> --out DIR [--format svg|pdf]
%       adcs report  store/results/<scenario>
%   asils.viz.run(rec, '', true) still puts the figures on the screen at the desk.
    if nargin < 2 || isempty(dir_out), dir_out = fullfile(asils.util.root(), 'store', 'results', rec.P.id); end
    asils.rec.write(rec, dir_out);
    save(fullfile(dir_out, 'rec.mat'), 'rec', '-v7');
    copyfile(rec.P.case.file, fullfile(dir_out, 'case.csv'));
    f = fullfile(dir_out, 'manifest.json');
end
