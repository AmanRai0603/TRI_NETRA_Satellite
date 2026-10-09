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
    % the case as flown: the file's, or the design's when the run read it from a design database (asils.util.design)
    cf = rec.P.case.file;
    if ~asils.util.hasinput(cf), cf = fullfile(asils.util.root(), cf); end
    if ~isempty(asils.util.design()) && ~isempty(asils.util.inputkey(cf))
        fid = fopen(fullfile(dir_out, 'case.csv'), 'w'); fwrite(fid, unicode2native(asils.util.readtext(cf), 'UTF-8')); fclose(fid);
    else
        copyfile(rec.P.case.file, fullfile(dir_out, 'case.csv'));
    end
    f = fullfile(dir_out, 'manifest.json');
end
