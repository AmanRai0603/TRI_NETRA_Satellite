function f = save(rec, dir_out)
%ASILS.RESULT.SAVE  File a finished run: channels (asils.rec.write), figures
%   (asils.viz.run), the recording itself (rec.mat) and a one-page HTML result,
%   in store/results/<scenario>/ beside a copy of its case.
    if nargin < 2 || isempty(dir_out), dir_out = fullfile(asils.util.root(), 'store', 'results', rec.P.id); end
    asils.rec.write(rec, dir_out);
    figs = asils.viz.run(rec, dir_out);
    save(fullfile(dir_out, 'rec.mat'), 'rec', '-v7');
    copyfile(rec.P.case.file, fullfile(dir_out, 'case.csv'));
    f = fullfile(dir_out, 'result.html');
    fid = fopen(f, 'w');
    fprintf(fid, '<!doctype html><meta charset="utf-8"><title>%s</title>\n', rec.P.id);
    fprintf(fid, '<body style="font-family:sans-serif;max-width:1100px;margin:auto">\n');
    fprintf(fid, '<h1>%s</h1><p>%s &middot; case <b>%s</b> (%s) &middot; product <b>%s</b> &middot; seed %d &middot; %s</p>\n', ...
        rec.P.scenario.label, asils.version(), rec.P.case.id, rec.P.case.title, rec.P.dev.id, rec.P.seed, 'owner: Agastya');
    fprintf(fid, '<table border=1 cellpadding=4 style="border-collapse:collapse"><tr><th>metric</th><th>value</th><th>required</th><th>verdict</th></tr>\n');
    for i = 1:numel(rec.metrics)
        m = rec.metrics(i); v = '&ndash;';
        if isfinite(m.pass), if m.pass, v = 'PASS'; else, v = 'FAIL'; end, end
        fprintf(fid, '<tr><td>%s</td><td>%.4g %s</td><td>%.4g</td><td>%s</td></tr>\n', m.id, m.value, m.unit, m.req, v);
    end
    fprintf(fid, '</table>\n');
    for i = 1:numel(figs)
        [~, n, e] = fileparts(figs{i}); fprintf(fid, '<p><img src="%s%s" style="max-width:100%%"></p>\n', n, e);
    end
    fprintf(fid, '<p style="color:#777">Copyright (c) 2026 Agastya. All rights reserved.</p></body>\n');
    fclose(fid);
end
